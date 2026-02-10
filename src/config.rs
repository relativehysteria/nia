//! Config parsing and stuff.

use std::io::{self, BufRead};
use std::path::PathBuf;

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
    pub name: String,

    /// A vector of the feeds in this section.
    pub feeds: Vec<Feed>,
}

/// A feed with a title and the url of the feed.
#[derive(Debug, Clone)]
pub struct Feed {
    /// Title of this feed that will be shown in the TUI.
    pub name: String,

    /// The provided url of this feed.
    pub url: String,

    /// The posts in the feed.
    pub posts: Vec<Post>,
}

/// A single post in a feed.
#[derive(Debug, Clone)]
pub struct Post;

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
        let mut sections = Vec::new();
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
                let name = line.trim_start_matches('#').trim().to_string();
                current_section = Some(Section::new(name))
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
                .expect("Couldn't get home directory.")
                .join(".config")
        };

        // Use the compile time project name as the config dir!
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
    fn new(name: String) -> Self {
        Section {
            name,
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
            let name = parts[0].to_string();
            let url = parts[1].to_string();

            // Validate the URL.
            if url.starts_with("https://") || url.starts_with("http://") {
                Ok(Feed { name, url, posts: Vec::new() })
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Invalid URL."))
            }
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

    fn parse_str(input: &str) -> io::Result<Config> {
        let cursor = Cursor::new(input);
        Config::parse_reader(cursor)
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
