#![deny(warnings)]

extern crate dirs;
extern crate docopt;
extern crate plist;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate tera;
extern crate urlencoding;
extern crate urlparse;

use std::io::Write;
use std::process;

mod applescript;
mod cli;
mod safari;
mod urls;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");


// http://stackoverflow.com/a/27590832/1558022
macro_rules! error(
  ($($arg:tt)*) => {
    {
      let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
      r.expect("failed printing to stderr");
      process::exit(1);
    }
  }
);


/// Exits the program if Safari isn't running.
fn assert_safari_is_running() {
  if !safari::is_safari_running() {
    error!("Safari is not running.");
  }
}


fn main() {
  let args = cli::parse_args(NAME);

  if args.flag_version {
    println!("{}.rs v{}", NAME, VERSION);
  }

  if args.cmd_url {
    assert_safari_is_running();
    match safari::get_url(args.flag_window, args.flag_tab) {
      Ok(url) => print!("{}", url),
      Err(e) => error!("{}", e),
    };
  }

  if args.cmd_resolve {
    print!("{}", urls::resolve(&args.arg_url));
  }

  if args.cmd_list_tabs {
    assert_safari_is_running();
    for url in safari::get_all_urls() {
      println!("{}", url);
    }
  }

  if args.cmd_close_tabs {
    assert_safari_is_running();
    let patterns = args.arg_urls_to_close.split(",").collect();
    safari::close_tabs(patterns);
  }

  if args.cmd_reading_list {
    match safari::get_reading_list_urls() {
      Ok(urls) => {
        for url in urls {
          println!("{}", url);
        }
      },
      Err(e) => error!("{}", e),
    };
  }

  if args.cmd_icloud_tabs {
    if args.flag_list_devices {
      match safari::list_icloud_tabs_devices() {
        Ok(devices) => {
          for device in devices {
            println!("{}", device);
          }
        },
        Err(e) => error!("{}", e),
      };
    } else {
      let tab_data = match safari::get_icloud_tabs_urls() {
        Ok(tab_data) => tab_data,
        Err(e) => error!("{}", e),
      };
      match args.flag_device {
        Some(d) => {
          match tab_data.get(&d) {
            Some(urls) => {
              for url in urls {
                println!("{}", url);
              }
            },
            None => (),
          }
        },
        None => {
          for urls in tab_data.values() {
            for url in urls {
              println!("{}", url);
            }
          }
        }
      }
    }
  }

  if args.cmd_tidy_url {
    print!("{}", urls::tidy_url(&args.arg_url));
  }
}
