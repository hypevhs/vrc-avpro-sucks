use central::CentralCommand;
use signal_hook::{consts::SIGINT, iterator::Signals};
use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};
use vrc_log_reader::{VrcLogWatcher, VrcLogWatcherEvent};

use mpv_commander::start_mpv;

use crate::{
    central::Central, mpv_commander::{spawn_mpv_ipc_threads, MpvIpcRequest}
};

mod common;
mod mpv_commander;
mod vrc_log_reader;
mod vrc_log_reader_tests;
mod central;

// TODO: join and unwrap all threads

fn main() {
    spawn_signal_handler_thread();

    let mut child = start_mpv();

    // We use channels for cross-thread communication, since moving data ownership around isn't feasible.
    // For example, MPV IPC and VrcLogReader and VrcLogWatcher can't all own references to each other.

    // this channel is used to send commands to the mpv process. it is read by the mpv IPC thread.
    let (mpv_ipc_tx, mpv_ipc_rx) = mpsc::channel::<MpvIpcRequest>();

    // This is the central dispatch. It runs on the main thread.
    // When created, it exposes a Sender channel, which can be cloned and handed out to other components, like MPV IPC
    // and the log reader and watchers.
    // Then, it receives responses from those other components. Central also holds some "global" state, which
    // determines how exactly to react to those responses.
    let mut central = Central::new(mpv_ipc_tx.clone());
    let central_tx = &central.central_tx.clone();
    spawn_mpv_ipc_threads(mpv_ipc_rx, central_tx.clone());

    let lines_read_initially = central.do_initial();

    spawn_log_watcher_thread(central_tx.clone(), lines_read_initially);

    // this should block forever, until some kind of exit condition is met.
    central.run_central_dispatch();

    // TODO: add an exit condition for central dispatch.

    let res = child.wait().expect("mpv process wasn't running");
    let code = res.code().expect("mpv process didn't exit with a code");
    log_debug!("mpv process exited with code: {}", code);
}

fn spawn_log_watcher_thread(
    central_tx: Sender<CentralCommand>,
    start_after_line: u64,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut vlw = VrcLogWatcher::from_latest();
        vlw.watch_file(
            start_after_line,
            |found_url| {
                log_debug!("Video URL found: {}", found_url.url);
                central_tx
                    .send(CentralCommand::VrcLogWatcherEvent(
                        VrcLogWatcherEvent::FoundUrl(found_url),
                    ))
                    .unwrap();
            },
            |found_seek| {
                log_debug!("Seek event found: {}", found_seek.seek_offset);
                central_tx
                    .send(CentralCommand::VrcLogWatcherEvent(
                        VrcLogWatcherEvent::FoundSeek(found_seek),
                    ))
                    .unwrap();
            },
        );
    })
}

fn spawn_signal_handler_thread() -> JoinHandle<()> {
    thread::spawn(|| {
        let mut signals = Signals::new(&[SIGINT]).expect("Failed to create signal iterator");
        for signal in signals.forever() {
            match signal {
                SIGINT => {
                    log_debug!("Received SIGINT, exiting");
                    std::process::exit(0);
                }
                _ => unreachable!(),
            }
        }
    })
}
