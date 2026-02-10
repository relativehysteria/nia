use std::thread;
use std::sync::mpsc;
use crate::config::FeedId;

pub enum DownloadEvent {
    DownloadFeed(FeedId)
}

pub struct DownloadState {
}

impl DownloadState {
    /// Spawn the background thread that will handle downloads.
    pub fn spawn_downloader_thread() -> Self {
        thread::spawn(move || {});
        Self {}
    }
}
