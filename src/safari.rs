use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::File;
use std::process;

use applescript::{run as run_applescript};

use plist::Plist;

use tera::{Context, Tera};

use urls;


macro_rules! error(
  ($($arg:tt)*) => { { return Err(format!($($arg)*)) } }
);


/// Returns true/false if Safari is running.
pub fn is_safari_running() -> bool {
  let cmd_result = process::Command::new("ps")
    .arg("-eaf")
    .output()
    .expect("Unable to test if Safari is running.");
  for line in String::from_utf8(cmd_result.stdout).unwrap().lines() {
    if line.contains("Safari.app/Contents/MacOS/Safari") {
      return true
    }
  }
  false
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
pub fn get_url(window: Option<i32>, tab: Option<i32>) -> Result<String, String> {
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
    Ok(urls::tidy_url(output.stdout.trim()))
  } else {
    if output.stderr.contains("Invalid index") {
      error!("Invalid index: no such window or tab.")
    } else {
      error!("Unexpected error from osascript: {:?}", output.stderr)
    }
  }
}


/// Return a list of URLs from Safari.
///
/// This returns a list of URLs, one for every tab in Safari.  Iteration
/// order depends on AppleScript, which I don't think is guaranteed to be
/// stable (in particular, I think it depends on which window is frontmost).
///
pub fn get_all_urls() -> Result<Vec<String>, String> {
  let script = include_str!("scripts/list-open-tabs.scpt");
  let output = run_applescript(&script);
  if output.status.success() {
    Ok(parse_list_open_tabs_output(&output.stdout))
  } else {
    error!("Unexpected error from osascript: {:?}", output.stderr);
  }
}


/// Given AppleScript output from list-open-tabs.scpt, return a list of URLs.
fn parse_list_open_tabs_output(stdout: &str) -> Vec<String> {
  stdout
    .trim()
    .split(", ")
    .map(|url| urls::tidy_url(url))
    .filter(|url| url != "favorites://")
    .filter(|url| url != "://missing value")
    .collect()
}


macro_rules! parse_list_open_tabs_output_tests {
  ($($name:ident: $value:expr,)*) => {
    $(
      #[test]
      fn $name() {
        let (input, expected) = $value;
        assert_eq!(expected, parse_list_open_tabs_output(input));
      }
    )*
  }
}


