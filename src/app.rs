use std::time::{Instant, Duration};
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use crate::tui::{main, Page, PageAction};
use crate::config::{Feed, FeedId, FeedConfig};

/// State of the feeds.
pub struct FeedState {
    /// State of the feeds.
    pub feed_config: FeedConfig,

    /// Number of currently active downloads.
    pub active_downloads: usize,
}

impl FeedState {
    /// Create a new feed state.
    pub fn new(feed_config: FeedConfig) -> Self {
        Self { feed_config, active_downloads: 0 }
    }

    /// Get a reference to a feed.
    pub fn get_feed(&self, feed_id: FeedId) -> Option<&Feed> {
        self.feed_config.sections.get(feed_id.section_idx)
            .map(|section| section.feeds.get(feed_id.feed_idx))
            .flatten()
    }
}

/// The application state.
pub struct App {
    /// The TUI page stack.
    pages: Vec<Box<dyn Page>>,

    /// Application state.
    feed_state: FeedState,
}

impl App {
    /// Create a new application state given the `config`.
    pub fn new(feeds: FeedConfig) -> Self {
        Self {
            pages: vec![Box::new(main::MainPage::new(&feeds))],
            feed_state: FeedState::new(feeds),
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
            if self.feed_state.active_downloads > 0 {
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

                // Call the tick handler for the page if it's the right time.
                if last_tick.elapsed() >= tick_rate {
                    let now = Instant::now();
                    self.tick(now);
                    last_tick = now;
                }
            } else {
                // No active animation. We can block on input
                if self.handle_input() {
                    break;
                }
            }
        }
    }

    /// Call the tick handler for the currently shown page.
    fn tick(&mut self, now: Instant) {
        self.pages.last_mut().unwrap().tick(now)
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
        match page.on_key(key.code) {
            PageAction::None => {},
            PageAction::NewPage(p) => self.pages.push(p),
            PageAction::DownloadFeed(feed_id) => {
                self.feed_state.active_downloads += 1;
            },
            PageAction::DownloadAllFeeds => {
            },
        }

        false
    }
}
