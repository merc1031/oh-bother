
use url::ParseError;
use hyper::Url;
use hyper::Client;
use hyper::client::{IntoUrl, RequestBuilder};
use hyper::header::{Headers, Authorization, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};

use rustc_serialize::{Encodable, json};

use std::io::Read;

#[derive(RustcDecodable, RustcEncodable)]
struct JQLQuery {
    jql: String,
    fields: Vec<String>,
}

impl JQLQuery {
    pub fn new(query: &str) -> JQLQuery {
        JQLQuery {
            jql: query.to_string(),
            fields: vec!["summary".to_string(), "status".to_string(), "assignee".to_string()],
        }
    }
}

struct AuthedClient {
    client: Client,
    headers: Headers
}

impl AuthedClient {
    pub fn new(auth: &str) -> AuthedClient {
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Basic {}", auth).to_owned()));
        headers.set(ContentType(Mime(TopLevel::Application,
                                     SubLevel::Json,
                                     vec![(Attr::Charset, Value::Utf8)])));
        AuthedClient { client: Client::new(), headers: headers }
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.post(url)
    }
}

#[derive(Debug)]
pub enum JiraError {
    ParseError(ParseError),
    RequestError(hyper::error::Error)
}

type JiraResult<T> = Result<T, JiraError>;

impl From<hyper::errror::Error> for JiraError {
    fn from(err: hyper::error::Error) -> JiraError {
        JiraError::RequestError(err)
    }
}

pub struct Jira {
    client: AuthedClient,
    base_url: Url
}

impl Jira {
    pub fn new(auth: &str, base_url: &str) -> Result<Jira, ParseError> {
        let url = try!(Url::parse(base_url));
        Ok(Jira { client: AuthedClient::new(auth), base_url: url })
    }

    pub fn query(&self, query: &str) -> JiraResult<String> {
        let q = JQLQuery::new(query);
        let body = try!(json::encode(&q));
        let mut res = try!(self.client.post(url).body(body.as_str()).send());
    }
}
