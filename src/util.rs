extern crate clap;

use std::process::Command;

use prettytable::Table;

use config::Config;
use jira::Jira;
use issue::{Issue, IssueVec};

pub fn exit(message: &str) -> ! {
    let err = clap::Error::with_description(message, clap::ErrorKind::InvalidValue);
    err.exit();
}

pub fn perform_query(jira: &Jira, query: &str) -> IssueVec {
    let result = match jira.query(query) {
        Err(why) => exit(&format!("Error executing query {}: {}", query, why)),
        Ok(result) => result,
    };

    if result.is_empty() {
        exit(&format!("the query \"{}\" returned no issues", query));
    }

    result
}

pub fn render_issues<F>(issues: &IssueVec, table_fn: F)
where
    F: Fn(&IssueVec) -> Table,
{
    table_fn(issues).print_tty(false);
}

pub fn open_in_browser(config: &Config, issue: &Issue) {
    match Command::new(config.browser_command.as_str())
        .arg(&issue.browse_url)
        .output()
    {
        Err(why) => exit(&format!("Error opening in browser: {}", why)),
        _ => {}
    }
}
