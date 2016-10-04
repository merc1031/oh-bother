extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};

use std::error::Error;
use std::fs::File;
use std::path::Path;

mod util;

fn main() {
    let matches = App::new("ob")
        .version("1.0.0")
        .author("Matt Chun-Lum")
        .about("JIRA interrupt management")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .arg(Arg::with_name("dry")
            .help("show what request(s) would be made")
            .long("dry-run"))
        .arg(Arg::with_name("config")
            .help("sets the config file to use (default is \"~/.ob.yml\")")
            .takes_value(true)
            .default_value("~/.ob.yml")
            .short("c")
            .long("config"))
        .subcommand(SubCommand::with_name("new")
            .about("Creates a new pull request")
            .setting(AppSettings::ColoredHelp)
            .arg(Arg::with_name("title")
                .help("the title of the pr")
                .index(1)
                .required(true))
            .arg(Arg::with_name("foo")
                .help("Foo")
                .long("foo")))
        .get_matches();

    let config_file = matches.value_of("config").unwrap();
    let config_path = Path::new(config_file);

    let config = match File::open(&config_path) {
        Err(why) => {
            util::exit(&format!("couldn't open config file {}: {}",
                                config_path.display(),
                                why.description()))
        }
        Ok(file) => file,
    };

    match matches.subcommand_name() {
        Some("new") => println!("new"),
        _ => println!("unknown"),
    }
}
