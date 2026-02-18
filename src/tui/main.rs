use std::sync::Arc;
use ratatui::{
    prelude::*,
    widgets::ListItem,
};
use crossterm::event::KeyCode;
use crate::tui::{
    PageAction, Page, NavigableList, ListPage, feed::FeedPage, Selectable};
use crate::config::{FeedConfig, FeedId};
use crate::app::FeedState;

/// Rows in the main page.
enum MainRow {
    SectionHeader(Arc<str>),
    Feed(FeedId),
    Spacer,
}

/// Only feeds are selectable.
impl Selectable for MainRow {
    fn selectable(&self) -> bool {
        matches!(self, MainRow::Feed { .. })
    }
}

/// The main page that lists out all the feeds.
pub struct MainPage {
    list: ListPage<MainRow>,
}

impl MainPage {
    /// Create a new main page.
    pub fn new(config: &FeedConfig) -> Self {
        // Build the rows for the main page.
        let mut rows = Vec::new();

        // Go through each section.
        for (section_idx, section) in config.sections.iter().enumerate() {
            // The first line of the section is the section title.
            rows.push(MainRow::SectionHeader(section.title.clone()));

            // Push the feeds into the section.
            for (feed_idx, _feed) in section.feeds.iter().enumerate() {
                rows.push(MainRow::Feed(FeedId { section_idx, feed_idx }));
            }

            // Separate the section from other secitons.
            rows.push(MainRow::Spacer);
        }

        Self {
            list: ListPage::new(rows),
        }
    }
}

impl Page for MainPage {
    fn draw(&mut self, f: &mut Frame, state: &FeedState) {
        // Build the list items.
        let items = self.list.items.iter().map(|row| match row {
            MainRow::Spacer => {
                ListItem::new("")
            }

            MainRow::SectionHeader(title) => {
                ListItem::new(Line::styled(
                    format!("────┤ {} ├────", title),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Magenta),
                ))
            }

            MainRow::Feed(feed_id) => {
                // If the feed is being downloaded, prepend it with a spinner.
                let spinner = if state.is_downloading(&feed_id) {
                    state.spinner.frame()
                } else {
                    ' '
                };

                let feed = state.get_feed(&feed_id).unwrap();
                ListItem::new(Line::from(vec![
                    Span::raw(format!("   {}  ", spinner)),
                    Span::raw(feed.title.as_ref()),
                ]))
            }
        });

        let list = crate::tui::build_list(" Feeds ", items);
        f.render_stateful_widget(list, f.area(), &mut self.list.state);
    }

    fn list(&mut self) -> &mut dyn NavigableList {
        &mut self.list
    }

    fn on_key(&mut self, key: KeyCode, state: &FeedState) -> PageAction {
        let Some(MainRow::Feed(feed_id)) = self.list.selected_item() else {
            return PageAction::None;
        };

        match key {
            // Because the main page is the first page shown, the 'h' key will
            // be passed through to us instead of being handled in the app input
            // handler to pop the page.

            // Download the currently selected feed.
            KeyCode::Char('h') => {
                PageAction::DownloadFeed(feed_id.clone())
            },

            // Download all feeds.
            KeyCode::Char('H') => {
                PageAction::DownloadAllFeeds
            },

            // Mark all posts in the feed as read.
            KeyCode::Char('r') => {
                PageAction::MarkFeedRead(feed_id.clone())
            },

            // Check the posts listing for the selected feed.
            KeyCode::Enter | KeyCode::Char('l') => {
                // Don't do anything if the feed is empty.
                let feed = state.get_feed(feed_id).unwrap();
                if feed.posts.len() == 0 {
                    PageAction::None
                } else {
                    PageAction::NewPage(
                        Box::new(FeedPage::new(feed_id.clone())))
                }
            },

            _ => PageAction::None,
        }
    }
}
