# safari-url

safari-url provides some tools for interacting with Safari.

## Commands

1.  Get the URL of the frontmost tab:

    ```console
    $ safari-url furl
    http://aromantic.wikia.com/wiki/Alterous⏎
    ```

2.  Get the URL of the front tab in window 2:

    ```console
    $ safari-url 2url
    http://stackoverflow.com/questions/39775060/reverse-iterating-over-a-vec-versus-vec-iter#39775142⏎
    ```

    This is useful if you have two Safari windows open, and you want to
    copy the URL from one window into a form in another.

3.  Close tabs containing certain URLs:

    ```console
    $ safari-url clean-tabs https://www.youtube.com,https://twitter.com
    ```

    I find this useful for quickly cutting down my open tabs.

4.  List the tabs that are open in every window:

    ```console
    $ safari-url list-tabs
    https://github.com/alexwlchan/ao3
    https://www.susanjfowler.com/blog/2017/2/19/reflecting-on-one-very-strange-year-at-uber
    https://github.com/Keats/tera
    https://crates.io/crates/tera
    ```

### Installation

You need [Rust installed][rust].  Then clone the repository and run
`cargo install`:

```console
$ cargo install --git https://github.com/alexwlchan/safari-url
```

I originally developed this against Rust 1.15.0.

[rust]: https://www.rust-lang.org/en-US/install.html

### URL transformations

The `furl` and `2url` tabs will do a bit of cleaning before they return
the URL:

*   Twitter links to the mobile site (`mobile.twitter.com`) are flipped to
    point to the desktop site (`twitter.com`).
*   Tracking data is partially stripped from Amazon, Buzzfeed, Mashable and
    Medium URLs.
*   The `#notes` fragment is removed from Tumblr URLs.
*   Everything except the `?v=` query parameter is removed from YouTube URLs.

### Motivation

I first got the idea for a script to access Safari URLs [from Dr. Drang][dr].
I've been through several different versions – AppleScript, shell, Python –
gradually adding the cleaning features – and now I've written a new
version in Rust.

Why Rust?

*   It's really fast.  The Rust script returns immediately – with some of
    the other versions, I had a noticeable delay when typing `;furl`.
*   I like Rust, and I’ve been enjoying playing with it recently.

[dr]: http://www.leancrew.com/all-this/2009/07/safari-tab-urls-via-textexpander/

### License

MIT license.
