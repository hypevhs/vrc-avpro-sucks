use std::sync::mpsc::{self, Receiver, Sender};

use chrono::{DateTime, Local};
use regex::Regex;

use crate::{
    log_debug,
    mpv_commander::{mpv_load_url, mpv_seek, MpvIpcRequest, MpvIpcResponse},
    vrc_log_reader::{FoundSeek, UrlAndSeekResult, VrcLogReader, VrcLogWatcherEvent},
};

pub(crate) enum CentralCommand {
    MpvIpcEvent(MpvIpcResponse),
    VrcLogWatcherEvent(VrcLogWatcherEvent),
}

pub(crate) struct Central {
    central_rx: Receiver<CentralCommand>,
    pub(crate) central_tx: Sender<CentralCommand>,
    mpv_ipc_tx: Sender<MpvIpcRequest>,

    // When a URL becomes Now Playing, we'll load it into MPV.
    // But we have to wait for MPV to fully load the new URL before we can seek it properly.
    // This structure holds any state for this in-between period.
    // As soon as MPV reports back that it's loaded, this gets reset to None.
    mid_loading_state: Option<MidLoadingState>,
}

struct InitialStateResult {
    initial_state: Option<InitialState>,
    lines_read_initially: u64,
}

struct InitialState {
    timing_state: TimingState,
    url: String,
}

struct MidLoadingState {
    timing_state: TimingState,
}

#[derive(Clone)]
struct TimingState {
    timestamp: DateTime<Local>,
    additional_offset: f64,
}

impl Central {
    pub(crate) fn new(mpv_ipc_tx: Sender<MpvIpcRequest>) -> Self {
        let (central_tx, central_rx) = mpsc::channel::<CentralCommand>();
        Self {
            central_rx,
            central_tx,
            mpv_ipc_tx,
            mid_loading_state: None,
        }
    }

    pub(crate) fn do_initial(&mut self, player_name_regex: &Option<Regex>) -> u64 {
        let initial_state_result = read_initial_state_from_log(player_name_regex);

        if let Some(initial_state) = initial_state_result.initial_state {
            log_debug!("Initial state found.");
            let timing_state = &initial_state.timing_state;
            log_debug!("Timestamp: {}", timing_state.timestamp);
            log_debug!("Additional offset: {}", timing_state.additional_offset);
            log_debug!("Given that right now is {}", chrono::Local::now());
            log_debug!(
                "At this rate, we'll seek to {} (and counting), once MPV has loaded the file.",
                calculate_seek_from_timing_state(&timing_state)
            );

            mpv_load_url(self.mpv_ipc_tx.clone(), &initial_state.url);
            self.mid_loading_state = Some(MidLoadingState {
                timing_state: timing_state.clone(),
            });
        }

        initial_state_result.lines_read_initially
    }

    pub(crate) fn run_central_dispatch(mut self) {
        for response in self.central_rx {
            match response {
                CentralCommand::MpvIpcEvent(MpvIpcResponse::PlaybackRestart) => {}
                CentralCommand::MpvIpcEvent(MpvIpcResponse::FileLoaded) => {
                    if let Some(state) = self.mid_loading_state {
                        self.mid_loading_state = None;
                        // We were waiting on MPV to load the file. We're finally allowed to seek.
                        let target_timestamp =
                            calculate_seek_from_timing_state(&state.timing_state);
                        mpv_seek(self.mpv_ipc_tx.clone(), target_timestamp);
                    }
                }
                CentralCommand::VrcLogWatcherEvent(VrcLogWatcherEvent::FoundUrl(found_url)) => {
                    mpv_load_url(self.mpv_ipc_tx.clone(), &found_url.url);

                    // By the time this video loads in MPV, several seconds will likely have passed.
                    // Let's say the clock starts ticking right when the log watcher reports FoundUrl.
                    // FIXME: Though maybe it'd be better to wait for _TvPlay? Research needed.
                    self.mid_loading_state = Some(MidLoadingState {
                        timing_state: TimingState {
                            timestamp: found_url.timestamp,
                            // Ingame players start playing new content from the beginning.
                            additional_offset: 0.0,
                        },
                    });
                }
                CentralCommand::VrcLogWatcherEvent(VrcLogWatcherEvent::FoundSeek(found_seek)) => {
                    if let Some(state) = self.mid_loading_state {
                        // We're still loading, so trying to seek now would be ignored.
                        // Let's just update the mid-loading state with this new, fresher seek estimate.
                        self.mid_loading_state = Some(MidLoadingState {
                            timing_state: TimingState {
                                timestamp: found_seek.timestamp,
                                additional_offset: found_seek.seek_offset,
                            },
                            ..state
                        });
                    } else {
                        // MPV is loaded. Seeks are allowed.
                        let new_timestamp = timestamp_from_seek_line(&found_seek);
                        mpv_seek(self.mpv_ipc_tx.clone(), new_timestamp);
                    }
                }
            }
        }
    }
}

fn timestamp_from_seek_line(found_seek: &FoundSeek) -> f64 {
    // how long has it been since this timestamp?
    let now = chrono::Local::now();
    let duration = now.signed_duration_since(found_seek.timestamp);
    // add this duration to the seek offset, which has also been returned from the log file
    let new_seek_offset = found_seek.seek_offset + duration.num_milliseconds() as f64 / 1000.0;

    new_seek_offset
}

// Do this as late as possible.
fn calculate_seek_from_timing_state(state: &TimingState) -> f64 {
    // how long has it been since this timestamp?
    let now = chrono::Local::now();
    let duration = now.signed_duration_since(state.timestamp);
    // add this duration to the seek offset, which may have also been returned from the log file
    let new_seek_offset = state.additional_offset + duration.num_milliseconds() as f64 / 1000.0;

    new_seek_offset
}

fn read_initial_state_from_log(player_name_regex: &Option<Regex>) -> InitialStateResult {
    let mut vlr = VrcLogReader::from_latest(player_name_regex);
    let url_and_seek = vlr.get_latest_url_and_seek();
    match url_and_seek {
        UrlAndSeekResult::Nothing(lines_read_initially) => {
            log_debug!("No URL found in the log file so far. We'll wait for some.");
            InitialStateResult {
                initial_state: None,
                lines_read_initially,
            }
        }
        UrlAndSeekResult::Url(found_url, lines_read_initially) => {
            log_debug!("URL found: {}", found_url.url);

            InitialStateResult {
                initial_state: Some(InitialState {
                    timing_state: TimingState {
                        timestamp: found_url.timestamp,
                        additional_offset: 0.0,
                    },
                    url: found_url.url,
                }),
                lines_read_initially,
            }
        }
        UrlAndSeekResult::UrlAndSeek(found_url, found_seek, lines_read_initially) => {
            InitialStateResult {
                initial_state: Some(InitialState {
                    timing_state: TimingState {
                        timestamp: found_seek.timestamp,
                        additional_offset: found_seek.seek_offset,
                    },
                    url: found_url.url,
                }),
                lines_read_initially,
            }
        }
    }
}
