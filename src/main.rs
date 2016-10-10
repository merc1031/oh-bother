extern crate itertools;
#[macro_use]
extern crate clap;
extern crate hyper;
extern crate prettytable;
extern crate rpassword;
extern crate rustc_serialize;
extern crate url;
extern crate yaml_rust;

use clap::{App, Arg, ArgMatches};
use prettytable::Table;

use std::path::Path;
use std::env;

use config::Config;
use jira::Jira;
use issue::IssueVec;

mod config;
mod issue;
mod jira;
mod util;

fn main() {
    let default_config_path = env::home_dir().unwrap().join(".ob.yml");
    let yml = load_yaml!("app.yml");
    let matches = App::from_yaml(yml)
        .arg(Arg::with_name("config")
            .help("sets the config file to use")
            .takes_value(true)
            .default_value(default_config_path.to_str().unwrap())
            .short("c")
            .long("config"))
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
            Some("issue") => issue(&jira, &matches),
            Some("list") => println!("list"),
            Some("current") => current(&config, &jira),
            Some("next") => next(&config, &jira),
            Some("start") => println!("start"),
            Some("stop") => println!("stop"),
            Some("close") => println!("close"),
            Some("new") => println!("new"),
            Some("jql") => jql(&jira, &matches),
            _ => util::exit("unknown command"), // shouldn't really ever get here
        }
    }
}

fn issue(jira: &Jira, matches: &ArgMatches) {
    let subcmd = match matches.subcommand_matches("issue") {
        Some(matches) => matches,
        None => util::exit("this should not be possible"),
    };

    let issue_key = subcmd.value_of("issue").unwrap();
    let result = match jira.issue(&issue_key) {
        Err(why) => util::exit(&format!("Error finding issue {}: {}", issue_key, why)),
        Ok(result) => result,
    };

    match result {
        Some(issue) => println!("{}", issue),
        None => println!("Issue {} not found", issue_key),
    }
}

fn current(config: &Config, jira: &Jira) {
    let query = format!("project in ({}) AND assignee = {} AND status not in (Resolved, Closed)",
                        config.projects(),
                        config.username);
    perform_query(jira, &query, |result| result.as_filtered_table(&["key", "status", "summary"]))
}

fn next(config: &Config, jira: &Jira) {
    let query = format!("project in ({}) AND status = Open AND assignee in ({})",
                        config.projects(),
                        config.npc_users());
    perform_query(jira, &query, |result| result.as_filtered_table(&["key", "reporter", "summary"]))
}

fn jql(jira: &Jira, matches: &ArgMatches) {
    let subcmd = match matches.subcommand_matches("jql") {
        Some(matches) => matches,
        None => util::exit("this should not be possible"),
    };

    let query = subcmd.value_of("query").unwrap();
    perform_query(jira, query, |result| result.as_table())
}

fn perform_query<F>(jira: &Jira, query: &str, table_fn: F)
    where F : Fn(IssueVec) -> Table
{
    let result = match jira.query(query) {
        Err(why) => util::exit(&format!("Error executing query {}: {}", query, why)),
        Ok(result) => result,
    };

    match result {
        Some(result) => table_fn(result).print_tty(false),
        None => println!("the query \"{}\" returned no issues", query),
    }
}
