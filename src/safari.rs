use std::env;
use std::fs::File;
use std::io::Write;
use std::process;

use applescript::{run as run_applescript};

use plist::Plist;

use tera::{Context, Tera};

use urls;


// http://stackoverflow.com/a/27590832/1558022
macro_rules! error(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
        process::exit(1);
    } }
);


/// Returns true/false if Safari is running.
pub fn is_safari_running() -> bool {
  let cmd_result = Command::new("ps")
    .arg("-eaf")
    .output()
    .expect("Unable to test if Safari is running.");
  for line in String::from_utf8(cmd_result.stdout).unwrap().lines() {
    if line.ends_with("Safari.app/Contents/MacOS/Safari") {
      return true
    }
  }
  false
}


/// Exits the program if Safari isn't running.
pub fn assert_safari_is_running() {
  if !is_safari_running() {
    error!("Safari is not running.");
  }
}


/// Return a URL from a Safari window.
///
/// Given a (window, tab) pair, this function looks up the URL of the tab.
/// Note that it doesn't do any error handling, so will throw an execution
/// error if it fails.
///
/// * `window` - Window index.  1 is frontmost.  If None, assumes the
///              frontmost window.
/// * `tab` - Tab index.  1 is leftmost.  If None, assumes the frontmost tab.
///
pub fn get_url(window: Option<i32>, tab: Option<i32>) -> String {
  // If a tab isn't specified, assume the user wants the frontmost tab.
  let command = match window {
    Some(w_idx) => {
      match tab {
        Some(t_idx) => format!("tell application \"Safari\" to get tab {} of window {}", t_idx, w_idx),
        None => format!("tell application \"Safari\" to get URL of document {}", w_idx),
      }
    },
    None => "tell application \"Safari\" to get URL of document 1".to_string(),
  };
  let output = run_applescript(&command);

  if output.status.success() {
    urls::tidy_url(output.stdout.trim())
  } else {
    if output.stderr.contains("Invalid index") {
      error!("Invalid index: no such window or tab.");
    } else {
      error!("Unexpected error from osascript: {:?}", output.stderr);
    }
  }
}


/// Return a list of URLs from Safari.
///
/// This returns a list of URLs, one for every tab in Safari.  Iteration
/// order depends on AppleScript, which I don't think is guaranteed to be
/// stable (in particular, I think it depends on which window is frontmost).
///
pub fn get_all_urls() -> Vec<String> {
  let script = include_str!("scripts/list-open-tabs.scpt");
  let output = run_applescript(&script);
  if output.status.success() {
    output.stdout.trim()
                 .split(", ")
                 .map(|url| urls::tidy_url(url))
                 .filter(|url| url != "favorites://")
                 .collect()
  } else {
    error!("Unexpected error from osascript: {:?}", output.stderr);
  }
}


/// Convert URL patterns into AppleScript conditions.
///
/// These conditions can be used in an `if` statement in AppleScript to
/// decide whether a tab should be closed.  Three patterns are supported,
/// a limited regex syntax:
///
///     example.com             matches anywhere in the URL
///     ^http://examples.com    matches at the start of the URL
///     example.com/$           matches at the end of the URL
///
fn parse_conditions(url_patterns: Vec<&str>) -> Vec<String> {
  url_patterns.iter().map(|p|
    if p.starts_with("^") {
      format!("starts with \"{}\"", p.replace("^", ""))
    } else if p.ends_with("$") {
      format!("ends with \"{}\"", p.replace("$", ""))
    } else {
      format!("contains \"{}\"", p)
    }
  ).collect()
}


/// Tests for parse_conditions().
#[cfg(test)]
mod tests {
  use safari::parse_conditions;

  #[test]
  fn test_parse_conditions() {
    let patterns = vec!["github.com", "^facebook.com", "twitter.com$"];
    let expected = vec!["contains \"github.com\"", "starts with \"facebook.com\"", "ends with \"twitter.com\""];
    let actual = parse_conditions(patterns);
    assert_eq!(actual, expected);
  }
}


