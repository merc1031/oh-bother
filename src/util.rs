extern crate clap;

use std::process::Command;

use rustc_serialize::json::Json;

use prettytable::Table;

use config::Config;
use jira::Jira;
use issue::{Issue, IssueVec};

pub fn exit(message: &str) -> ! {
    let err = clap::Error::with_description(message, clap::ErrorKind::InvalidValue);
    err.exit();
}

pub fn perform_query<F>(jira: &Jira, query: &str, table_fn: F)
    where F: Fn(IssueVec) -> Table
{
    let result = match jira.query(query) {
        Err(why) => exit(&format!("Error executing query {}: {}", query, why)),
        Ok(result) => result,
    };

    match result {
        Some(result) => table_fn(result).print_tty(false),
        None => println!("the query \"{}\" returned no issues", query),
    }
}

pub fn open_in_browser(config: &Config, jira: &Jira, issue: &Issue) {
    let url = match jira.browse_url_for(issue) {
        Err(why) => exit(&format!("Error making browse url: {}", why)),
        Ok(url) => url,
    };

    match Command::new(config.browser_command.as_str()).arg(url.as_str()).output() {
        Err(why) => exit(&format!("Error opening in browser: {}", why)),
        _ => {}
    }
}

pub fn extract_string(data: &Json, path: &[&str]) -> String {
    match data.find_path(path) {
        // unwrap should be safe because we check first
        Some(obj) if obj.is_string() => obj.as_string().unwrap().to_string(),
        _ => "unknown".to_string(),
    }
}

pub fn extract_string_array(data: &Json, path: &[&str]) -> Vec<String> {
    match data.find_path(path) {
        Some(obj) if obj.is_array() => {
            obj.as_array()
                .unwrap()
                .into_iter()
                .map(|elem| elem.as_string().unwrap().to_string())
                .collect()
        }
        _ => Vec::new(),
    }
}
