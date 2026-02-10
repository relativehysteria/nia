use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};
use crossterm::event::KeyCode;
use crate::tui::{PageAction, Page, NavigableList, ListPage, post::PostPage};
use crate::app::FeedState;

/// The feed page that lists out all the posts.
pub struct FeedPage {
    /// The title of this feed.
    title: String,

    /// List of rows on the feed page.
    ///
    /// In this case, each row is the title of a post.
    list: ListPage<String>,
}

impl FeedPage {
    pub fn new(title: String) -> Self {
        // Fake data for now
        let rows = vec![];

        Self { title, list: ListPage::new(rows), }
    }
}


impl Page for FeedPage {
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

    fn on_key(&mut self, key: KeyCode, state: &FeedState) -> PageAction {
        match key {
            // Check the post page of the selected post.
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(selected) = self.list.selected_item() {
                    let title = selected.clone();
                    PageAction::NewPage(Box::new(PostPage::new(title)))
                } else {
                    PageAction::None
                }
            }
            _ => PageAction::None,
        }
    }
}
