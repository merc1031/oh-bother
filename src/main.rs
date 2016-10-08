extern crate itertools;
extern crate clap;
extern crate hyper;
extern crate prettytable;
extern crate rpassword;
extern crate rustc_serialize;
extern crate url;
extern crate yaml_rust;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use std::path::Path;
use std::env;

use config::Config;
use jira::Jira;

mod config;
mod issue;
mod jira;
mod util;

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
        .subcommand(SubCommand::with_name("list")
            .about("Lists interrupts")
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("current")
            .about("Lists tickets currently being worked on or assigned")
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("next")
            .about("Lists available interrupts")
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("start")
            .about("Start the specified interrupt")
            .arg(Arg::with_name("issue")
                .help("the issue")
                .index(1)
                .required(true))
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("close")
            .about("Close the specified interrupt")
            .arg(Arg::with_name("issue")
                .help("the issue")
                .index(1)
                .required(true))
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("new")
            .about("Creates a new interrupt")
            .arg(Arg::with_name("title")
                .help("the title of the interrupt")
                .index(1)
                .required(true))
            .arg(Arg::with_name("foo")
                .help("Foo")
                .long("foo"))
            .setting(AppSettings::ColoredHelp))
        .subcommand(SubCommand::with_name("jql")
            .about("Execute a raw jql query")
            .arg(Arg::with_name("query")
                .help("the jql query")
                .index(1)
                .required(true))
            .setting(AppSettings::ColoredHelp))
        .get_matches();

    let config_file = matches.value_of("config").unwrap();
    let config_path = Path::new(config_file);

    if matches.is_present("setup") {
        match Config::create(&config_path) {
            Err(why) => {
                println!("There was an error creating the config.");
                util::exit(&format!("couldn't create config file {}: {}",
                                    config_path.display(),
                                    why))
            }
            Ok(_) => {}
        }
        println!("Please edit {} to include your desired configuration",
                 config_path.display());
    } else {
        let config = match Config::new(&config_path) {
            Err(why) => {
                println!("There was an error loading the config. Maybe run 'setup'?");
                util::exit(&format!("couldn't open config file {}: {}",
                                    config_path.display(),
                                    why))
            }
            Ok(config) => config,
        };

        let jira = match Jira::new(config.auth.as_str(), config.jira_url.as_str()) {
            Err(why) => util::exit(&format!("couldn't construct client: {}", why)),
            Ok(jira) => jira,
        };

        match matches.subcommand_name() {
            Some("list") => println!("list"),
            Some("current") => current(&config, &jira),
            Some("next") => next(&config, &jira),
            Some("start") => println!("start"),
            Some("close") => println!("close"),
            Some("new") => println!("new"),
            Some("jql") => jql(&jira, &matches),
            _ => util::exit("unknown command"), // shouldn't really ever get here
        }
    }
}

fn current(config: &Config, jira: &Jira) {
    let query = format!("project = IRA AND assignee = {} AND status not in (Resolved, Closed)",
                        config.username);
    let result = match jira.query(&query) {
        Err(why) => util::exit(&format!("Error executing query {}: {}", query, why)),
        Ok(result) => result,
    };

    match result {
        Some(result) => result.as_filtered_table(&["key", "status", "summary"]).print_tty(false),
        None => println!("the query \"{}\" returned no issues", query),
    }
}

fn next(config: &Config, jira: &Jira) {
    // let query = format!("project = IRA AND status = Open AND assignee = {}", config.username);
    let query = "project = IRA AND status = Open AND assignee = ir-devtools-robot";
    let result = match jira.query(&query) {
        Err(why) => util::exit(&format!("Error executing query {}: {}", query, why)),
        Ok(result) => result,
    };

    match result {
        Some(result) => result.as_filtered_table(&["key", "summary"]).print_tty(false),
        None => println!("the query \"{}\" returned no issues", query),
    }
}

fn jql(jira: &Jira, matches: &ArgMatches) {
    let subcmd = match matches.subcommand_matches("jql") {
        Some(matches) => matches,
        None => util::exit("this should not be possible"),
    };

    let query = subcmd.value_of("query").unwrap();
    let result = match jira.query(&query) {
        Err(why) => util::exit(&format!("Error executing query {}: {}", query, why)),
        Ok(result) => result,
    };

    match result {
        Some(result) => result.as_table().print_tty(false),
        None => println!("the query \"{}\" returned no issues", query),
    }
}
