use base64::DecodeError;
use clap;
use eprompt;
use hyper;
use serde_json;
use std::io;
use std;
use url::ParseError;
use yaml_rust::scanner::ScanError;

error_chain! {
    links {
        EPrompt(eprompt::Error, eprompt::ErrorKind);
    }

    foreign_links {
        Base64DecodeError(DecodeError);
        FromUtf8Error(std::string::FromUtf8Error);
        IoError(io::Error);
        JsonError(serde_json::Error);
        ParseError(ParseError);
        RequestError(hyper::error::Error);
        ScanError(ScanError);
    }

    errors {
        InvalidConfig {
            description("invalid config file")
            display("invalid config file")
        }
        Unexpected(message: String) {
            description("unexpected result")
            display("unexpected result: {}", message)
        }
    }
}

pub trait UnwrapOrExit<T>
where
    Self: Sized,
{
    fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T;

    fn unwrap_or_exit(self, message: &str) -> T {
        let err = clap::Error::with_description(message, clap::ErrorKind::InvalidValue);
        self.unwrap_or_else(|| err.exit())
    }
}

impl<T> UnwrapOrExit<T> for Option<T> {
    fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.unwrap_or_else(f)
    }
}

impl<T> UnwrapOrExit<T> for Result<T> {
    fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.unwrap_or_else(|_| f())
    }

    fn unwrap_or_exit(self, message: &str) -> T {
        self.unwrap_or_else(|e| {
            let err = clap::Error::with_description(
                &format!("{}: {}", message, e),
                clap::ErrorKind::InvalidValue,
            );
            err.exit()
        })
    }
}
