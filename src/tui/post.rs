use ratatui::{
    prelude::*,
    widgets::ListItem,
};
use crate::tui::{Page, NavigableList, ListPage};
use crate::app::FeedState;
use crate::config::FeedId;

impl crate::tui::Selectable for url::Url {
    fn selectable(&self) -> bool {
        true
    }
}

/// The post page that lists out all URLs in a post.
pub struct PostPage {
    /// The identifier of this post's feed.
    feed_id: FeedId,

    /// The index of this post within the feed.
    post_idx: usize,

    /// List of rows on the post page.
    ///
    /// In this case, each row is a URL in this post.
    list: ListPage<url::Url>,
}

impl PostPage {
    pub fn new(feed_id: FeedId, post_idx: usize) -> Self {
        Self { feed_id, post_idx, list: ListPage::new(Vec::new()) }
    }
}

impl Page for PostPage {
    fn draw(&mut self, f: &mut Frame, state: &FeedState) {
        // Get this post state.
        let feed = state.get_feed(&self.feed_id).unwrap();
        let post = &feed.posts[self.post_idx];

        // Rebuild the URL list if the lengths differ.
        if self.list.items.len() != post.urls.len() {
            self.list = ListPage::new(post.urls.clone());
        }

        let items = post.urls.iter().enumerate().map(|(idx, url)| {
            ListItem::new(Line::from(vec![
                Span::raw(format!("{:>3}  │  ", idx)),
                Span::raw(url.to_string()),
            ]))
        });

        let section = &state.get_section(self.feed_id.section_idx).unwrap().name;
        let title = format!(" {} | {} | {} ", section, feed.name, &post.name);
        let list = crate::tui::build_list(&title, items);

        f.render_stateful_widget(list, f.area(), &mut self.list.state);
    }

    fn list(&mut self) -> &mut dyn NavigableList {
        &mut self.list
    }
}
