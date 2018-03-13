use hyper::Client;
use hyper::Url;
use hyper::client::{IntoUrl, RequestBuilder};
use hyper::header::{Headers, ContentType, Cookie, CookiePair};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::base64::FromBase64;
use rustc_serialize::json::Json;
use rustc_serialize::json;
use std::io::Read;

use util;
use error::{ErrorKind, Result};
use issue::{Issue, IssueVec};

#[derive(RustcDecodable, RustcEncodable)]
struct AuthRequest {
    username: String,
    password: String,
}

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
    pub fn new(auth: &str, base_url: &Url) -> Result<AuthedClient> {
        let client = Client::new();
        let auth_url = base_url.join("rest/auth/1/session")?;

        let bytes = auth.from_base64()?;
        let whole_key =  String::from_utf8(bytes)?;
        let mut splitter = whole_key.split(':');
        let username = splitter.next().unwrap().to_string();
        let password = splitter.fold("".to_string(), |a, b| a + b);

        let auth_body = AuthRequest {
            username: username,
            password: password,
        };

        let mut headers = Headers::new();
        headers.set(ContentType(Mime(TopLevel::Application,
                                     SubLevel::Json,
                                     vec![(Attr::Charset, Value::Utf8)])));

        let body = json::encode(&auth_body)?;
        let mut res = client.post(auth_url).headers(headers.clone()).body(body.as_str()).send()?;
        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        let data = Json::from_str(response_body.as_str())?;
        let name = util::extract_string(&data, &["session", "name"]);
        let value = util::extract_string(&data, &["session", "value"]);

        headers.set(
            Cookie(vec![CookiePair::new(name, value)])
        );


        Ok(AuthedClient {
            client: client,
            headers: headers,
        })
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.post(url).headers(self.headers.clone())
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.get(url).headers(self.headers.clone())
    }
}

pub struct Jira {
    client: AuthedClient,
    base_url: Url,
    debug: bool
}

impl Jira {
    pub fn new(auth: &str, base_url: &str, debug: bool) -> Result<Jira> {
        let url = Url::parse(base_url)?;
        let client = AuthedClient::new(auth, &url)?;
        Ok(Jira {
            client: client,
            base_url: url,
            debug: debug
        })
    }

    pub fn query(&self, query: &str) -> Result<Option<IssueVec>> {
        let url = self.base_url.join("rest/api/2/search")?;
        let q = JQLQuery::new(query);
        let body = json::encode(&q)?;
        let mut res = self.client.post(url).body(body.as_str()).send()?;
        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        let data = Json::from_str(response_body.as_str())?;
        Ok(Issue::issues_from_response(&data))
    }

    pub fn create_issue(&self,
                        project_key: &str,
                        summary: &str,
                        description: &str,
                        assignee: &str,
                        labels: &Vec<String>)
                        -> Result<Option<Issue>> {
        let url = self.base_url.join("rest/api/2/issue")?;
        let request = CreateIssueRequest::new(project_key, summary, description, assignee, labels);
        let body = json::encode(&request)?;

        if self.debug {
            println!("{}", body.as_str());
        }

        let mut res = self.client.post(url).body(body.as_str()).send()?;
        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        let data = Json::from_str(response_body.as_str())?;

        if let Some(obj) = data.find_path(&["key"]) {
            let issue_key = obj.as_string().unwrap();
            return self.issue(issue_key);
        } else {
            Err(ErrorKind::Unexpected(format!("Jira response did not contain new issue key: {}",
                                            response_body.as_str())).into())
        }
    }

    pub fn issue(&self, issue_key: &str) -> Result<Option<Issue>> {
        let url = self.base_url.join(&format!("rest/api/2/issue/{}", issue_key))?;
        let mut res = self.client.get(url).send()?;
        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        let data = Json::from_str(response_body.as_str())?;
        Ok(Some(Issue::from_data(&data)))
    }
}
