# Changelog

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
