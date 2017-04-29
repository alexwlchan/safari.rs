#![deny(warnings)]

extern crate docopt;
extern crate plist;
extern crate rustc_serialize;
extern crate tera;
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
    println!("{} v{}", NAME, VERSION);
  } else if args.cmd_url {
    safari::assert_safari_is_running();
    print!("{}", safari::get_url(args.flag_window, args.flag_tab));
  } else if args.cmd_urls_all {
    safari::assert_safari_is_running();
    let urls = safari::get_all_urls();
    for url in urls {
      println!("{}", url);
    }
  } else if args.cmd_close_tabs {
    safari::assert_safari_is_running();
    let patterns = args.arg_urls_to_close.split(",").collect();
    safari::close_tabs(patterns);
  } else if args.cmd_reading_list {
    let urls = safari::get_reading_list_urls();
    for url in urls {
      println!("{}", url);
    }
  }
}
