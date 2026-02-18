pub mod main;
pub mod feed;
pub mod post;

use std::time::{Duration, Instant};
use ratatui::{
    prelude::*,
    widgets::{ListState, ListItem, List, Block, Borders}
};
use crossterm::event::KeyCode;
use crate::app::FeedState;
use crate::config::FeedId;
use crate::database::DatabaseChannel;

/// Trait which must be implemented for all entries in a navigable list that are
/// selectable.
pub trait Selectable {
    /// Returns whether this entry can be selected or not.
    fn selectable(&self) -> bool;
}

/// Implementation of a single page in the TUI.
pub trait Page {
    /// Draw this page in the TUI.
    fn draw(&mut self, f: &mut Frame, state: &FeedState);

    /// Called after list navigation keys are handled.
    #[allow(unused_variables)]
    fn on_key(&mut self, key: KeyCode, state: &FeedState) -> PageAction {
        PageAction::None
    }

    /// Access to the list for shared navigation.
    fn list(&mut self) -> &mut dyn NavigableList;

    /// A hook that is executed by the app when the page is created and pushed
    /// to the page stack.
    #[allow(unused_variables)]
    fn on_new(&mut self, state: &mut FeedState, database: &DatabaseChannel) {}
}

/// Navigation controls for selectable lists.
pub trait NavigableList {
    /// Select the entry `amount` above the currently selected one.
    fn up(&mut self, amount: usize);

    /// Select the entry `amount` below the currently selected one.
    fn down(&mut self, amount: usize);
}

/// Strings in lists are always selectable.
impl Selectable for String {
    fn selectable(&self) -> bool {
        true
    }
}

/// Page actions that might be returned from the page specific input handler.
pub enum PageAction {
    /// No action.
    None,

    /// Go to a new page.
    NewPage(Box<dyn Page>),

    /// Download a feed.
    DownloadFeed(FeedId),

    /// Download all feeds.
    DownloadAllFeeds,
}

/// A page that lists out selectable `T` elements.
pub struct ListPage<T> {
    /// All items in the list.
    items: Vec<T>,

    /// Indices of the items which are selectable.
    selectable: Vec<usize>,

    /// Index into `selectable`.
    selected: usize,

    /// The state of the ratatui list.
    state: ListState,
}

impl<T: Selectable> ListPage<T> {
    /// Create a new listings page.
    pub fn new(items: Vec<T>) -> Self {
        let selectable: Vec<_> = items.iter().enumerate()
            .filter_map(|(i, item)| item.selectable().then_some(i))
            .collect();

        let mut state = ListState::default();
        state.select(selectable.get(0).copied());

        Self { items, state, selectable, selected: 0 }
    }

    /// Get a reference to the currently selected item.
    pub fn selected_item(&self) -> Option<&T> {
        self.selectable.get(self.selected).and_then(|&idx| self.items.get(idx))
    }

    /// Map `selected` into `state`.
    pub fn update_state(&mut self) {
        self.state.select(self.selectable.get(self.selected).copied())
    }
}

impl<T: Selectable> NavigableList for ListPage<T> {
    fn up(&mut self, amount: usize) {
        self.selected = self.selected.saturating_sub(amount);
        self.update_state();
    }

    /// Move down by `amount` selectable entries.
    fn down(&mut self, amount: usize) {
        let max = self.selectable.len().saturating_sub(1);
        self.selected = max.min(self.selected.saturating_add(amount));
        self.update_state();
    }
}

/// Animated spinner that can be used to show that something is being loaded.
pub struct Spinner {
    /// The current frame of the spinner.
    frame_idx: usize,

    /// The time when the current frame has been shown.
    last_tick: Instant,
}

impl Spinner {
    /// Frames of the spinner which will be shown on the screen when a feed is
    /// being actively downloaded.
    const UNICODE_SPINNER: &[char] = &[
        '⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'
    ];

    /// The time for which a single frame of the spinner will be shown.
    const SPINNER_FRAME_TIME: Duration = Duration::from_millis(120);

    /// Create a new animated spinner.
    pub fn new() -> Self {
        Self {
            frame_idx: 0,
            last_tick: Instant::now(),
        }
    }

    /// Tick the animated spinner.
    pub fn tick(&mut self, now: Instant) {
        if now.duration_since(self.last_tick) >= Self::SPINNER_FRAME_TIME {
            self.frame_idx = (self.frame_idx + 1) % Self::UNICODE_SPINNER.len();
            self.last_tick = now;
        }
    }

    /// Returns the current frame of the animation.
    pub fn frame(&self) -> char {
        Self::UNICODE_SPINNER[self.frame_idx]
    }

    /// Reset the spinner to the first frame.
    pub fn reset(&mut self) {
        self.frame_idx = 0;
        self.last_tick = Instant::now();
    }
}

/// Helper function to build the page list.
fn build_list<'a, T>(title: &'a str, items: T) -> List<'a>
where
    T: IntoIterator,
    <T as IntoIterator>::Item: Into<ListItem<'a>>
{
    List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::ITALIC)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol(" ")
        .scroll_padding(4)
}
