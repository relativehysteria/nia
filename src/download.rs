use std::thread;
use std::sync::mpsc;
use atom_syndication::Feed as AtomFeed;
use rss::Channel as RssChannel;
use url::Url;
use crate::config::{FeedId, FeedConfig, Post};

/// A map of sections to feeds to URLs.
#[derive(Debug)]
pub struct UrlMap(pub Vec<Vec<Url>>);

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
                    .collect::<Vec<Url>>()
            })
            .collect::<Vec<Vec<Url>>>();

        Self(map)
    }
}

/// A download request from the application to the downloader.
pub enum DownloadRequest {
    /// Download a single feed.
    Feed {
        feed: FeedId,
        url: Url,
    },

    /// Download all feeds.
    ///
    /// The map here is
    All(UrlMap),
}

/// A response from the downloader to the app.
pub enum DownloadResponse {
    /// The downloader has started downloading a feed.
    Started(FeedId),

    /// The downloader couldn't download the feed.
    Failed(FeedId),

    /// The downloader has finished downloading a feed.
    Finished {
        feed: FeedId,
        posts: Vec<Post>,
    },
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
                    // Immediately start a downloader when downloading one feed.
                    DownloadRequest::Feed { feed, url } => {
                        let feed = vec![(feed, url)];
                        spawn_feed_downloader(feed, response_tx.clone());
                    },

                    // Start one downloader per section when downloading all
                    // feeds.
                    DownloadRequest::All(map) => {
                        let map = map.0.into_iter();
                        for (section_idx, section) in map.enumerate() {
                            let feeds = section
                                .into_iter()
                                .enumerate()
                                .map(|(feed_idx, url)| {
                                    (FeedId { section_idx, feed_idx, }, url)
                                }).collect::<Vec<(FeedId, Url)>>();

                            spawn_feed_downloader(feeds, response_tx.clone());
                        }
                    },
                }
            }
        });

        // Return the application end.
        Self { request_tx, response_rx }
    }
}

/// Spawn a thread that downloads `feeds` sequentially.
fn spawn_feed_downloader(
    feeds: Vec<(FeedId, Url)>,
    response_tx: mpsc::Sender<DownloadResponse>,
) {
    std::thread::spawn(move || {
        for (feed, url) in feeds.into_iter() {
            // Tell the app we have started the download.
            let _ = response_tx.send(DownloadResponse::Started(feed.clone()));

            // Do the actual download.
            let result = reqwest::blocking::get(String::from(url))
                .and_then(|r| r.error_for_status())
                .and_then(|r| r.text());

            // If we got an error for this feed, just go next.
            let Ok(body) = result else {
                let _ = response_tx.send(
                    DownloadResponse::Failed(feed.clone()));
                continue;
            };

            // Extract the urls.
            let mut posts = if let Ok(atom) = body.parse::<AtomFeed>() {
                extract_from_atom(&atom)
            } else if let Ok(rss) = body.parse::<RssChannel>() {
                extract_from_rss(&rss)
            } else {
                Vec::new()
            };

            // Sort the posts by date.
            posts.sort_unstable_by(|a, b| a.published.cmp(&b.published));

            // Tell the app we have finished the download.
            let _ = response_tx
                .send(DownloadResponse::Finished { feed, posts });
        }
    });
}

/// Parse a valid URL from `s` and push it into `acc`.
fn push_url(acc: &mut Vec<Url>, s: &str) {
    // TODO: Handle relative links.

    // These checks are not expensive enough to warrant something more optimized
    if let Ok(url) = Url::parse(s) {
        if !acc.contains(&url) {
            acc.push(url);
        }
    }
}

/// Parse valid URLs from `s` and push them into `acc`.
fn extract_urls_from_text(acc: &mut Vec<Url>, s: &str) {
    let mut finder = linkify::LinkFinder::new();
    finder.kinds(&[linkify::LinkKind::Url]);

    for link in finder.links(s).map(|link| link.as_str()) {
        push_url(acc, link);
    }
}

/// Extract the posts from an Atom feed.
///
/// All of the posts will be marked as unread. It is up to the application to
/// make sure that before read posts are marked as such.
fn extract_from_atom(feed: &AtomFeed) -> Vec<Post> {
    let mut posts = Vec::new();

    // Go through each post.
    for entry in feed.entries() {
        // Set the metadata for this post.
        let id = entry.id.clone().into();
        let name = entry.title.value.clone();
        let published = entry.updated.to_utc();

        // Parse the URLs from this post.
        let mut urls = Vec::new();

        for link in entry.links() {
            push_url(&mut urls, link.href())
        }

        if let Some(content) = entry.content().and_then(|c| c.value()) {
            extract_urls_from_text(&mut urls, content);
        }

        if let Some(summary) = entry.summary() {
            extract_urls_from_text(&mut urls, summary);
        }

        // Save the post.
        let read = false;
        posts.push(Post { urls, id, name, published, read });
    }

    posts
}

/// Extract the posts from an RSS feed.
///
/// All of the posts will be marked as unread. It is up to the application to
/// make sure that before read posts are marked as such.
fn extract_from_rss(channel: &RssChannel) -> Vec<Post> {
    let mut posts = Vec::new();

    // Go through each post.
    for item in channel.items() {
        // Set the metadata for this post. Unlike Atom, RSS requires almost no
        // metadata for posts. If we don't have much to work with, we'll do it
        // ourselves.
        let name = item.title.clone()
            .or_else(|| item.description.as_ref()
                .map(|d| truncate_chars(&d, 20)))
            .unwrap_or_else(|| "Untitled".to_string());
        let published = item.pub_date.as_ref()
            .and_then(|date| chrono::DateTime::parse_from_rfc2822(&date).ok())
            .map(|date| date.with_timezone(&chrono::Utc))
            .unwrap_or_else(|| chrono::Utc::now());
        let id = item.guid.as_ref().map(|g| g.value.clone())
            .unwrap_or_else(|| hash(&format!("{:?} {:?}", published, name)))
            .into();

        // Parse the URLs from this post.
        let mut urls = Vec::new();

        if let Some(link) = item.link() {
            push_url(&mut urls, link);
        }

        if let Some(desc) = item.description() {
            extract_urls_from_text(&mut urls, desc);
        }

        if let Some(content) = item.content() {
            extract_urls_from_text(&mut urls, content);
        }

        // Save the post.
        let read = false;
        posts.push(Post { id, name, urls, published, read });
    }

    posts
}

/// A function that generates a stable hash for `s`.
fn hash(s: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;

    for byte in s.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    hash.to_string()
}

// Utility function to truncate a string to at most `n` characters safely.
fn truncate_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
