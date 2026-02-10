use std::thread;
use std::sync::mpsc;
use crate::config::{FeedId, FeedConfig};

/// A map of sections to feeds to URLs.
#[derive(Debug)]
pub struct UrlMap(pub Vec<Vec<String>>);

impl From<&FeedConfig> for UrlMap {
    /// Given a feed config, create a `FeedId -> URL` map.
    fn from(feed_config: &FeedConfig) -> Self {
        let map = feed_config
            .sections
            .iter()
            .map(|section| {
                section
                    .feeds
                    .iter()
                    .map(|feed| feed.url.clone())
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>();

        Self(map)
    }
}

/// A download request from the application to the downloader.
pub enum DownloadRequest {
    /// Download a single feed.
    DownloadFeed {
        feed: FeedId,
        url: String,
    },

    /// Download all feeds.
    ///
    /// The map here is
    DownloadAll(UrlMap),
}

/// A response from the downloader to the app.
pub enum DownloadResponse {
    /// The downloader has started downloading a feed.
    DownloadStarted(FeedId),

    /// The downloader has finished downloading a feed.
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
                    DownloadRequest::DownloadAll(map) => {
                        for (section_idx, section) in map.0.iter().enumerate() {
                            for (feed_idx, _) in section.iter().enumerate() {
                                let feed = FeedId { section_idx, feed_idx };
                                let _ = response_tx.send(
                                    DownloadResponse::DownloadStarted(feed));
                                break;
                            }
                        }
                    },
                }
            }
        });

        // Return the application end
        Self { request_tx, response_rx }
    }
}
