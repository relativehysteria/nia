use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};
use crossterm::event::KeyCode;
use crate::tui::{PageAction, Page, NavigableList, ListPage, post::PostPage};

/// The feed page that lists out all the posts.
pub struct FeedPage {
    list: ListPage<String>,
}

impl FeedPage {
    pub fn new(feed_name: String) -> Self {
        // Fake data for now
        let rows = vec!["test 1".to_string(), "test 2".to_string()];

        Self { list: ListPage::new(rows), }
    }
}


impl Page for FeedPage {
    fn draw(&mut self, f: &mut Frame) {
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

    fn on_key(&mut self, key: KeyCode) -> PageAction {
        match key {
            // Check the post page of the selected post.
            KeyCode::Enter | KeyCode::Char('l') =>
                PageAction::Push(Box::new(PostPage::new())),
            _ => PageAction::None,
        }
    }
}
