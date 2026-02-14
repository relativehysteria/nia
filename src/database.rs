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

    /// Load all post metadata for a feed.
    LoadFeed {
        feed: FeedId
    },

    /// Mark a post as read or unread.
    MarkRead {
        read: bool,
        feed: FeedId,
        post_idx: usize,
    },
}

/// A database response to the application.
pub enum DatabaseResponse {
    /// Post metadata loaded.
    ///
    /// Unread posts will also have their contents loaded.
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

        // Spawn the database thread.
        thread::spawn(move || {
            // // Tracks unread posts: FeedId -> indices of unread posts
            // let mut unread_index: HashMap<FeedId, HashSet<usize>> =
            //     HashMap::new();

            // // On startup, try to load existing unread indices
            // let unread = data_dir.join("unread.json");
            // if unread.exists() {
            //     let file = fs::File::open(unread)
            //         .expect("Couldn't open the post read index.");
            //     unread_index = serde_json::from_reader(file)
            //         .unwrap_or_default();
            // }

            while let Ok(request) = request_rx.recv() {
                match request {
                    DatabaseRequest::SavePosts { feed, posts } => {
                    },
                    DatabaseRequest::LoadFeed { feed } => {
                    },
                    DatabaseRequest::MarkRead { read, feed, post_idx } => {
                    },
                }
            }
        });

        // Return the application end.
        Self { request_tx, response_rx }
    }
}

/// The database state.
struct Database<P: AsRef<Path>> {
    /// Directory where the data will be stored.
    data_dir: P,
}
