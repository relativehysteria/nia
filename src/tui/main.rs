use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};
use crossterm::event::KeyCode;
use crate::tui::{
    PageAction, Page, NavigableList, ListPage, feed::FeedPage, Selectable};
use crate::config::FeedConfig;


/// Rows in the main page.
enum MainRow {
    SectionHeader(String),
    Feed(String),
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
    pub fn new(config: FeedConfig) -> Self {
        // Build the rows for the main page.
        let mut rows = Vec::new();

        // Go through each section.
        for section in config.sections.iter() {
            // The first line of the section is the section title.
            rows.push(MainRow::SectionHeader(section.name.clone()));

            // Push the feeds into the section.
            for feed in section.feeds.iter() {
                rows.push(MainRow::Feed(feed.name.clone()));
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
    fn draw(&mut self, f: &mut Frame) {
        // Build the list items.
        let items = self.list.items.iter().map(|row| match row {
            MainRow::SectionHeader(name) => {
                ListItem::new(Line::styled(
                    format!("────┤ {} ├────", name),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Magenta),
                ))
            }

            MainRow::Feed(name) => {
                ListItem::new(Line::from(vec![
                    Span::raw("      "),
                    Span::raw(name.clone()),
                ]))
            }

            MainRow::Spacer => {
                ListItem::new("")
            }
        });

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Feeds"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("")
            .scroll_padding(4);

        f.render_stateful_widget(list, f.area(), &mut self.list.state);
    }

    fn list(&mut self) -> &mut dyn NavigableList {
        &mut self.list
    }

    fn on_key(&mut self, key: KeyCode) -> PageAction {
        match key {
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(MainRow::Feed(name)) = self.list.selected_item() {
                    PageAction::Push(Box::new(FeedPage::new(name.clone())))
                } else {
                    PageAction::None
                }
            }
            _ => PageAction::None,
        }
    }
}
