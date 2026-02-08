use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use crossterm::event::{self, Event, KeyCode};
use crate::config::FeedConfig;


/// Rows in the main TUI page.
enum MainPageRow<'a> {
    /// Section title.
    Section(&'a str),

    /// Spacer between sections.
    Spacer,

    /// Feed in a section.
    Feed {
        /// Name of the feed.
        name: &'a str,
    },
}

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
    /// Rows in the main page.
    rows: Vec<MainPageRow<'a>>,
    selected_feed: usize,
    page: Page,
}

impl<'a> App<'a> {
    /// Given the `config`, builds a new `App` state.
    pub fn new(config: &'a FeedConfig) -> Self {
        // The terminal lines that will be shown on the TUI.
        let mut rows = Vec::new();

        // Go through each section.
        for section in &config.sections {
            // The first line of the section is the section title.
            rows.push(MainPageRow::Section(&section.name));

            // Push the feeds into the section.
            for feed in &section.feeds {
                rows.push(MainPageRow::Feed { name: &feed.name });
            }

            // Separate the section from other sections.
            rows.push(MainPageRow::Spacer);
        }

        Self {
            rows,
            selected_feed: 0,
            page: Page::Main,
        }
    }

    /// Runs the application loop until exit.
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) {
        loop {
            // Draw the page.
            terminal.draw(|f| match self.page {
                Page::Main => self.draw_main(f),
                Page::Feed => Self::draw_feed(f),
                _ => unreachable!(),
            }).unwrap();

            // Handle input.
            if self.handle_input() {
                break;
            }
        }
    }

    /// Get the [`MainPageRow`] indices into `self.rows` that are feeds.
    fn feed_row_indices(&self) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter_map(|(i, row)|
                matches!(row, MainPageRow::Feed { .. }).then_some(i))
            .collect()
    }

    /// Get the index of the currently selected feed.
    fn selected_feed_idx(&self) -> usize {
        self.feed_row_indices()[self.selected_feed]
    }

    /// Draw the main feeds listings page.
    fn draw_main(&self, f: &mut Frame) {
        // Generate the rows.
        let text: Vec<Line> = self.rows.iter().enumerate()
            .map(|(i, row)| match row {
                // Nice section title header.
                MainPageRow::Section(name) => Line::from(Span::styled(
                    format!("────┤ {name} ├────"),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Magenta),
                )),

                // Empty row.
                MainPageRow::Spacer => Line::from(""),

                // Feed with an indented title.
                MainPageRow::Feed { name, .. } => {
                    // Selected rows have reversed colors.
                    let style = if i == self.selected_feed_idx() {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };

                    Line::from(vec![
                        Span::raw("      "),
                        Span::styled(*name, style),
                    ])
                }
            })
            .collect();

        // Using the rows, generate the page block.
        let widget = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Feeds"));

        // Render the page.
        f.render_widget(widget, f.area());
    }

    /// Draw the post listings for a feed page.
    fn draw_feed(f: &mut Frame) {
        // Empty for now
        let widget = Paragraph::new("Detail page")
            .block(Block::default().borders(Borders::ALL).title("Detail"));

        f.render_widget(widget, f.area());
    }

    /// Handle the input for the app.
    fn handle_input(&mut self) -> bool {
        if let Event::Key(key) = event::read().unwrap() {
            match self.page {
                Page::Main => match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.selected_feed > 0 {
                            self.selected_feed -= 1;
                        }
                    }

                    KeyCode::Down | KeyCode::Char('j') => {
                        let max = self.feed_row_indices().len() - 1;
                        if self.selected_feed < max {
                            self.selected_feed += 1;
                        }
                    }

                    KeyCode::Enter | KeyCode::Char('l') => {
                        self.page = Page::Feed;
                    }

                    KeyCode::Esc | KeyCode::Char('q') => {
                        return true;
                    }

                    _ => {}
                },
                Page::Feed => match key.code {
                    KeyCode::Esc | KeyCode::Char('h') => self.page = Page::Main,
                    KeyCode::Char('q') => return true,
                    _ => {}
                },
                _ => unreachable!(),
            }
        }

        false
    }
}
