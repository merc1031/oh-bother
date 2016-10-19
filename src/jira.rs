
use hyper;
use hyper::Client;
use hyper::Url;
use hyper::client::{IntoUrl, RequestBuilder};
use hyper::header::{Headers, Authorization, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use rustc_serialize::json::{Json, BuilderError, EncoderError};
use rustc_serialize::json;
use std::fmt;
use std::io;
use std::io::Read;
use url::ParseError;

use issue::{Issue, IssueVec};

#[derive(RustcDecodable, RustcEncodable)]
struct JQLQuery {
    jql: String,
    fields: Vec<String>,
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

#[derive(Debug)]
pub enum JiraError {
    IoError(io::Error),
    ParseError(ParseError),
    BuilderError(BuilderError),
    EncoderError(EncoderError),
    RequestError(hyper::error::Error),
}

type JiraResult<T> = Result<T, JiraError>;

impl From<io::Error> for JiraError {
    fn from(err: io::Error) -> JiraError {
        JiraError::IoError(err)
    }
}

impl From<ParseError> for JiraError {
    fn from(err: ParseError) -> JiraError {
        JiraError::ParseError(err)
    }
}

impl From<BuilderError> for JiraError {
    fn from(err: BuilderError) -> JiraError {
        JiraError::BuilderError(err)
    }
}

impl From<EncoderError> for JiraError {
    fn from(err: EncoderError) -> JiraError {
        JiraError::EncoderError(err)
    }
}

impl From<hyper::error::Error> for JiraError {
    fn from(err: hyper::error::Error) -> JiraError {
        JiraError::RequestError(err)
    }
}

impl fmt::Display for JiraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JiraError::IoError(ref e) => e.fmt(f),
            JiraError::ParseError(ref e) => e.fmt(f),
            JiraError::BuilderError(ref e) => e.fmt(f),
            JiraError::EncoderError(ref e) => e.fmt(f),
            JiraError::RequestError(ref e) => e.fmt(f),
        }
    }
}

pub struct Jira {
    client: AuthedClient,
    base_url: Url,
}

impl Jira {
    pub fn new(auth: &str, base_url: &str) -> JiraResult<Jira> {
        let url = try!(Url::parse(base_url));
        Ok(Jira {
            client: AuthedClient::new(auth),
            base_url: url,
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
                        summary: &str,
                        assignee: &str,
                        labels: &[&str])
                        -> JiraResult<Option<Issue>> {
    }

    pub fn issue(&self, issue_key: &str) -> JiraResult<Option<Issue>> {
        let url = try!(self.base_url.join(&format!("rest/api/2/issue/{}", issue_key)));
        let mut res = try!(self.client.get(url).send());
        let mut response_body = String::new();
        try!(res.read_to_string(&mut response_body));
        let data = try!(Json::from_str(response_body.as_str()));
        Ok(Some(Issue::from_data(&data)))
    }

    pub fn browse_url_for(&self, issue: &Issue) -> Result<Url, ParseError> {
        self.base_url.join(&format!("browse/{}", issue.key))
    }
}
