use nia::config::FeedConfig;

fn main() {
    let feeds = FeedConfig::parse_feed_file()
        .expect("Couldn't parse feed file");

    if let Some(feeds) = feeds {
        for section in feeds.sections.iter() {
            println!("{}", section.name);
            for feed in section.feeds.iter() {
                println!(" > {} | {}", feed.name, feed.url);
            }
        }
    }
}
