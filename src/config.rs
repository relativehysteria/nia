//! Config parsing and stuff.

use std::sync::Arc;
use std::io::{self, BufRead};
use std::path::PathBuf;
use url::Url;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// A parsed config file.
#[derive(Debug, Clone)]
pub struct FeedConfig {
    /// A vector of sections parsed from the config.
    pub sections: Vec<Section>,
}

/// A parsed section containing 0 or more feeds.
#[derive(Debug, Clone)]
pub struct Section {
    /// Title of the section.
    pub title: Arc<str>,

    /// A vector of the feeds in this section.
    pub feeds: Vec<Feed>,
}

/// A feed with a title and the url of the feed.
#[derive(Debug, Clone)]
pub struct Feed {
    /// Title of this feed that will be shown in the TUI.
    pub title: Arc<str>,

    /// The provided url of this feed.
    pub url: Url,

    /// The posts in the feed.
    pub posts: Posts,
}

/// A vector of posts sorted by their published date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posts {
    /// The inner vector of posts.
    inner: Vec<Post>,

    /// Number of unread posts within inner.
    unread: usize,
}

impl From<Vec<Post>> for Posts {
    fn from(mut v: Vec<Post>) -> Self {
        v.sort_unstable_by(|a, b| a.published.cmp(&b.published).reverse());
        let unread = v.iter().filter(|p| !p.read).count();

        Self {
            inner: v,
            unread,
        }
    }
}

impl From<Post> for Posts {
    fn from(post: Post) -> Self {
        Self {
            unread: (!post.read) as usize,
            inner: vec![post],
        }
    }
}

impl Posts {
    /// Create a new post vector.
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            unread: 0,
        }
    }

    /// Append posts from `other` to this vector.
    pub fn append(&mut self, other: Posts) {
        other.inner.into_iter().for_each(|post| self.insert(post));
    }

    /// only retain elements specified by the predicate.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Post) -> bool
    {
        self.inner.retain(|post| {
            let keep = f(post);
            if !keep && !post.read {
                self.unread -= 1;
            }
            keep
        });
    }

    /// Insert a new post into the vector.
    pub fn insert(&mut self, post: Post) {
        let idx = self.inner
            .binary_search_by(|p| p.cmp(&post).reverse())
            .unwrap_or_else(|p| p);

        if !post.read {
            self.unread += 1;
        }

        self.inner.insert(idx, post);
    }

    /// Check if the vector contains `post` already.
    pub fn contains(&self, post: &Post) -> bool {
        self.inner.binary_search_by(|p| p.cmp(post).reverse()).is_ok()
    }

    /// Get the length of the posts vector.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Get the number of unread posts within this feed.
    pub fn unread(&self) -> usize {
        self.unread
    }

    /// Mark a post as read/unread.
    pub fn mark_read(&mut self, post_id: &PostId, read: bool) {
        // Get the post if it exists.
        let Some(post) = self.get_by_id_mut(post_id) else {
            return;
        };

        // If it already has the same read mark, there's nothing to change.
        if post.read == read { return; }

        // Mark the post
        post.read = read;

        // Change the tracking unread count.
        if read {
            self.unread -= 1;
        } else {
            self.unread += 1;
        }
    }

    /// Toggle a post as read/unread.
    pub fn toggle_read(&mut self, post_id: &PostId) {
        // Get the post if it exists.
        let Some(post) = self.get_by_id_mut(post_id) else {
            return;
        };

        // Toggle the read status.
        post.read = !post.read;

        // Change the tracking unread count.
        if post.read {
            self.unread -= 1;
        } else {
            self.unread += 1;
        }
    }

    /// Get a reference to post given its ID.
    pub fn get_by_id(&self, id: &PostId) -> Option<&Post> {
        self.inner.iter().find(|p| &p.id == id)
    }

    /// Get a mutable reference to a post given its ID.
    fn get_by_id_mut(&mut self, id: &PostId) -> Option<&mut Post> {
        self.inner.iter_mut().find(|p| &p.id == id)
    }

    /// Get a reference to the inner vector.
    pub fn as_ref(&self) -> &[Post] {
        &self.inner
    }
}

