use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::Duration;

pub struct DownloadState {
}

impl DownloadState {
    /// Spawn the background thread that will handle downloads.
    fn spawn_downloader_thread() -> Self {
        Self {}
    }
}
