# Changelog

## v2.0.5 (2017-05-18)

*   Bugfix: don't include `://` in the list of currently open Safari tabs.
    ([#35](https://github.com/alexwlchan/safari.rs/issues/35))

## v2.0.4 (2017-05-08)

*   URL tweak: Stack Overflow URLs now use the Share links provided for SO referrals.

## v2.0.3 (2017-05-06)

*   URL tweak: discard `?highlight` and fragment from `docs.python.org` URLs
    if they're pointing at a module on a module page.

## v2.0.2 (2017-05-02)

*   Bugfix: don't panic on the `icloud-tabs` command if there's a device that doesn't have any tabs open ([#23](https://github.com/alexwlchan/safari.rs/issues/23)).

## v2.0.1 (2017-04-30)

*   Bugfix: don't include `://missing value` in the list of currently open Safari tabs.

## v2.0.0 (2017-04-30)

*   A near-complete rewrite that overhauls the command-line parsing, improves the interface, and has better error reporting.
*   Replace the `furl` and `2url` commands with a more generic `url` command with configurable `--window` and `--tab` parameters.
*   Rename the `list-tabs` command to `urls-all`.
*   Add a new `icloud-tabs` command for getting data about iCloud Tabs.
*   Strip UTM tracking parameters from all URLs.
*   Bugfix: don't panic when processing a YouTube URL that isn't a video.

## v1.2.0 (2017-02-25)

*   Add a `reading-list` command that a list of URLs from Reading List.

## v1.1.0 (2017-02-23)

*   Renamed from `safari-url` to `safari`.
*   Add a `clean-tabs` command that closes tabs whose URLs match a particular pattern.
*   Add a `list-tabs` command that gets a list of URLs in every open tab.
*   Strip tracking data from Buzzfeed and Mashable URLs.

## v1.0.0 (2017-02-20)

*   Initial release (under the name `safari-url`).
