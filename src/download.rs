use std::thread;
use std::sync::mpsc;
use crate::config::FeedId;

pub enum DownloadRequest {
    DownloadFeed {
        feed: FeedId,
        url: String,
    },
    DownloadAll {
    },
}

pub enum DownloadResponse {
    DownloadStarted(FeedId),
    DownloadFinished(FeedId),
}

/// The application end of the channel between the application and the
/// downloader.
pub struct DownloadChannel {
    /// Channel for download requests from the application to the downloader.
    pub request_tx: mpsc::Sender<DownloadRequest>,

    /// Channel for download responses from the downloader to the application.
    pub response_rx: mpsc::Receiver<DownloadResponse>,
}

impl DownloadChannel {
    /// Spawn the background thread that will handle downloads.
    pub fn spawn_downloader_thread() -> Self {
        // Spawn the channels for download requests and responses.
        let (request_tx, request_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();

        // Spawn the downloader thread.
        thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                match request {
                    DownloadRequest::DownloadFeed { feed, url } => {
                        let _ = response_tx.send(
                            DownloadResponse::DownloadStarted(feed));
                    },
                    DownloadRequest::DownloadAll { .. } => {},
                }
            }
        });

        // Return the application end
        Self { request_tx, response_rx }
    }
}
