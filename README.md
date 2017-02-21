# safari-url

safari-url is a tool for providing quick access to URLs in Safari.
It also  does some cleaning of the URL -- removing mobile URLs,
tracking junk, and so on.

```console
$ safari-url furl
https://github.com/alexwlchan/safari-url⏎
```

I use this in Keyboard Maestro and shell scripts for getting information
from my web browser.

### Installation

You need [Rust installed][rust].  Then clone the repository and run
`cargo install`:

```console
$ cargo install --git https://github.com/alexwlchan/safari-url
```

I originally developed this against Rust 1.15.0.

[rust]: https://www.rust-lang.org/en-US/install.html

### URL transformations

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
