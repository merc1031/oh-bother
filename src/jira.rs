use base64::decode;
use hyper::Client;
use hyper::Url;
use hyper::client::{IntoUrl, RequestBuilder};
use hyper::header::{ContentType, Cookie, CookiePair, Headers};
use hyper::mime::{Attr, Mime, SubLevel, TopLevel, Value};
use serde_json;
use std::io::Read;

use error::{ErrorKind, Result};
use issue::{Issue, IssueVec};
use jira_data::{AuthRequest, AuthResponse, CreateIssueRequest, CreateIssueResponse, IssueResponse,
                JQLQuery};

struct AuthedClient {
    client: Client,
    headers: Headers,
}

impl AuthedClient {
    pub fn new(auth: &str, base_url: &Url) -> Result<AuthedClient> {
        let client = Client::new();
        let auth_url = base_url.join("rest/auth/1/session")?;

        let whole_key = String::from_utf8(decode(auth)?)?;
        let mut splitter = whole_key.split(':');
        let username = splitter.next().unwrap().to_string();
        let password = splitter.fold("".to_string(), |a, b| a + b);

        let auth_body = AuthRequest {
            username: username,
            password: password,
        };

        let mut headers = Headers::new();
        headers.set(ContentType(Mime(
            TopLevel::Application,
            SubLevel::Json,
            vec![(Attr::Charset, Value::Utf8)],
        )));

        let body = serde_json::to_string(&auth_body)?;
        let mut res = client
            .post(auth_url)
            .headers(headers.clone())
            .body(body.as_str())
            .send()?;
        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        let auth_response: AuthResponse = serde_json::from_str(response_body.as_str())?;

        headers.set(Cookie(vec![
            CookiePair::new(auth_response.session.name, auth_response.session.value),
        ]));

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
    debug: bool,
}

impl Jira {
    pub fn new(auth: &str, base_url: &str, debug: bool) -> Result<Jira> {
        let url = Url::parse(base_url)?;
        let client = AuthedClient::new(auth, &url)?;
        Ok(Jira {
            client: client,
            base_url: url,
            debug: debug,
        })
    }

    pub fn query(&self, query: &str) -> Result<IssueVec> {
        let url = self.base_url.join("rest/api/2/search")?;
        let q = JQLQuery::new(query);
        let body = serde_json::to_string(&q)?;
        let mut res = self.client.post(url).body(body.as_str()).send()?;
        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        let data = serde_json::from_str(response_body.as_str())?;
        Ok(Issue::issues_from_response(&data))
    }

    pub fn create_issue(
        &self,
        project_key: &str,
        issue_type: &str,
        summary: &str,
        description: &str,
        assignee: &str,
        labels: &Vec<String>,
        debug: bool
    ) -> Result<Issue> {
        let url = self.base_url.join("rest/api/2/issue")?;
        let request = CreateIssueRequest::new(project_key, issue_type, summary, description, assignee, labels);
        let body = serde_json::to_string(&request)?;

        if self.debug {
            println!("{}", body.as_str());
        }

        let mut res = self.client.post(url).body(body.as_str()).send()?;
        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        if debug {
            println!("{}", response_body);
        }
        let response: serde_json::Result<CreateIssueResponse> =
            serde_json::from_str(response_body.as_str());
        match response {
            Ok(r) => self.issue(&r.key),
            Err(_) => {
                return Err(ErrorKind::Unexpected(format!(
                    "Jira response did not contain new issue key: {}",
                    response_body.as_str()
                )).into())
            }
        }
    }

    pub fn issue(&self, issue_key: &str) -> Result<Issue> {
        let url = self.base_url
            .join(&format!("rest/api/2/issue/{}", issue_key))?;
        let mut res = self.client.get(url).send()?;
        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;
        let response: serde_json::Result<IssueResponse> =
            serde_json::from_str(response_body.as_str());
        match response {
            Ok(r) => Ok(Issue::from_issue_response(&r)),
            Err(e) => Err(ErrorKind::Unexpected(format!("Issue {} not found {}", issue_key, e)).into()),
        }
    }
}
