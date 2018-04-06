use hyper::Url;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format;
use std::collections::HashMap;
use std::fmt;

use jira_data::{IssueResponse, IssueResponseList};

pub struct Issue {
    pub self_url: String,
    pub key: String,
    pub summary: String,
    pub status: String,
    pub assignee: String,
    pub reporter: String,
    pub labels: Vec<String>,
    pub browse_url: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} assigned: {} status: {}",
            self.key, self.summary, self.assignee, self.status
        )
    }
}

impl Issue {
    pub fn from_issue_response(response: &IssueResponse) -> Self {
        let browse_url = match Url::parse(&response.self_url) {
            Ok(mut url) => {
                url.set_path(&format!("browse/{}", response.key.clone()));
                url.into_string()
            }
            Err(_) => String::new(),
        };

        Issue {
            self_url: response.self_url.clone(),
            key: response.key.clone(),
            summary: response.fields.summary.clone(),
            status: match response.fields.status {
                Some(ref status) => status.name.clone(),
                None => "Unknown".to_string(),
            },
            assignee: response.fields.assignee.display_name(),
            reporter: match response.fields.reporter {
                Some(ref reporter) => reporter.display_name(),
                None => "Unknown".to_string(),
            },
            labels: response.fields.labels.clone(),
            browse_url: browse_url,
        }
    }

    pub fn issues_from_response(response_list: &IssueResponseList) -> IssueVec {
        IssueVec(
            response_list
                .issues
                .iter()
                .rev()
                .map(|elem| Self::from_issue_response(elem))
                .collect(),
        )
    }

    pub fn as_hash_map(&self) -> HashMap<&str, String> {
        let labels = self.labels.join(", ");
        let mut map = HashMap::new();
        map.insert("self_url", self.self_url.clone());
        map.insert("key", self.key.clone());
        map.insert("summary", self.summary.clone());
        map.insert("status", self.status.clone());
        map.insert("assignee", self.assignee.clone());
        map.insert("reporter", self.reporter.clone());
        map.insert("labels", labels);
        map.insert("browse_url", self.browse_url.clone());
        map
    }

    pub fn print_tty(&self, force_colorize: bool) {
        let mut table = Table::new();

        let format = format::FormatBuilder::new().padding(1, 1).build();

        table.set_format(format);

        table.add_row(Row::new(vec![
            Cell::new("Summary"),
            Cell::new(self.summary.as_str()).style_spec("b"),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Url"),
            Cell::new(self.browse_url.as_str()),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Key"),
            Cell::new(self.key.as_str()),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Status"),
            Cell::new(self.status.as_str()),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Reporter"),
            Cell::new(self.reporter.as_str()),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Assignee"),
            Cell::new(self.assignee.as_str()),
        ]));

        if !self.labels.is_empty() {
            table.add_row(Row::new(vec![
                Cell::new("Labels"),
                Cell::new(self.labels.join(", ").as_str()),
            ]));
        }

        table.print_tty(force_colorize)
    }
}

pub struct IssueVec(Vec<Issue>);

impl fmt::Display for IssueVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_table().fmt(f)
    }
}

impl IssueVec {
    pub fn as_table(&self) -> Table {
        self.as_filtered_table(&["key", "reporter", "assignee", "status", "summary", "labels"])
    }

    pub fn as_filtered_table(&self, fields: &[&str]) -> Table {
        let mut table = Table::new();

        let format = format::FormatBuilder::new()
            .padding(1, 1)
            .separator(
                format::LinePosition::Title,
                format::LineSeparator::new('-', '-', '-', '-'),
            )
            .build();

        table.set_format(format);

        let mut titles = Vec::new();
        titles.push(Cell::new("#"));
        for key in fields {
            titles.push(Cell::new(key));
        }
        table.set_titles(Row::new(titles));

        for (i, issue) in self.0.iter().enumerate() {
            let hash_map = issue.as_hash_map();
            let mut row = Vec::new();
            row.push(Cell::new(&format!("{}", i + 1)));
            for key in fields {
                let val = match hash_map.get(key) {
                    Some(val) => val,
                    None => "<key missing>",
                };

                row.push(Cell::new(val));
            }

            table.add_row(Row::new(row));
        }
        table
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, idx: usize) -> Option<&Issue> {
        self.0.get(idx)
    }
}