/// A post identifier.
#[repr(transparent)]
#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Deserialize)]
pub struct PostId(
    #[serde(with = "arc_str_serde")]
    pub Arc<str>
);

impl From<String> for PostId {
    fn from(s: String) -> Self {
        Self(Arc::from(s))
    }
}

/// A single post in a feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    /// Identifier of the post.
    pub id: PostId,

    /// Title of this post.
    #[serde(with = "arc_str_serde")]
    pub title: Arc<str>,

    /// The URLs present in this post.
    #[serde(with = "vec_url_serde")]
    pub urls: Vec<Url>,

    /// Time when the feed was published (for RSS) or updated (for Atom).
    #[serde(with = "datetime_serde")]
    pub published: DateTime<Utc>,

    /// Whether this post has been read or not.
    pub read: bool,
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Post {}

impl PartialOrd for Post {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.published.partial_cmp(&other.published)
    }
}

impl Ord for Post {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.published.cmp(&other.published)
    }
}

/// Feed index information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FeedId {
    /// Index into `FeedConfig.sections`.
    pub section_idx: usize,

    /// Index into `Section.feeds`.
    pub feed_idx: usize,
}

impl FeedConfig {
    /// Parse a config from any buffered reader.
    pub fn parse_reader<R: BufRead>(reader: R) -> io::Result<Self> {
        // Read the sections.
        let mut sections: Vec<Section> = Vec::new();
        let mut current_section: Option<Section> = None;

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Skip empty lines.
            if line.is_empty() {
                continue;
            }

            // If the line starts with '#', it's a section
            if line.starts_with('#') {
                // Save the previous section, if any, before starting a new one.
                if let Some(section) = current_section.take() {
                    sections.push(section);
                }

                // Create a new section.
                let title = line.trim_start_matches('#').trim().to_string();
                current_section = Some(Section::new(title))
            } else if let Some(section) = &mut current_section {
                // It's a feed line in the current section.
                section.feeds.push(Feed::parse(line)?);
            }
        }

        // If there is an unfinished section, add it.
        if let Some(section) = current_section {
            sections.push(section);
        }

        Ok(Self { sections })
    }

    /// Parse the feed file.
    pub fn parse_feed_file() -> io::Result<Option<Self>> {
        let Some(feed_file) = Self::get_feed_file()? else {
            return Ok(None);
        };

        let file = std::fs::File::open(feed_file)?;
        let reader = io::BufReader::new(file);
        Ok(Some(Self::parse_reader(reader)?))
    }

    /// Get path to the config directory.
    ///
    /// If it doesn't exist, will create an empty one.
    pub fn get_config_dir() -> io::Result<PathBuf> {
        // Get a path to the config directory.
        let config_dir = match std::env::var("XDG_CONFIG_HOME") {
            Ok(dir) => PathBuf::new().join(dir),
            Err(_) => std::env::home_dir()
                .expect("Couldn't get home directory")
                .join(".config")
        };

        // Use the compile time project name as the config dir.
        let config_dir = config_dir.join(env!("CARGO_PKG_NAME"));

        // If the directory doesn't exist, create it.
        if !config_dir.exists() {
            std::fs::DirBuilder::new().recursive(true).create(&config_dir)?;
        }

        Ok(config_dir)
    }

    /// Get path to the feed file, creating the config directory if it doesn't
    /// exist yet.
    ///
    /// Returns `None` if the file doesn't exist.
    pub fn get_feed_file() -> io::Result<Option<PathBuf>> {
        // Get the config dir.
        let config_dir = Self::get_config_dir()?;
        let config_file = config_dir.join("feeds");

        // Make sure it's a file.
        config_file.metadata()
            .map(|metadata| {
                if metadata.is_file() {
                    Ok(Some(config_file))
                } else {
                    let err = format!("Path exists but isn't a file: {}",
                        config_file.display());
                    Err(io::Error::new(io::ErrorKind::Other, err))
                }
            })
            .unwrap_or(Ok(None))
    }
}

