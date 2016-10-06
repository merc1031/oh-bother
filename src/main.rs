extern crate clap;
extern crate yaml_rust;

use clap::{App, AppSettings, Arg, SubCommand};

use std::path::Path;
use std::env;

use config::Config;

mod util;
mod config;

fn main() {
    let default_config_path = env::home_dir().unwrap().join(".ob.yml");
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
            .help("sets the config file to use")
            .takes_value(true)
            .default_value(default_config_path.to_str().unwrap())
            .short("c")
            .long("config"))
        .subcommand(SubCommand::with_name("setup")
            .about("Configures oh-bother")
            .setting(AppSettings::ColoredHelp))
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

    let config = match Config::new(&config_path) {
        Err(why) => {
            println!("There was an error loading the config. Maybe run 'setup'?");
            util::exit(&format!("couldn't open config file {}: {}",
                                config_path.display(),
                                why))
        }
        Ok(config) => config,
    };

    match matches.subcommand_name() {
        Some("new") => println!("new"),
        _ => println!("unknown"),
    }
}