parse_list_open_tabs_output_tests! {

  // I don't have a good way to put an empty vector on the right-hand side of
  // this comparison, so no test checks for nothing (yet).

  favorites_with_url: (
    "favorites://, http://example.org",
    vec!["http://example.org"]
  ),

  url_with_missing_value: (
    "://missing value, https://www.example.net",
    vec!["https://www.example.net"]
  ),

  single_url: (
    "http://foo_bar.com",
    vec!["http://foo_bar.com"]
  ),

  multiple_urls: (
    "http://example.org, https://www.example.net, http://test.co.uk",
    vec!["http://example.org", "https://www.example.net", "http://test.co.uk"]
  ),

  with_extra_whitespace: (
    "    http://space.org, https://www.nasa.gov   ",
    vec!["http://space.org", "https://www.nasa.gov"]
  ),

  applies_tidy_url_transform: (
    "https://mobile.twitter.com",
    vec!["https://twitter.com"]
  ),
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
fn read_bookmarks_plist(title: &str) -> Result<Plist, String> {

  // All Safari data lives at ~/Library/Safari/Bookmarks.plist
  // TODO: There's probably a more idiomatic Rust-like way to write this.
  let mut plist_path = match env::home_dir() {
    Some(v) => v,
    None => error!("Unable to get home directory?"),
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

  Ok(result.to_owned())
}


/// Return a list of URLs from Reading List.
///
/// Iteration order depends on the order in which they're stored in
/// Bookmarks.plist, which is usually (but not guaranteed to be) newest first.
///
pub fn get_reading_list_urls() -> Result<Vec<String>, String> {
  let plist = match read_bookmarks_plist("com.apple.ReadingList") {
    Ok(v) => v,
    Err(e) => return Err(e),
  };

  // TODO: All these unwrap() calls should probably be handled better
  let children = plist
    .as_dictionary().unwrap()
    .get("Children").unwrap()
    .as_array().unwrap();

  Ok(children
    .iter()
    .map(|child|
      child
        .as_dictionary().unwrap()
        .get("URLString").unwrap()
        .as_string().unwrap()
    )
    .map(|url| urls::tidy_url(url))
    .collect()
  )
}


/// Get the com.apple.Safari.plist preferences file
fn read_safari_plist() -> Result<BTreeMap<String, Plist>, String> {

  // All Safari data lives at ~/Library/Safari/Bookmarks.plist
  // TODO: There's probably a more idiomatic Rust-like way to write this.
  let mut plist_path = match env::home_dir() {
    Some(v) => v,
    None => error!("Unable to get home directory?"),
  };
  plist_path.push("Library/SyncedPreferences/com.apple.Safari.plist");

  let file = match File::open(plist_path) {
    Ok(v) => v,
    Err(e) => error!("Unable to open com.apple.Safari.plist: {:?}", e),
  };

  let plist = match Plist::read(file) {
    Ok(v) => v,
    Err(e) => error!("Unable to read com.apple.Safari.plist: {:?}", e),
  };

  let data = match plist.as_dictionary() {
    Some(v) => v,
    None => error!("Unable to parse com.apple.Safari.plist as dictionary?"),
  };

  // The structure of com.apple.Safari.plist is as follows:
  //
  //      <dict>
  //        <key>values</key>
  //        <dict>
  //          <dict>
  //            <key>[[ UUID ]]</key>
  //            <dict>
  //              ... dict data ...
  //              <key>value</key>
  //              <dict>
  //                ... dict data ...
  //                <key>DeviceName</key><string>[[ Device name ]]</string>
  //                <key>Tabs</key>
  //                <array>
  //                  <dict>
  //                    <key>Title</key><string>[[ Title ]]</string>
  //                    <key>URL</key><string>[[ URL ]]</string>
  //                  </dict>
  //                  ... array items for other tabs ...
  //                </array>
  //              </dict>
  //            </dict>
  //            ... dicts for other devices ...
  //          </dict>
  //        </dict>
  //      </dict>
  //
  match data.get("values") {
    Some(child_key) => match child_key.as_dictionary() {
      Some(v) => Ok(v.to_owned()),
      None => error!("Top-level values key in com.apple.Safari.plist isn't an dictionary?"),
    },
    None => error!("Unable to find top-level values key in com.apple.Safari.plist"),
  }
}


/// Return a list of devices in iCloud Tabs.
pub fn list_icloud_tabs_devices() -> Result<Vec<String>, String> {
  let plist = match read_safari_plist() {
    Ok(plist) => plist,
    Err(e) => return Err(e),
  };

  Ok(plist
    .values()
    .map(|value| value
      .as_dictionary().unwrap()
      .get("value").unwrap()
      .as_dictionary().unwrap()
      .get("DeviceName").unwrap()
      .as_string().unwrap()
      .to_owned()
    )
    .collect()
  )
}


/// Return a list of URLs from iCloud Tabs.
pub fn get_icloud_tabs_urls() -> Result<HashMap<String, Vec<String>>, String> {
  let plist = match read_safari_plist() {
    Ok(plist) => plist,
    Err(e) => return Err(e),
  };

  // Within the `values` dictionary, each device is as follows:
  //
  //    <key>[[ device UUID ]]</key>
  //    <dict>
  //      [[ device data ]]
  //      <dict>
  //        <key>Tabs</key>
  //        <key>DeviceName</key><string>[[ device name ]]</key>
  //        <key>Tabs</key>
  //        <array>
  //          <dict>
  //            <key>URL</key><string>[[ URL ]]</string>
  //            [[ other tab data ]]
  //          </dict>
  //          [[ other tabs ]]
  //
  let mut result: HashMap<String, Vec<String>> = HashMap::new();
  let devices_with_tabs = plist
    .values()
    .map(|value| value
      .as_dictionary().unwrap()
      .get("value").unwrap()
      .as_dictionary().unwrap()
    );
  for data in devices_with_tabs {
    let name: String = data
      .get("DeviceName").unwrap()
      .as_string().unwrap()
      .to_owned();

    // If a device is registered with iCloud but Safari isn't running or
    // there aren't any tabs open, there isn't a Tabs field.
    if !data.contains_key("Tabs") {
      continue;
    }

    let urls: Vec<String> = data
      .get("Tabs").unwrap()
      .as_array().unwrap()
      .iter()
      .map(|tab| tab
        .as_dictionary().unwrap()
        .get("URL").unwrap()
        .as_string().unwrap()
      )
      .map(|url| urls::tidy_url(url))
      .collect();
    result.insert(name, urls);
  }
  Ok(result)
}
