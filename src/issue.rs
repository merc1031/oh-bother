use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;
use prettytable::format;
use rustc_serialize::json::Json;
use std::collections::HashMap;
use std::fmt;
use itertools;

fn extract_string(data: &Json, path: &[&str]) -> String {
    match data.find_path(path) {
        // unwrap should be safe because we check first
        Some(obj) if obj.is_string() => obj.as_string().unwrap().to_string(),
        _ => "unknown".to_string(),
    }
}

fn extract_string_array(data: &Json, path: &[&str]) -> Vec<String> {
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

pub struct Issue {
    self_url: String,
    key: String,
    summary: String,
    status: String,
    assignee: String,
    reporter: String,
    labels: Vec<String>,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}: {} assigned: {} status: {}",
               self.key,
               self.summary,
               self.assignee,
               self.status)
    }
}

impl Issue {
    pub fn from_data(data: &Json) -> Self {
        Issue {
            self_url: extract_string(data, &["self"]),
            key: extract_string(data, &["key"]),
            summary: extract_string(data, &["fields", "summary"]),
            status: extract_string(data, &["fields", "status", "name"]),
            assignee: extract_string(data, &["fields", "assignee", "displayName"]),
            reporter: extract_string(data, &["fields", "reporter", "displayName"]),
            labels: extract_string_array(data, &["fields", "labels"]),
        }
    }

    pub fn issues_from_response(data: &Json) -> Option<IssueVec> {
        if let Some(ref raw_issues) = data.find("issues") {
            if raw_issues.is_array() {
                let issues: Vec<Self> = raw_issues.as_array()
                    .unwrap() // unwrap should be safe because we check first
                    .iter()
                    .rev()
                    .map(|elem| Self::from_data(elem))
                    .collect();
                return Some(IssueVec(issues));
            }
        }
        None
    }

    pub fn as_hash_map(&self) -> HashMap<&str, String> {
        let labels = itertools::join(self.labels.clone(), ", ");
        let mut map = HashMap::new();
        map.insert("self_url", self.self_url.clone());
        map.insert("key", self.key.clone());
        map.insert("summary", self.summary.clone());
        map.insert("status", self.status.clone());
        map.insert("assignee", self.assignee.clone());
        map.insert("reporter", self.reporter.clone());
        map.insert("labels", labels);
        map
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
            .separator(format::LinePosition::Title,
                       format::LineSeparator::new('-', '-', '-', '-'))
            .build();

        table.set_format(format);

        let mut titles = Vec::new();
        for key in fields {
            titles.push(Cell::new(key));
        }
        table.set_titles(Row::new(titles));

        for issue in &self.0 {
            let hash_map = issue.as_hash_map();
            let mut row = Vec::new();
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
}
