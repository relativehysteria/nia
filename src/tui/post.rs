use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};
use crate::tui::{Page, NavigableList, ListPage};
use crate::app::FeedState;

pub struct PostPage {
    list: ListPage<String>,
}

impl PostPage {
    pub fn new() -> Self {
        // Fake for now
        let rows = vec!["post test 1".to_string(), "post test 2".to_string()];

        Self { list: ListPage::new(rows), }
    }
}

impl Page for PostPage {
    fn draw(&mut self, f: &mut Frame, _state: &FeedState) {
        let items = self.list.items.iter().map(|title| {
            ListItem::new(title.as_str())
        });

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Posts"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(list, f.area(), &mut self.list.state);
    }

    fn list(&mut self) -> &mut dyn NavigableList {
        &mut self.list
    }
}
