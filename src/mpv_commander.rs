use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    process::{Child, Command, Stdio},
    sync::mpsc::{Receiver, Sender},
    thread::{self, JoinHandle},
};

use crate::{central::CentralCommand, log_debug, log_error};

#[derive(Debug)]
pub(crate) enum MpvIpcResponse {
    FileLoaded,
    PlaybackRestart,
}

#[derive(Debug)]
pub(crate) enum MpvIpcRequest {
    LoadUrl(String),
    Seek(f64),
}

impl Into<String> for MpvIpcRequest {
    fn into(self) -> String {
        match self {
            // not proper json encoding, but it works
            MpvIpcRequest::LoadUrl(url) => {
                format!("{{ \"command\": [\"loadfile\", \"{}\"] }}\n", url)
            }
            MpvIpcRequest::Seek(timestamp) => {
                format!(
                    "{{ \"command\": [\"seek\", {}, \"absolute\"] }}\n",
                    timestamp
                )
            }
        }
    }
}

pub(crate) fn start_mpv() -> Child {
    Command::new("mpv")
        .arg("--script=seconds.lua")
        .arg("--input-ipc-server=/tmp/mpvsocket")
        .arg("--force-window")
        .arg("--idle")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to exec mpv process")
}

fn send_to_mpv(mpv_ipc_tx: Sender<MpvIpcRequest>, command: MpvIpcRequest) {
    match mpv_ipc_tx.send(command) {
        Ok(_) => {}
        Err(e) => {
            log_error!("Failed to send command to MPV: {}", e);
        }
    }
}

pub(crate) fn mpv_load_url(mpv_ipc_tx: Sender<MpvIpcRequest>, url: &str) {
    send_to_mpv(mpv_ipc_tx, MpvIpcRequest::LoadUrl(url.to_string()));
}

pub(crate) fn mpv_seek(mpv_ipc_tx: Sender<MpvIpcRequest>, timestamp: f64) {
    send_to_mpv(mpv_ipc_tx, MpvIpcRequest::Seek(timestamp));
}

pub(crate) fn spawn_mpv_ipc_threads(
    mpv_ipc_rx: Receiver<MpvIpcRequest>,
    central_tx: Sender<CentralCommand>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let stream: UnixStream;

        // try to connect to the socket until it works
        let mut attempts: u32 = 0;
        loop {
            if attempts > 40 {
                log_error!("Failed to connect to MPV socket after ~2s worth of attempts.");
                std::process::exit(1);
            }
            match UnixStream::connect("/tmp/mpvsocket") {
                Ok(s) => {
                    stream = s;
                    log_debug!("Connected to MPV socket.");
                    break;
                }
                Err(e) => {
                    log_debug!("MPV socket not ready yet: {}", e);
                    attempts += 1;
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }

        let stream_for_send = stream.try_clone().expect("Failed to clone MPV socket");
        let stream_for_recv = stream.try_clone().expect("Failed to clone MPV socket");
        spawn_mpv_ipc_req_thread(stream_for_send, mpv_ipc_rx);
        spawn_mpv_ipc_res_thread(stream_for_recv, central_tx.clone());
    })
}

fn spawn_mpv_ipc_req_thread(
    mut stream: UnixStream,
    ipc_request_rx: Receiver<MpvIpcRequest>,
) -> JoinHandle<()> {
    thread::spawn(move || loop {
        let command: String;
        match ipc_request_rx.recv() {
            Err(_) => {
                log_debug!(
                    "Queue of commands to send to mpv ran dry, so stopping the IPC request thread."
                );
                break;
            }
            Ok(read_command) => {
                command = read_command.into();
            }
        }
        command.lines().for_each(|line| {
            log_debug!("[MPV] > {}", line);
        });
        match stream.write_all(command.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                log_error!("Failed to send command to MPV: {}", e);
                match e.raw_os_error() {
                    Some(32) => {
                        log_debug!("MPV socket closed, so quitting the application.");
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }
    })
}

fn spawn_mpv_ipc_res_thread(
    stream: UnixStream,
    central_tx: Sender<CentralCommand>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        log_debug!("MPV socket closed, so quitting the application");
                        std::process::exit(0);
                    }
                    let trimmed_line = line.trim_end();
                    log_debug!("[MPV] < {}", trimmed_line);
                    match trimmed_line {
                        r#"{"event":"playback-restart"}"# => {
                            central_tx
                                .send(CentralCommand::MpvIpcEvent(MpvIpcResponse::PlaybackRestart))
                                .unwrap();
                        }
                        r#"{"event":"file-loaded"}"# => {
                            central_tx
                                .send(CentralCommand::MpvIpcEvent(MpvIpcResponse::FileLoaded))
                                .unwrap();
                        }
                        _ => {}
                    }
                }
                Err(err) => {
                    log_error!("Failed to read response from MPV: {}", err);
                    break;
                }
            }
        }
    })
}
