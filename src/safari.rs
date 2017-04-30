use std::env;
use std::fs::File;
use std::process;

use applescript::{run as run_applescript};

use plist::Plist;

use tera::{Context, Tera};


pub fn safari_furl() -> String {
    run_applescript("tell application \"Safari\" to get URL of document 1").stdout
}


pub fn safari_2url() -> String {
    run_applescript("tell application \"Safari\" to get URL of document 2").stdout
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
