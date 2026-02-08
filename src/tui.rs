use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use crossterm::event::{self, Event, KeyCode};
use crate::config::FeedConfig;

/// Pages in the TUI.
enum Page {
    /// The main page that lists out the sections and the feeds.
    Main,

    /// The feed page that lists out the posts in a feed.
    Feed,

    /// The post page that lists out the details about the post.
    Post,
}

/// TUI application state.
pub struct App<'a> {
    /// Rows in the main page built from the feed config.
    rows: Vec<ListItem<'a>>,

    /// List state over `rows`
    list_state: ListState,

    /// Indices of feeds in `rows`.
    feed_row_indices: Vec<usize>,

    /// Index of the currently selected feed.
    selected_feed: usize,

    /// The currently active page.
    page: Page,
}

impl<'a> App<'a> {
    /// Given the `config`, builds a new `App` state.
    pub fn new(config: &'a FeedConfig) -> Self {
        // The terminal lines that will be shown on the TUI.
        let mut rows = Vec::new();
        let mut feed_row_indices = Vec::new();

        // Go through each section.
        for section in &config.sections {
            // The first line of the section is the section title.
            rows.push(ListItem::new(Line::styled(
                format!("────┤ {} ├────", section.name),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Magenta),
            )));

            // Push the feeds into the section.
            for feed in &section.feeds {
                // Save the index to this feed.
                feed_row_indices.push(rows.len());

                // Save this feed.
                rows.push(ListItem::new(Line::from(vec![
                    Span::raw("      "),
                    Span::raw(feed.name.clone()),
                ])));
            }

            // Separate the section from other sections.
            rows.push(ListItem::new(""));
        }

        let mut list_state = ListState::default();
        list_state.select(feed_row_indices.get(0).copied());

        Self {
            rows,
            feed_row_indices,
            list_state,
            selected_feed: 0,
            page: Page::Main,
        }
    }

    /// Runs the application loop until exit.
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) {
        // If we don't get input, the viewport doesn't change.
        // If we do, it's dirty and we should redraw it.
        let mut dirty = true;

        loop {
            // Draw the page.
            if dirty {
                terminal.draw(|f| self.draw(f)).unwrap();
            }

            // Handle input.
            if self.handle_input(&mut dirty) {
                break;
            }
        }
    }

    /// Get the index of the currently selected feed.
    fn selected_feed_idx(&self) -> usize {
        self.feed_row_indices[self.selected_feed]
    }

    /// Draw the TUI.
    fn draw(&mut self, f: &mut Frame) {
        match self.page {
            Page::Main => self.draw_main(f),
            Page::Feed => self.draw_feed(f),
            Page::Post => unreachable!(),
        }
    }

    /// Draw the main feeds listings page.
    fn draw_main(&mut self, f: &mut Frame) {
        // Map the selection to the actual row index.
        self.list_state.select(Some(self.selected_feed_idx()));

        let list = List::new(self.rows.clone())
            .block(Block::default().borders(Borders::ALL).title("Feeds"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("")
            .scroll_padding(4);

        f.render_stateful_widget(list, f.area(), &mut self.list_state);
    }

    /// Draw the post listings for a feed page.
    fn draw_feed(&self, f: &mut Frame) {
        // Empty for now
        let widget = Block::default().borders(Borders::ALL).title("Detail");
        f.render_widget(widget, f.area());
    }

    /// Handle the input for the app.
    ///
    /// If we get input, `dirty` is set to true.
    fn handle_input(&mut self, dirty: &mut bool) -> bool {
        let Event::Key(key) = event::read().unwrap() else {
            return false;
        };

        // We got input, set the dirty bool.
        *dirty = true;

        match self.page {
            Page::Main => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.selected_feed = self.selected_feed
                        .saturating_sub(1);
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    let max = self.feed_row_indices.len() - 1;
                    self.selected_feed = max.min(self.selected_feed + 1);
                }

                KeyCode::Enter | KeyCode::Char('l') => {
                    self.page = Page::Feed;
                }

                KeyCode::Esc | KeyCode::Char('q') => {
                    return true;
                }

                KeyCode::PageUp | KeyCode::Char('K') => {
                    self.selected_feed = self.selected_feed
                        .saturating_sub(10);
                },

                KeyCode::PageDown | KeyCode::Char('J') => {
                    let max = self.feed_row_indices.len() - 1;
                    self.selected_feed = max.min(self.selected_feed + 10);
                }

                _ => {
                    // Unhandled input doesn't dirty the viewport.
                    *dirty = false;
                }
            },

            Page::Feed => match key.code {
                KeyCode::Esc | KeyCode::Char('h') => {
                    self.page = Page::Main;
                }

                KeyCode::Char('q') => {
                    return true;
                }

                _ => {
                    // Unhandled input doesn't dirty the viewport.
                    *dirty = false;
                }
            },
            _ => unreachable!(),
        }

        false
    }
}
