use hyper;
use clap;
use eprompt;
use std;
use std::io;
use url::ParseError;
use rustc_serialize::json::{BuilderError, EncoderError};
use rustc_serialize::base64::FromBase64Error;
use yaml_rust::scanner::ScanError;

error_chain! {
    links {
        EPrompt(eprompt::Error, eprompt::ErrorKind);
    }

    foreign_links {
        IoError(io::Error);
        ScanError(ScanError);
        ParseError(ParseError);
        BuilderError(BuilderError);
        EncoderError(EncoderError);
        FromBase64Error(FromBase64Error);
        FromUtf8Error(std::string::FromUtf8Error);
        RequestError(hyper::error::Error);
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
    where Self: Sized
{
    fn unwrap_or_else<F>(self, f: F) -> T where F: FnOnce() -> T;

    fn unwrap_or_exit(self, message: &str) -> T {
        let err = clap::Error::with_description(message, clap::ErrorKind::InvalidValue);
        self.unwrap_or_else(|| err.exit())
    }
}

impl<T> UnwrapOrExit<T> for Option<T> {
    fn unwrap_or_else<F>(self, f: F) -> T
        where F: FnOnce() -> T
    {
        self.unwrap_or_else(f)
    }
}

impl<T> UnwrapOrExit<T> for Result<T> {
    fn unwrap_or_else<F>(self, f: F) -> T
        where F: FnOnce() -> T
    {
        self.unwrap_or_else(|_| f())
    }

    fn unwrap_or_exit(self, message: &str) -> T {
        self.unwrap_or_else(|e| {
            let err = clap::Error::with_description(&format!("{}: {}", message, e),
                                                    clap::ErrorKind::InvalidValue);
            err.exit()
        })
    }
}
