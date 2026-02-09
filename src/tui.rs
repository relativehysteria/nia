pub mod main;
pub mod feed;
pub mod post;

use std::time::{Duration, Instant};
use ratatui::{prelude::*, widgets::ListState};
use crossterm::event::{self, Event, KeyCode};
use crate::config::FeedConfig;


/// Trait which must be implemented for all entries in a navigable list that are
/// selectable.
pub trait Selectable {
    /// Returns whether this entry can be selected or not.
    fn selectable(&self) -> bool;
}

/// Implementation of a single page in the TUI.
pub trait Page {
    /// Draw this page in the TUI.
    fn draw(&mut self, f: &mut Frame);

    /// Access to the list for shared navigation.
    fn list(&mut self) -> &mut dyn NavigableList;

    /// Called after list navigation keys are handled.
    #[allow(unused_variables)]
    fn on_key(&mut self, key: KeyCode) -> PageAction {
        PageAction::None
    }

    /// Handle a frame tick.
    #[allow(unused_variables)]
    fn tick(&mut self, now: Instant) {}

    /// Return whether this page has an active animation and the `tick()`
    /// handler should be called.
    fn has_active_animation(&self) -> bool {
        false
    }
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
    None,
    Push(Box<dyn Page>),
}

/// A page that lists out selectable `T` elements.
pub struct ListPage<T> {
    /// All items in the list.
    items: Vec<T>,

    /// Indices of the items which are selectable.
    selectable: Vec<usize>,

    /// Index into `selectable`.
    selected: usize,

    /// The state of the list.
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
    pub fn current(&self) -> char {
        Self::UNICODE_SPINNER[self.frame_idx]
    }

    /// Reset the spinner to the first frame.
    pub fn reset(&mut self) {
        self.frame_idx = 0;
        self.last_tick = Instant::now();
    }
}

/// The TUI application state.
pub struct App {
    /// The page stack.
    pages: Vec<Box<dyn Page>>,
}

impl App {
    /// Create a new application state given the `config`.
    pub fn new(config: FeedConfig) -> Self {
        Self {
            pages: vec![Box::new(main::MainPage::new(config))],
        }
    }

    /// Run the application.
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) {
        // Set the tick rate for animations.
        let fps = 60;
        let tick_rate = Duration::from_millis(1000 / fps);
        let mut last_tick = Instant::now();

        loop {
            // Draw the page.
            terminal.draw(|f| self.draw(f)).unwrap();

            // If there's an active animation, we have to do ticks.
            if self.has_active_animation() {
                // Our input handler _blocks_, so we will poll for events on a
                // timeout and only call the handler when we get an event.
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::ZERO);

                if event::poll(timeout).unwrap() {
                    if self.handle_input() {
                        break;
                    }
                }

                // Call the tick handler for the page if it's the right time.
                if last_tick.elapsed() >= tick_rate {
                    let now = Instant::now();
                    self.tick(now);
                    last_tick = now;
                }
            } else {
                // No active animation. We can block on input
                if self.handle_input() {
                    break;
                }
            }
        }
    }

    /// Ask the current page whether it has an active animation and should be
    /// ticked.
    fn has_active_animation(&self) -> bool {
        self.current_page_ref().has_active_animation()
    }

    /// Call the tick handler for the currently shown page.
    fn tick(&mut self, now: Instant) {
        self.current_page().tick(now)
    }

    /// Get the currently shown page.
    fn current_page(&mut self) -> &mut Box<dyn Page> {
        self.pages.last_mut().unwrap()
    }

    /// Get a reference to the currently shown page.
    fn current_page_ref(&self) -> &Box<dyn Page> {
        self.pages.last().unwrap()
    }

    /// Go back from the currently shown page to the one before.
    fn go_back(&mut self) {
        if self.pages.len() > 1 {
            self.pages.pop();
        }
    }

    /// Draw the page.
    fn draw(&mut self, f: &mut Frame) {
        self.current_page().draw(f)
    }

    /// Handle the input for the app in a blocking manner.
    fn handle_input(&mut self) -> bool {
        // Get the key.
        let Event::Key(key) = event::read().unwrap() else {
            return false;
        };

        // Global escape: pop page if possible. If we're on the first page, we
        // allow this event to reach it, otherwise we use it to pop the current
        // page.
        if self.pages.len() > 1 {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('h')) {
                self.go_back();
                return false;
            }
        }

        // Shared list navigation hook for all pages. If we handle the input
        // here, it won't be passed to the page specific handler.
        let page = self.current_page();
        let mut input_handled = true;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => page.list().up(1),
            KeyCode::Down | KeyCode::Char('j') => page.list().down(1),
            KeyCode::PageUp | KeyCode::Char('K') => page.list().up(10),
            KeyCode::PageDown | KeyCode::Char('J') => page.list().down(10),
            KeyCode::Char('q') => return true,
            _ => input_handled = false,
        }

        // If we have handled the input above, there's nothing else to do.
        if input_handled {
            return false;
        }

        // We haven't handled the input above. The page might wanna handle it
        // instead.
        match page.on_key(key.code) {
            PageAction::None => {},
            PageAction::Push(p) => self.pages.push(p),
        }

        false
    }
}
