#![recursion_limit = "1024"] // error chain recursion can be deep

extern crate base64;
#[macro_use]
extern crate clap;
extern crate eprompt;
#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate prettytable;
extern crate rpassword;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate yaml_rust;

use std::env;
use std::path::Path;

use clap::{App, Arg, ArgMatches};
use eprompt::Prompt;

use config::Config;
use jira::Jira;

mod config;
mod issue;
mod jira;
mod jira_data;
mod util;
mod error;

fn main() {
    let default_config_path = env::home_dir().unwrap().join(".ob.yml");
    let yml = load_yaml!("app.yml");
    let matches = App::from_yaml(yml)
        .arg(
            Arg::with_name("config")
                .help("sets the config file to use")
                .takes_value(true)
                .default_value(default_config_path.to_str().unwrap())
                .short("c")
                .long("config")
                .global(true),
        )
        .get_matches();

    let config_file = matches.value_of("config").unwrap();
    let config_path = Path::new(config_file);

    let debug = matches.is_present("debug");

    if matches.is_present("setup") {
        match Config::create(&config_path) {
            Err(why) => {
                println!("There was an error creating the config.");
                util::exit(&format!(
                    "couldn't create config file {}: {}",
                    config_path.display(),
                    why
                ))
            }
            Ok(_) => {}
        }
        println!(
            "Please edit {} to include your desired configuration",
            config_path.display()
        );
    } else {
        let config = match Config::new(&config_path) {
            Err(why) => {
                println!("There was an error loading the config. Maybe run 'setup'?");
                util::exit(&format!(
                    "couldn't open config file {}: {}",
                    config_path.display(),
                    why
                ))
            }
            Ok(config) => config,
        };

        let jira = match Jira::new(config.auth.as_str(), config.jira_url.as_str(), debug) {
            Err(why) => util::exit(&format!("couldn't construct client: {}", why)),
            Ok(jira) => jira,
        };

        match matches.subcommand_name() {
            Some("issue") => issue(&config, &jira, &matches),
            Some("list") => list(&config, &jira, &matches),
            Some("current") => current(&config, &jira, &matches),
            Some("next") => next(&config, &jira, &matches),
            Some("start") => println!("start not implemented"),
            Some("stop") => println!("stop not implemented"),
            Some("close") => println!("close not implemented"),
            Some("new") => new(&config, &jira, &matches, debug),
            Some("jql") => jql(&config, &jira, &matches),
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
    let issue = match jira.issue(issue_key) {
        Err(why) => util::exit(&format!("Error finding issue {}: {}", issue_key, why)),
        Ok(issue) => issue,
    };

    issue.print_tty(false);

    if subcmd.is_present("open") {
        util::open_in_browser(config, &issue)
    }
}

fn query_helper(config: &Config, jira: &Jira, query: &str, output_columns: &[&str], prompt: bool) {
    let issues = util::perform_query(jira, query);
    util::render_issues(&issues, |result| {
        result.as_filtered_table(output_columns)
    });

    if prompt {
        let issue = util::prompt_for_issue(&issues);
        util::open_in_browser(config, issue);
    }
}

fn list(config: &Config, jira: &Jira, matches: &ArgMatches) {
    let query = format!(
        "project in ({}) AND status not in (Resolved, Closed)",
        config.projects()
    );
    let output_columns = ["key", "reporter", "assignee", "status", "summary"];
    query_helper(config, jira, &query, &output_columns, matches.is_present("open"));
}

fn current(config: &Config, jira: &Jira, matches: &ArgMatches) {
    let query = format!(
        "project in ({}) AND assignee = {} AND status not in (Resolved, Closed)",
        config.projects(),
        config.username
    );
    let output_columns = ["key", "reporter", "status", "summary"];
    query_helper(config, jira, &query, &output_columns, matches.is_present("open"));
}

fn next(config: &Config, jira: &Jira, matches: &ArgMatches) {
    let query = format!(
        "project in ({}) AND status = Open AND assignee in ({})",
        config.projects(),
        config.npc_users()
    );
    let output_columns = ["key", "reporter", "summary"];
    query_helper(config, jira, &query, &output_columns, matches.is_present("open"));
}

fn new(config: &Config, jira: &Jira, matches: &ArgMatches, debug: bool) {
    let subcmd = match matches.subcommand_matches("new") {
        Some(matches) => matches,
        None => util::exit("this should not be possible"),
    };

    let project = subcmd
        .value_of("project")
        .unwrap_or(config.defaults.project_key.as_str());
    let summary = subcmd.value_of("summary").unwrap();
    let assignee = subcmd
        .value_of("assignee")
        .unwrap_or(config.defaults.assignee.as_str());

    let labels = match subcmd.values_of_lossy("label") {
        Some(labels) => labels,
        None => config.defaults.labels.to_owned(),
    };

    let mut description = subcmd.value_of("description").unwrap_or("").to_string();
    if subcmd.is_present("long_description") {
        description = match Prompt::new().execute() {
            Err(why) => util::exit(&format!("Failed to get description from editor: {}", why)),
            Ok(description) => description,
        };
    }

    let issue = match jira.create_issue(project, summary, description.as_str(), assignee, &labels, debug) {
        Err(why) => util::exit(&format!("Error creating issue \"{}\": {}", summary, why)),
        Ok(issue) => issue,
    };

    issue.print_tty(false);

    if config.open_in_browser {
        util::open_in_browser(config, &issue)
    }
}

fn jql(config: &Config, jira: &Jira, matches: &ArgMatches) {
    let subcmd = match matches.subcommand_matches("jql") {
        Some(matches) => matches,
        None => util::exit("this should not be possible"),
    };

    let query = subcmd.value_of("query").unwrap();
    let issues = util::perform_query(jira, query);
    util::render_issues(&issues, |result| {
        if subcmd.is_present("url") {
            result.as_filtered_table(&["key", "browse_url"])
        } else {
            result.as_table()
        }
    });

    if matches.is_present("open") {
        let issue = util::prompt_for_issue(&issues);
        util::open_in_browser(config, issue);
    }
}
