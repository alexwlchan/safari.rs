use docopt::{Docopt, Error};

const USAGE: &str = "
Usage: <NAME> url [--window=<WINDOW> [--tab=<TAB>]]
       <NAME> urls-all
       <NAME> close-tabs <urls-to-close>
       <NAME> reading-list
       <NAME> icloud-tabs [--list-devices | --device=<DEVICE>]
       <NAME> (-h | --help)
       <NAME> --version

Options:
    -h --help           Show this screen.
    --version           Show version.
    --window=<WINDOW>   Which window to choose a URL from.  Use 1 for the
                        frontmost window, 2 for the second window, and so on.
    --tab=<TAB>         Which tab to choose a URL from.  Use 1 for the leftmost
                        tab, 2 for second-from-left, and so on.
    --list-devices      Get a list of all the devices known to iCloud Tabs.
    --device=<DEVICE>   Only get iCloud URLs for this device.

Commands:
    url           Print a URL from an open Safari tab.
    urls-all      Prints a list of URLs from every open Safari tab.
    close-tabs    Close any tabs with the given URLs.
    reading-list  Print a list of URLs from Reading List.
    icloud-tabs   Get a list of URLs from iCloud Tabs.  Default is to list URLs
                  from every device, or you can filter with the --device flag.
";

#[derive(Debug, RustcDecodable)]
pub struct Args {
  pub cmd_url: bool,
  pub cmd_urls_all: bool,
  pub cmd_close_tabs: bool,
  pub cmd_icloud_tabs: bool,
  pub cmd_reading_list: bool,
  pub flag_window: Option<i32>,
  pub flag_tab: Option<i32>,
  pub flag_version: bool,
  pub flag_list_devices: bool,
  pub flag_device: Option<String>,
  pub arg_urls_to_close: String,
}

pub fn parse_args(name: &str) -> Args {
  let mut args: Args = Docopt::new(str::replace(USAGE, "<NAME>", name))
                              .and_then(|d| d.decode())
                              .unwrap_or_else(|e| e.exit());

  // 0 is the default value for the --window and --tab flags, so if we get
  // this value then replace it with None.
  if args.cmd_url {
    match args.flag_window {
      Some(v) => {
        if v == 0 {
          args.flag_window = None;
        } else if v < 0 {
          let err_message = format!("--window must be greater than 0; got {}.", v);
          Error::Usage(err_message).exit();
        };
      },
      None => {},
    };
    match args.flag_tab {
      Some(v) => {
        if v == 0 {
          args.flag_tab = None;
        } else if v < 0 {
          let err_message = format!("--tab must be greater than 0; got {}.", v);
          Error::Usage(err_message).exit();
        };
      },
      None => {},
    };

    if args.flag_tab.is_some() && args.flag_window.is_none() {
      Error::Usage("Cannot use --tab without --window.".to_string()).exit();
    }
  }

  args
}
