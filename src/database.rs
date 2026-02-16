use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::path::Path;
use std::thread;
use crate::config::{FeedId, Post};

/// A database request from the application to the database.
pub enum DatabaseRequest {
    /// Save the specified posts into database.
    SavePosts {
        feed: FeedId,
        posts: Vec<Post>
    },

    /// Load all posts for a feed.
    LoadFeed {
        feed: FeedId
    },
}

/// A database response to the application.
pub enum DatabaseResponse {
    /// Posts loaded for a feed.
    FeedLoaded {
        feed: FeedId,
        posts: Vec<Post>,
    },
}

/// The application end of the channel between the channel and the feed
/// database.
pub struct DatabaseChannel {
    /// Channel for database requests from the application to the database.
    pub request_tx: mpsc::Sender<DatabaseRequest>,

    /// Channel for database responses from the database to the application.
    pub response_rx: mpsc::Receiver<DatabaseResponse>,
}

impl DatabaseChannel {
    /// Spawn the background database thread that will handle all permanent
    /// feed storage accesses.
    pub fn spawn_database_thread() -> Self {
        // Spawn the channels for the database requests and responses.
        let (request_tx, request_rx) = mpsc::channel::<DatabaseRequest>();
        let (response_tx, response_rx) = mpsc::channel::<DatabaseResponse>();

        // Spawn the database.
        let db = Database::new("/tmp/nia_test");

        // Spawn the database thread.
        thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                match request {
                    DatabaseRequest::SavePosts { feed, posts } => {
                    },
                    DatabaseRequest::LoadFeed { feed } => {
                    },
                }
            }
        });

        // Return the application end.
        Self { request_tx, response_rx }
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
        let db = sled::open(data_dir).expect("failed to open sled db");
        Self { db }
    }
}
