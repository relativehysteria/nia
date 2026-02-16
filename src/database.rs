use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::path::Path;
use std::thread;
use std::sync::Arc;
use crate::config::{FeedId, Post, FeedConfig, Posts};

/// A database request from the application to the database.
pub enum DatabaseRequest {
    /// Save the specified posts into database.
    SavePosts {
        feed_url: Arc<str>,
        posts: Vec<Post>
    },
}

/// The application end of the channel between the channel and the feed
/// database.
pub struct DatabaseChannel {
    /// Channel for database requests from the application to the database.
    pub request_tx: mpsc::Sender<DatabaseRequest>,
}

impl DatabaseChannel {
    /// Spawn the background database thread that will handle all permanent
    /// feed storage accesses.
    pub fn spawn_database_thread<P: AsRef<Path>>(
        data_dir: P,
        cfg: &mut FeedConfig
    ) -> Self {
        // Spawn the channels for the database requests and responses.
        let (request_tx, request_rx) = mpsc::channel::<DatabaseRequest>();

        // Spawn the database.
        let db = Database::new(data_dir);

        // Load all posts into the feed config.
        for section in &mut cfg.sections {
            for feed in &mut section.feeds {
                let feed_url = feed.url.as_str();
                let posts = db.load_feed(feed_url);
                feed.posts = posts.into();
            }
        }

        // Spawn the database thread.
        thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                match request {
                    DatabaseRequest::SavePosts { feed_url, posts } => {
                        db.save_posts(&feed_url, posts)
                    },
                }
            }
        });

        // Return the application end.
        Self { request_tx }
    }
}

/// Implementation of the database.
struct Database {
    /// The internal sled database state.
    db: sled::Db,
}

impl Database {
    /// Create a new database.
    fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        let db = sled::open(data_dir).expect("Failed to open sled db");
        Self { db }
    }

    /// Open (or create) the "posts" tree.
    fn posts_tree(&self) -> sled::Tree {
        self.db.open_tree("posts").expect("Failed to open posts tree")
    }

    /// Make a sled key for a post.
    fn make_key(feed_url: &str, post: &Post) -> Vec<u8> {
        let mut key = Vec::with_capacity(
            feed_url.len() + 8 + post.id.0.len());

        // Feed URL bytes.
        key.extend_from_slice(feed_url.as_bytes());

        // Inverted timestamp (newest-first).
        let ts = post.published.timestamp() as u64;
        let inverted = u64::MAX - ts;
        key.extend_from_slice(&inverted.to_be_bytes());

        // Post ID.
        key.extend_from_slice(post.id.0.as_bytes());

        key
    }

    /// Get the prefix for scanning all posts of a feed.
    fn feed_prefix(feed_url: &str) -> Vec<u8> {
        feed_url.as_bytes().to_vec()
    }

    /// Save posts to the database.
    pub fn save_posts(&self, feed_url: &str, posts: Vec<Post>) {
        let tree = self.posts_tree();

        for post in posts {
            let key = Self::make_key(feed_url, &post);
            let value = postcard::to_stdvec(&post)
                .expect("Failed to serialize post");
            tree.insert(key, value).expect("Failed to insert post");
        }

        tree.flush().expect("Failed to flush posts tree");
    }

    /// Load all posts for a feed.
    pub fn load_feed(&self, feed_url: &str) -> Vec<Post> {
        let tree = self.posts_tree();
        let prefix = Self::feed_prefix(feed_url);

        tree.scan_prefix(prefix)
            .filter_map(|res| res.ok())
            .filter_map(|(_, v)| postcard::from_bytes::<Post>(&v).ok())
            .collect()
    }
}
