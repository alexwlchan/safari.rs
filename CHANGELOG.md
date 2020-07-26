# Changelog

## v2.3.0 (2020-07-26)

*   Add a new command `title` to get the title of a Safari window.

## v2.2.9 (2020-07-26)

*   Internal refactoring to remove special-case handling for the old Wellcome Images site (`wellcomeimages.org`).
    Since that site was shut down over two years ago, the code to handle it can be removed without a user-facing effect.

## v2.2.8 (2020-02-09)

*   URL tweak: strip the `ref=` and `sort=` parameters from Redbubble URLs.

## v2.2.7 (2020-01-12)

*   URL tweak: strip the `m=` parameter from Blogspot URLs (so links are never mobile links).

## v2.2.6 (2019-12-29)

*   URL tweak: strip the `app=` parameter from YouTube URLs.

## v2.2.5 (2019-12-28)

*   URL tweak: strip more tracking parameters from Etsy URLs.

## v2.2.4 (2019-04-21)

*   URL tweak: strip tracking parameters from Etsy URLs.

## v2.2.3 (2018-05-22)

*   URL tweak: links to the files tab of GitHub pull requests are now replaced
    by links to the top of the pull request.

## v2.2.2 (2017-10-07)

*   URL tweak: mobile.nytimes.com links are now replaced by non-mobile versions.
    ([#59](https://github.com/alexwlchan/safari.rs/issues/59))

## v2.2.1 (2017-08-29)

*   URL tweak: Remove the `_ga` tracking parameter from shared URLs.
    ([#56](https://github.com/alexwlchan/safari.rs/issues/56))

## v2.2.0 (2017-06-10)

*   New command: the `resolve` command can take a URL as an argument, and print to stdout the final location of that URL after following any redirects.
    Useful for working with, e.g., `t.co` or `bit.ly` URLs.

## v2.1.2 (2017-06-10)

*   URL tweak: remove tracking parameters from URLs on `telegraph.co.uk`.
    ([#48](https://github.com/alexwlchan/safari.rs/issues/48))

## v2.1.1 (2017-06-05)

*   URL tweak: most of `wellcomeimages.org` is loaded entirely in `<iframe>` tags, and the page URL doesn't change.
    This isn't very useful, so now the tool will attempt to guess a sensible permalink on image pages, based on the HTML.

## v2.1.0 (2017-06-03)

*   Deprecation: the `urls-all` command has been replaced by `list-tabs`.
    This reverts the change in v2.0, because I kept trying to use the old command, and realised I actually prefer the old command.
    ([#41](https://github.com/alexwlchan/safari.rs/pull/41))
*   New command: the `tidy-url` command can take a URL as an argument, and
    pass it through the URL cleaning pipeline.
    ([#43](https://github.com/alexwlchan/safari.rs/pull/43))

## v2.0.7 (2017-06-02)

*   URL tweak: all stackexchange.com URLs now use the referral links.

## v2.0.6 (2017-05-29)

*   URL tweak: scifi.stackexchange.com URLs now uses the Share links provided for SFF.SE referrals.

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
