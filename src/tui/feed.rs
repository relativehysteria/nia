use ratatui::{
    prelude::*,
    widgets::ListItem,
};
use crossterm::event::KeyCode;
use crate::tui::{PageAction, Page, NavigableList, ListPage, post::PostPage};
use crate::app::FeedState;
use crate::config::FeedId;

impl crate::tui::Selectable for usize {
    fn selectable(&self) -> bool {
        true
    }
}

/// The feed page that lists out all the posts.
pub struct FeedPage {
    /// The identifier of this feed.
    feed_id: FeedId,

    /// List of rows on the feed page.
    ///
    /// In this case, each row is a post index.
    list: ListPage<usize>,
}

impl FeedPage {
    pub fn new(feed_id: FeedId) -> Self {
        Self { feed_id, list: ListPage::new(Vec::new()), }
    }
}

impl Page for FeedPage {
    fn draw(&mut self, f: &mut Frame, state: &FeedState) {
        // Get this feed state.
        let feed = state.get_feed(&self.feed_id).unwrap();

        // Rebuild index list if lengths differ.
        if self.list.items.len() != feed.posts.len() {
            self.list = ListPage::new((0..feed.posts.len()).collect());
        }

        let items = feed.posts.iter().enumerate().map(|(idx, post)| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:>5}", idx.to_string())),
                Span::raw(post.published
                    .format("  ┊  %Y-%m-%d  │  ").to_string()),
                Span::raw(post.title.as_str()),
            ]))
        });

        let section = state.get_section(self.feed_id.section_idx).unwrap();
        let title = format!(" {} | {} ", section.title, feed.title);
        let list = crate::tui::build_list(&title, items);

        f.render_stateful_widget(list, f.area(), &mut self.list.state);
    }

    fn list(&mut self) -> &mut dyn NavigableList {
        &mut self.list
    }

    fn on_key(&mut self, key: KeyCode, _state: &FeedState) -> PageAction {
        match key {
            // Check the post page of the selected post.
            KeyCode::Enter | KeyCode::Char('l') => {
                if let Some(&selected) = self.list.selected_item() {
                    PageAction::NewPage(Box::new(
                            PostPage::new(self.feed_id.clone(), selected)))
                } else {
                    PageAction::None
                }
            }
            _ => PageAction::None,
        }
    }
}