/// Close tabs in Safari that match URL patterns.
///
/// Takes a list of URL patterns, and tries to close any matching Safari tabs.
/// Bugginess in AppleScript means this isn't always perfect, but we can
/// have a go.  In general it will fail to close tabs, rather than close
/// the wrong tabs.
///
pub fn close_tabs(url_patterns: Vec<&str>) {
  let conditions = parse_conditions(url_patterns);

  let clean_tabs_template = include_str!("scripts/clean-tabs.scpt");
  let mut context = Context::new();
  context.add("conditions", &conditions);
  let script = Tera::one_off(&clean_tabs_template, context, false).unwrap();

  // Run it twice to get around weird AppleScript bugs.
  run_applescript(&script);
  run_applescript(&script);
}


/// Get the Bookmarks.plist dict for a given title
fn read_bookmarks_plist(title: &str) -> Plist {

  // All Safari data lives at ~/Library/Safari/Bookmarks.plist
  // TODO: There's probably a more idiomatic Rust-like way to write this.
  let mut plist_path = match env::home_dir() {
    Some(v) => v,
    None => error!("Unable to get path to Bookmarks.plist?"),
  };
  plist_path.push("Library/Safari/Bookmarks.plist");

  let file = match File::open(plist_path) {
    Ok(v) => v,
    Err(e) => error!("Unable to open ~/Library/Safari/Bookmarks.plist: {:?}", e),
  };

  let plist = match Plist::read(file) {
    Ok(v) => v,
    Err(e) => error!("Unable to read ~/Library/Safari/Bookmarks.plist: {:?}", e),
  };

  let data = match plist.as_dictionary() {
    Some(v) => v,
    None => error!("Unable to parse ~/Library/Safari/Bookmarks.plist as dictionary?"),
  };

  // The structure of Bookmarks.plist is as follows:
  //
  //     <dict>
  //       <key>Children</key>
  //       <array>
  //         <dict>
  //           <key>Title</key><string>History</string>
  //           ... dict data ...
  //         </dict>
  //         <dict>
  //           <key>Title</key><string>com.apple.ReadingList</string>
  //           ... dict data ...
  //         </dict>
  //         ... other array items ...
  //       </array>
  //     </dict>
  //
  let children = match data.get("Children") {
    Some(child_key) => match child_key.as_array() {
      Some(v) => v,
      None => error!("Top-level children key in ~/Library/Safari/Bookmarks.plist isn't an array?"),
    },
    None => error!("Unable to find top-level Children key in ~/Library/Safari/Bookmarks.plist"),
  };

  let mut matching_children = children.iter().filter(|d|
    match d.as_dictionary() {
      Some(dict) => match dict.get("Title") {
        Some(title_elem) => match title_elem.as_string() {
          Some(v) => (v == title),
          None => false,
        },
        None => false,
      },
      None => false,
    }
  );

  // Check we got one, and only one result.
  let result = match matching_children.next() {
    Some(v) => v,
    None => error!("Unable to find key {} in Bookmarks.plist", title),
  };

  match matching_children.next() {
    Some(_) => error!("Got more than one result for {} in Bookmarks.plist", title),
    None => {},
  };

  result.to_owned()
}


/// Return a list of URLs from Reading List.
///
/// Iteration order depends on the order in which they're stored in
/// Bookmarks.plist, which is usually (but not guaranteed to be) newest first.
///
pub fn get_reading_list_urls() -> Vec<String> {
  let plist = read_bookmarks_plist("com.apple.ReadingList");

  // TODO: All these unwrap() calls should probably be handled better
  let children = plist
    .as_dictionary().unwrap()
    .get("Children").unwrap()
    .as_array().unwrap();

  children
    .iter()
    .map(|child|
      child
        .as_dictionary().unwrap()
        .get("URLString").unwrap()
        .as_string().unwrap()
    )
    .map(|url| urls::tidy_url(url))
    .collect()
}