impl Section {
    /// Create a new empty section.
    fn new(title: impl Into<Arc<str>>) -> Self {
        Section {
            title: title.into(),
            feeds: Vec::new(),
        }
    }
}

impl Feed {
    /// Parse a line into a feed if it matches the expected format.
    fn parse(line: &str) -> io::Result<Self> {
        // Split on the pipe character.
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();

        // We expect `title | url`.
        if parts.len() == 2 {
            let title = parts[0].to_string().into();
            let url = Url::parse(parts[1])
                .expect("Invalid URL specified for feed");
            Ok(Feed { title, url, posts: Posts::new() })
        } else {
            Err(io::Error::new(io::ErrorKind::Other,
                "Invalid line. Expected \"<title> | <url>\""))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn parse_str(input: &str) -> io::Result<FeedConfig> {
        let cursor = Cursor::new(input);
        FeedConfig::parse_reader(cursor)
    }

    #[test]
    fn parses_single_section() {
        let cfg = r#"
# News
Rust Blog | https://blog.rust-lang.org
"#;

        let config = parse_str(cfg).unwrap();

        assert_eq!(config.sections.len(), 1);
        let section = &config.sections[0];
        assert_eq!(section.name, "News");
        assert_eq!(section.feeds.len(), 1);
    }

    #[test]
    fn parses_multiple_sections() {
        let cfg = r#"
# Tech
HN | https://news.ycombinator.com

# Comics
xkcd | https://xkcd.com
"#;

        let config = parse_str(cfg).unwrap();

        assert_eq!(config.sections.len(), 2);
        assert_eq!(config.sections[0].feeds.len(), 1);
        assert_eq!(config.sections[1].feeds.len(), 1);
    }

    #[test]
    fn ignores_lines_before_first_section() {
        let cfg = r#"
Feed | https://example.com

# Proper
Feed | https://example.com
"#;

        let config = parse_str(cfg).unwrap();

        assert_eq!(config.sections.len(), 1);
        assert_eq!(config.sections[0].feeds.len(), 1);
    }

    #[test]
    fn errors_on_invalid_feed() {
        let cfg = r#"
# Bad
not a feed
"#;

        let err = parse_str(cfg).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::Other);
    }

    #[test]
    fn empty_input_produces_no_sections() {
        let config = parse_str("").unwrap();
        assert!(config.sections.is_empty());
    }
}

mod arc_str_serde {
    use serde::{Serializer, Deserializer, Deserialize};
    use std::sync::Arc;

    pub fn serialize<S>(arc: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        serializer.serialize_str(arc.as_ref())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
    where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(Arc::from(s))
    }
}

mod vec_url_serde {
    use serde::{Serializer, Deserializer, Deserialize, Serialize};
    use url::Url;

    pub fn serialize<S>(urls: &Vec<Url>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        // Convert each Url to &str and serialize as Vec<&str>
        let strings: Vec<&str> = urls.iter().map(|u| u.as_str()).collect();
        strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Url>, D::Error>
    where
        D: Deserializer<'de>
    {
        let strings: Vec<String> = Vec::deserialize(deserializer)?;
        strings
            .into_iter()
            .map(|s| Url::parse(&s).map_err(serde::de::Error::custom))
            .collect()
    }
}

mod datetime_serde {
    use serde::{Serializer, Deserializer, Deserialize};
    use chrono::{DateTime, Utc, TimeZone};

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        // store as i64 seconds since epoch
        serializer.serialize_i64(dt.timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>
    {
        let ts = i64::deserialize(deserializer)?;
        Ok(Utc.timestamp_opt(ts, 0).unwrap())
    }
}
