use std::collections::BTreeMap;
use serde::{Deserialize, Deserializer};

#[derive(Serialize, Debug, PartialEq)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct AuthResponse {
    pub session: Session,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Session {
    pub name: String,
    pub value: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct JQLQuery {
    jql: String,
    fields: Vec<String>,
    maxResults: u16,
}

impl JQLQuery {
    pub fn new(query: &str) -> JQLQuery {
        JQLQuery {
            jql: query.to_string(),
            fields: vec![
                "summary".to_string(),
                "status".to_string(),
                "assignee".to_string(),
                "reporter".to_string(),
                "labels".to_string(),
            ],
            maxResults: 200,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CreateIssueRequest {
    fields: IssueFields,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CreateIssueResponse {
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IssueResponse {
    pub fields: IssueFields,
    pub key: String,
    #[serde(rename = "self")] pub self_url: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IssueResponseList {
    pub issues: Vec<IssueResponse>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IssueFields {
    pub summary: String,
    #[serde(default = "default_description", deserialize_with = "nullable_description")] pub description: String,
    #[serde(default = "default_user", deserialize_with = "nullable_user_fields")] pub assignee: UserFields,
    pub labels: Vec<String>,
    #[serde(default = "default_project")] pub project: ProjectFields,
    #[serde(default = "default_issuetype")] pub issuetype: IssueTypeFields,
    #[serde(skip_serializing)] pub reporter: Option<UserFields>,
    #[serde(skip_serializing)] pub status: Option<Status>,
    #[serde(skip_deserializing, flatten)] pub extra: BTreeMap<String, SelectListFields>,
}

fn nullable_user_fields<'de, D>(deserializer: D) -> Result<UserFields, D::Error>
    where D: Deserializer<'de>
{
    let opt = Option::deserialize(deserializer)?;
    match opt {
        Some(o) => Ok(o),
        None => Ok(default_user()),
    }
}

fn default_user() -> UserFields {
    UserFields {
        name: "<unknown>".to_string(),
        displayName: None,
    }
}

fn nullable_description<'de, D>(deserializer: D) -> Result<String, D::Error>
    where D: Deserializer<'de>
{
    let opt = Option::deserialize(deserializer)?;
    match opt {
        Some(o) => Ok(o),
        None => Ok(default_description()),
    }
}

fn default_description() -> String {
    "Unknown".to_string()
}

fn default_project() -> ProjectFields {
    ProjectFields {
        key: "Unknown".to_string(),
    }
}

fn default_issuetype() -> IssueTypeFields {
    IssueTypeFields {
        name: "Unknown".to_string(),
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UserFields {
    pub name: String,
    #[serde(skip_serializing)] displayName: Option<String>,
}

impl UserFields {
    pub fn display_name(&self) -> String {
        match self.displayName {
            Some(ref name) => name.clone(),
            None => "Unknown".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Status {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ProjectFields {
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct IssueTypeFields {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SelectListFields {
    pub id: String,
}

impl CreateIssueRequest {
    pub fn new(
        project_key: &str,
        issue_type: &str,
        summary: &str,
        description: &str,
        assignee: &str,
        labels: &Vec<String>,
        extra_fields: &BTreeMap<String, String>,
    ) -> Self {
        CreateIssueRequest {
            fields: IssueFields {
                assignee: UserFields {
                    name: assignee.to_string(),
                    displayName: None,
                },
                description: description.to_string(),
                issuetype: IssueTypeFields {
                    name: issue_type.to_string(),
                },
                labels: labels.clone(),
                project: ProjectFields {
                    key: project_key.to_string(),
                },
                reporter: None,
                status: None,
                summary: summary.to_string(),
                extra: extra_fields.clone().into_iter().map(|(k, x)| (k, SelectListFields { id: x })).collect::<BTreeMap<_,_>>(),
            },
        }
    }
}
