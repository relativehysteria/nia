use std::time::{Instant, Duration};
use std::collections::HashMap;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use crate::tui::{main, Page, PageAction, Spinner};
use crate::config::{Feed, FeedId, FeedConfig};
use crate::download::*;

/// The download state of this feed.
enum DownloadState {
    /// It is queued to be downloaded but is not being downloaded yet.
    Queued,

    /// Is being downloaded.
    Downloading,
}

/// State of the feeds.
pub struct FeedState {
    /// A global spinner that can be used to draw a spin animation.
    pub spinner: Spinner,

    /// State of the feeds.
    feed_config: FeedConfig,

    /// A map of feeds that are currently queued to be downloaded.
    downloading: HashMap<FeedId, DownloadState>,
}

impl FeedState {
    /// Create a new feed state.
    pub fn new(feed_config: FeedConfig) -> Self {
        Self {
            feed_config,
            downloading: HashMap::new(),
            spinner: Spinner::new(),
        }
    }

    /// Check whether the `feed_id` is being currently downloaded.
    pub fn is_downloading(&self, feed_id: &FeedId) -> bool {
        self.downloading.get(&feed_id)
            .map(|state| matches!(state, DownloadState::Downloading))
            .unwrap_or(false)
    }

    /// Get a reference to a feed.
    pub fn get_feed(&self, feed_id: &FeedId) -> Option<&Feed> {
        self.feed_config.sections.get(feed_id.section_idx)
            .map(|section| section.feeds.get(feed_id.feed_idx))
            .flatten()
    }

    /// Get a mutable reference to a feed.
    pub fn get_feed_mut(&mut self, feed_id: &FeedId) -> Option<&mut Feed> {
        self.feed_config.sections.get_mut(feed_id.section_idx)
            .map(|section| section.feeds.get_mut(feed_id.feed_idx))
            .flatten()
    }
}

/// The application state.
pub struct App {
    /// The TUI page stack.
    pages: Vec<Box<dyn Page>>,

    /// Application state.
    feed_state: FeedState,

    /// State of the background feed downloader.
    download: DownloadChannel,
}

impl App {
    /// Create a new application state given the `config`.
    pub fn new(feeds: FeedConfig) -> Self {
        Self {
            pages: vec![Box::new(main::MainPage::new(&feeds))],
            feed_state: FeedState::new(feeds),
            download: DownloadChannel::spawn_downloader_thread(),
        }
    }

    /// Go back from the currently shown page to the one before.
    fn go_back(&mut self) {
        if self.pages.len() > 1 {
            self.pages.pop();
        }
    }

    /// Draw the page.
    fn draw(&mut self, f: &mut Frame) {
        self.pages.last_mut().unwrap().draw(f, &self.feed_state);
    }

    /// Start downloading a single feed.
    fn start_download(&mut self, feed: FeedId) {
        // Immediately mark the feed as being downloaded instead of waiting for
        // the download task to tell us that the download has started.
        // We do this so the `App::run()` loop can start ticking immediately.
        self.feed_state.downloading.insert(feed.clone(), DownloadState::Queued);


        // Send the request to the downloader.
        let url = self.feed_state.get_feed(&feed).unwrap().url.clone();
        self.download
            .request_tx
            .send(DownloadRequest::DownloadFeed { feed, url })
            .expect("The downloader has closed abruptly.");
    }

    /// Download all feeds.
    ///
    /// One downloader is spawned for each section.
    fn download_all(&mut self) {
        // Build the URL map for the request.
        let url_map = UrlMap::from(&self.feed_state.feed_config);

        // Queue up all feeds.
        for (section_idx, section) in url_map.0.iter().enumerate() {
            for (feed_idx, _) in section.iter().enumerate() {
                let feed = FeedId { section_idx, feed_idx };
                self.feed_state.downloading.insert(feed, DownloadState::Queued);
            }
        }

        // Send the request to the downloader.
        self.download
            .request_tx
            .send(DownloadRequest::DownloadAll(url_map))
            .expect("The downloader has closed abruptly.");
    }

    /// Handle events from the background downloader _in a non-blocking manner_.
    fn handle_download_events(&mut self) {
        for response in self.download.response_rx.try_iter() {
            match response {
                DownloadResponse::Started(feed) => {
                    self.feed_state.downloading.insert(
                        feed, DownloadState::Downloading);
                },
                DownloadResponse::Finished(feed) => {
                    self.feed_state.downloading.remove(&feed);
                },
            }
        }
    }

    /// Run the application.
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) {
        // Set the tick rate for animations.
        let fps = 60;
        let tick_rate = Duration::from_millis(1000 / fps);
        let mut last_tick = Instant::now();

        loop {
            // Draw the page.
            terminal.draw(|f| self.draw(f)).unwrap();

            // If there's an active download, we have to do ticks because of
            // animations and polls and stuff.
            if !self.feed_state.downloading.is_empty() {
                // Handle events from the background downloader.
                self.handle_download_events();

                // Our input handler _blocks_, so we will poll for events on a
                // timeout and only call the handler when we get an event.
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::ZERO);

                if event::poll(timeout).unwrap() {
                    if self.handle_input() {
                        break;
                    }
                }

                // Animate the global spinner.
                if last_tick.elapsed() >= tick_rate {
                    let now = Instant::now();
                    self.feed_state.spinner.tick(now);
                    last_tick = now;
                }
            } else {
                // No active download. We can block on input
                if self.handle_input() {
                    break;
                }
            }
        }
    }

    /// Handle the input for the app in a blocking manner.
    fn handle_input(&mut self) -> bool {
        // Get the key.
        let Event::Key(key) = event::read().unwrap() else {
            return false;
        };

        // Global escape: pop page if possible. If we're on the first page, we
        // allow this event to reach it, otherwise we use it to pop the current
        // page.
        if self.pages.len() > 1 {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('h')) {
                self.go_back();
                return false;
            }
        }

        // Shared list navigation hook for all pages. If we handle the input
        // here, it won't be passed to the page specific handler.
        let page = self.pages.last_mut().unwrap();
        let mut input_handled = true;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => page.list().up(1),
            KeyCode::Down | KeyCode::Char('j') => page.list().down(1),
            KeyCode::PageUp | KeyCode::Char('K') => page.list().up(10),
            KeyCode::PageDown | KeyCode::Char('J') => page.list().down(10),
            KeyCode::Char('q') => return true,
            _ => input_handled = false,
        }

        // If we have handled the input above, there's nothing else to do.
        if input_handled {
            return false;
        }

        // We haven't handled the input above. The page might wanna handle it
        // instead.
        match page.on_key(key.code, &self.feed_state) {
            PageAction::None                  => {},
            PageAction::NewPage(p)            => self.pages.push(p),
            PageAction::DownloadFeed(feed_id) => self.start_download(feed_id),
            PageAction::DownloadAllFeeds      => self.download_all(),
        }

        false
    }
}
