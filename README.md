# nia

`nia` is an ultra-minimal Atom/RSS feed reader that extracts and displays only
the URLs found in each post.

The reason for this is simple; I found out that many feeds do not send full post
contents, instead choosing to cut the content and redirect you to the actual
post. Likewise, most terminals can't render images/videos and so feeds that
contain non-text data end up being read in a browser anyway.

And so nia was born.

# Config

Keeping everything simple, the only thing that is configurable is the sections
and feeds! Read the [example config](example_feeds) to learn about its
structure. Your configuration should be placed into `XDG_CONFIG_HOME/nia/feeds`!

# Usage

Only vim keybinds are supported (merely out of my laziness).

On the main feed page, you can refresh the selected feed with `h` or all feeds
at once with `H` (each section is refreshed in parallel with other sections, so
the parallelization depends on the amount of your sections).

Each feed can be refreshed only once per runtime, preventing spam or accidental
refreshes. If you want to refresh a feed that has already been refreshed, just
re-run the application!
