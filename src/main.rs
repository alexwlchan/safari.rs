#![deny(warnings)]

extern crate docopt;
extern crate plist;
extern crate rustc_serialize;
extern crate tera;
extern crate urlencoding;
extern crate urlparse;

mod applescript;
mod cli;
mod safari;
mod urls;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");


fn main() {
  let args = cli::parse_args(NAME);

  if args.flag_version {
    println!("{}.rs v{}", NAME, VERSION);
  }

  if args.cmd_url {
    safari::assert_safari_is_running();
    print!("{}", safari::get_url(args.flag_window, args.flag_tab));
  }

  if args.cmd_urls_all {
    safari::assert_safari_is_running();
    for url in safari::get_all_urls() {
      println!("{}", url);
    }
  }

  if args.cmd_close_tabs {
    safari::assert_safari_is_running();
    let patterns = args.arg_urls_to_close.split(",").collect();
    safari::close_tabs(patterns);
  }

  if args.cmd_reading_list {
    for url in safari::get_reading_list_urls() {
      println!("{}", url);
    }
  }

  if args.cmd_icloud_tabs {
    if args.flag_list_devices {
      for device in safari::list_icloud_tabs_devices() {
        println!("{}", device);
      }
    } else {
      let tab_data = safari::get_icloud_tabs_urls();
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
}
