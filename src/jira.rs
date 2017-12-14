use hyper::Client;
use hyper::Url;
use hyper::client::{IntoUrl, RequestBuilder};
use hyper::header::{Headers, Authorization, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::json::Json;
use rustc_serialize::json;
use std::io::Read;

use error::ObError;
use issue::{Issue, IssueVec};

#[derive(RustcDecodable, RustcEncodable)]
struct JQLQuery {
    jql: String,
    fields: Vec<String>,
    maxResults: u16,
}

impl JQLQuery {
    pub fn new(query: &str) -> JQLQuery {
        JQLQuery {
            jql: query.to_string(),
            fields: vec!["summary".to_string(),
                         "status".to_string(),
                         "assignee".to_string(),
                         "reporter".to_string(),
                         "labels".to_string()],
            maxResults: 200,
        }
    }
}


#[derive(RustcDecodable, RustcEncodable)]
struct CreateIssueRequest {
    fields: IssueFields,
}

#[derive(RustcDecodable, RustcEncodable)]
struct IssueFields {
    summary: String,
    description: String,
    assignee: AssigneeFields,
    labels: Vec<String>,
    project: ProjectFields,
    issuetype: IssueTypeFields,
}

#[derive(RustcDecodable, RustcEncodable)]
struct AssigneeFields {
    name: String,
}

#[derive(RustcDecodable, RustcEncodable)]
struct ProjectFields {
    key: String,
}

#[derive(RustcDecodable, RustcEncodable)]
struct IssueTypeFields {
    name: String,
}

impl CreateIssueRequest {
    pub fn new(project_key: &str,
               summary: &str,
               description: &str,
               assignee: &str,
               labels: &Vec<String>)
               -> Self {
        CreateIssueRequest {
            fields: IssueFields {
                summary: summary.to_string(),
                description: description.to_string(),
                assignee: AssigneeFields { name: assignee.to_string() },
                labels: labels.clone(),
                project: ProjectFields { key: project_key.to_string() },
                issuetype: IssueTypeFields { name: "Bug".to_string() },
            },
        }
    }
}

struct AuthedClient {
    client: Client,
    headers: Headers,
}

impl AuthedClient {
    pub fn new(auth: &str) -> AuthedClient {
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Basic {}", auth).to_owned()));
        headers.set(ContentType(Mime(TopLevel::Application,
                                     SubLevel::Json,
                                     vec![(Attr::Charset, Value::Utf8)])));
        AuthedClient {
            client: Client::new(),
            headers: headers,
        }
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.post(url).headers(self.headers.clone())
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.get(url).headers(self.headers.clone())
    }
}

type JiraResult<T> = Result<T, ObError>;

pub struct Jira {
    client: AuthedClient,
    base_url: Url,
    debug: bool
}

impl Jira {
    pub fn new(auth: &str, base_url: &str, debug: bool) -> JiraResult<Jira> {
        let url = try!(Url::parse(base_url));
        Ok(Jira {
            client: AuthedClient::new(auth),
            base_url: url,
            debug: debug
        })
    }

    pub fn query(&self, query: &str) -> JiraResult<Option<IssueVec>> {
        let url = try!(self.base_url.join("rest/api/2/search"));
        let q = JQLQuery::new(query);
        let body = try!(json::encode(&q));
        let mut res = try!(self.client.post(url).body(body.as_str()).send());
        let mut response_body = String::new();
        try!(res.read_to_string(&mut response_body));
        let data = try!(Json::from_str(response_body.as_str()));
        Ok(Issue::issues_from_response(&data))
    }

    pub fn create_issue(&self,
                        project_key: &str,
                        summary: &str,
                        description: &str,
                        assignee: &str,
                        labels: &Vec<String>)
                        -> JiraResult<Option<Issue>> {
        let url = try!(self.base_url.join("rest/api/2/issue"));
        let request = CreateIssueRequest::new(project_key, summary, description, assignee, labels);
        let body = try!(json::encode(&request));

        if self.debug {
            println!("{}", body.as_str());
        }

        let mut res = try!(self.client.post(url).body(body.as_str()).send());
        let mut response_body = String::new();
        try!(res.read_to_string(&mut response_body));
        let data = try!(Json::from_str(response_body.as_str()));

        if let Some(obj) = data.find_path(&["key"]) {
            let issue_key = obj.as_string().unwrap();
            return self.issue(issue_key);
        } else {
            Err(ObError::Unexpected(format!("Jira response did not contain new issue key: {}",
                                            response_body.as_str())))
        }
    }

    pub fn issue(&self, issue_key: &str) -> JiraResult<Option<Issue>> {
        let url = try!(self.base_url.join(&format!("rest/api/2/issue/{}", issue_key)));
        let mut res = try!(self.client.get(url).send());
        let mut response_body = String::new();
        try!(res.read_to_string(&mut response_body));
        let data = try!(Json::from_str(response_body.as_str()));
        Ok(Some(Issue::from_data(&data)))
    }
}
