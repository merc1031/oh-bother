#[macro_use]
extern crate clap;
extern crate hyper;
extern crate prettytable;
extern crate rpassword;
extern crate rustc_serialize;
extern crate url;
extern crate yaml_rust;

use clap::{App, Arg, ArgMatches};

use std::env;
use std::path::Path;

use config::Config;
use jira::Jira;

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
            Some("issue") => issue(&config, &jira, &matches),
            Some("list") => println!("list"),
            Some("current") => current(&config, &jira),
            Some("next") => next(&config, &jira),
            Some("start") => println!("start"),
            Some("stop") => println!("stop"),
            Some("close") => println!("close"),
            Some("new") => new(&config, &jira, &matches),
            Some("jql") => jql(&jira, &matches),
            _ => util::exit("unknown command"), // shouldn't really ever get here
        }
    }
}

fn issue(config: &Config, jira: &Jira, matches: &ArgMatches) {
    let subcmd = match matches.subcommand_matches("issue") {
        Some(matches) => matches,
        None => util::exit("this should not be possible"),
    };

    let issue_key = subcmd.value_of("issue").unwrap();
    let result = match jira.issue(issue_key) {
        Err(why) => util::exit(&format!("Error finding issue {}: {}", issue_key, why)),
        Ok(result) => result,
    };

    let issue = match result {
        Some(issue) => issue,
        None => util::exit(&format!("Issue {} not found", issue_key)),
    };

    issue.print_tty(false);

    if subcmd.is_present("open") {
        util::open_in_browser(config, jira, &issue)
    }
}

fn current(config: &Config, jira: &Jira) {
    let query = format!("project in ({}) AND assignee = {} AND status not in (Resolved, Closed)",
                        config.projects(),
                        config.username);
    util::perform_query(jira, &query, |result| result.as_filtered_table(&["key", "status", "summary"]))
}

fn next(config: &Config, jira: &Jira) {
    let query = format!("project in ({}) AND status = Open AND assignee in ({})",
                        config.projects(),
                        config.npc_users());
    util::perform_query(jira, &query, |result| result.as_filtered_table(&["key", "reporter", "summary"]))
}

fn new(config: &Config, jira: &Jira, matches: &ArgMatches) {
    let subcmd = match matches.subcommand_matches("issue") {
        Some(matches) => matches,
        None => util::exit("this should not be possible"),
    };

    let project = subcmd.value_of("project").unwrap_or(config.defaults.project_key.as_str());
    let summary = subcmd.value_of("summary").unwrap();
    let assignee = subcmd.value_of("assignee").unwrap_or(config.defaults.assignee.as_str());

    let labels = ["foo"];

    let issue = match jira.create_issue(project, summary, assignee, &labels) {
        Err(why) => util::exit(&format!("Error creating issue \"{}\": {}", summary, why)),
        Ok(issue) => issue,
    };

    if config.open_in_browser {
        util::open_in_browser(config, jira, &issue)
    }
}

fn jql(jira: &Jira, matches: &ArgMatches) {
    let subcmd = match matches.subcommand_matches("jql") {
        Some(matches) => matches,
        None => util::exit("this should not be possible"),
    };

    let query = subcmd.value_of("query").unwrap();
    util::perform_query(jira, query, |result| result.as_table())
}
