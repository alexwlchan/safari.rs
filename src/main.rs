#![deny(warnings)]

#[macro_use]
extern crate clap;
extern crate plist;
extern crate tera;
extern crate urlparse;

use std::process;

use clap::App;

mod applescript;
mod safari;
mod urls;


fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml).usage("furl <subcommand>");
    let matches = app.get_matches();

    if let Some(_) = matches.subcommand_matches("furl") {
      print!("{}", safari::get_url(Some(1), None));
    } else if let Some(_) = matches.subcommand_matches("2url") {
      print!("{}", safari::get_url(Some(2), None));
    } else if let Some(matches) = matches.subcommand_matches("clean-tabs") {
      // TODO: Tidy this up!!
      let patterns = matches.args.get("urls").unwrap()
        .vals.get(1).unwrap()
        .to_str().unwrap()
        .split(",")
        .collect();
      safari::close_tabs(patterns);
    } else if let Some(_) = matches.subcommand_matches("list-tabs") {
      let urls = safari::get_all_urls();
      for url in urls {
        println!("{}", url);
      }
    } else if let Some(_) = matches.subcommand_matches("reading-list") {
      let urls = safari::get_reading_list_urls();
      for url in urls {
        println!("{}", url);
      }
    } else {
        App::from_yaml(yaml)
            .usage("furl <subcommand>")
            .print_help()
            .ok();
        process::exit(1);
    }
}
