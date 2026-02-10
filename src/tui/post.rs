use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};
use crate::tui::{Page, NavigableList, ListPage};
use crate::app::FeedState;

/// The post page that lists out all URLs in a post.
pub struct PostPage {
    /// The title of this post.
    title: String,

    /// List of rows on the post page.
    ///
    /// In this case, each row is a URL in this post.
    list: ListPage<String>,
}

impl PostPage {
    pub fn new(title: String) -> Self {
        // Fake for now
        let rows = vec!["post test 1".to_string(), "post test 2".to_string()];

        Self { title, list: ListPage::new(rows), }
    }
}

impl Page for PostPage {
    fn draw(&mut self, f: &mut Frame, _state: &FeedState) {
        let items = self.list.items.iter().map(|title| {
            ListItem::new(title.as_str())
        });

        let title = self.title.as_str();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_stateful_widget(list, f.area(), &mut self.list.state);
    }

    fn list(&mut self) -> &mut dyn NavigableList {
        &mut self.list
    }
}
