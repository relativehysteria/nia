use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> io::Result<()> {
    // Parse the feeds
    let feeds = nia::config::FeedConfig::parse_feed_file()
        .expect("Couldn't parse the feed file.");
    let Some(feeds) = feeds else {
        println!("No feeds!");
        return Ok(());
    };

    // Set up the terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app!
    nia::tui::App::new(feeds).run(&mut terminal);

    // Restore the terminal.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
