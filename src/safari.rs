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
pub fn get_safari_url(window: Option<i32>, tab: Option<i32>) -> String {
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


pub fn safari_furl() -> String {
    get_safari_url(Some(1), None)
}


pub fn safari_2url() -> String {
    get_safari_url(Some(2), None)
}


pub fn safari_closetabs(urls: Vec<&str>) -> String {
    let clean_tabs_template = include_str!("scripts/clean-tabs.scpt");

    let mut context = Context::new();
    context.add("urls", &urls);

    let script = Tera::one_off(&clean_tabs_template, context, false).unwrap();
    run_applescript(&script).stdout
}


/// Prints a list of open tabs in Safari
pub fn list_open_tabs() -> String {
    let list_open_tabs_template = include_str!("scripts/list-open-tabs.scpt");

    let context = Context::new();
    let script = Tera::one_off(&list_open_tabs_template,
                               context,
                               false).unwrap();
    run_applescript(&script).stdout
}


/// Prints a list of tabs in Reading List
pub fn reading_list() {
    let mut plist_path = env::home_dir().unwrap();
    plist_path.push("Library/Safari/Bookmarks.plist");
    let file = File::open(plist_path).unwrap();
    let plist = Plist::read(file).unwrap();

    let data = match plist {
        Plist::Dictionary(dict) => dict,
        _ => {
            println!("Unable to parse ~/Library/Safari/Bookmarks.plist");
            process::exit(1);
        }
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
    // We're interested in the dict with title 'com.apple.ReadingList'.
    let children = data.get("Children").unwrap();
    for child in children.as_array().unwrap().iter() {

        // There might be a more Rust-idiomatic way to get to the <dict>
        // we want, but this seems to work.
        let child_dict = match child.as_dictionary() {
            Some(d) => d,
            None => continue,
        };
        match child_dict.get("Title") {
            Some(d) => {
                if d.as_string().unwrap() != "com.apple.ReadingList" {
                    continue
                }
            },
            None => continue,
        }
        let rl_items = child_dict.get("Children").unwrap();
        for item in rl_items.as_array().unwrap().iter() {
            println!("{}", item.as_dictionary().unwrap()
                               .get("URLString").unwrap()
                               .as_string().unwrap());
        }
    }
}
