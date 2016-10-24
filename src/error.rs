use hyper;
use std;
use std::io;
use std::fmt;
use url::ParseError;
use rustc_serialize::json::{BuilderError, EncoderError};
use yaml_rust::scanner::ScanError;

#[derive(Debug)]
pub enum ObError {
    IoError(io::Error),
    ScanError(ScanError),
    ParseError(ParseError),
    BuilderError(BuilderError),
    EncoderError(EncoderError),
    RequestError(hyper::error::Error),
    Unexpected(String),
    InvalidConfig,
}

impl From<io::Error> for ObError {
    fn from(err: io::Error) -> ObError {
        ObError::IoError(err)
    }
}

impl From<ScanError> for ObError {
    fn from(err: ScanError) -> ObError {
        ObError::ScanError(err)
    }
}

impl From<ParseError> for ObError {
    fn from(err: ParseError) -> ObError {
        ObError::ParseError(err)
    }
}

impl From<BuilderError> for ObError {
    fn from(err: BuilderError) -> ObError {
        ObError::BuilderError(err)
    }
}

impl From<EncoderError> for ObError {
    fn from(err: EncoderError) -> ObError {
        ObError::EncoderError(err)
    }
}

impl From<hyper::error::Error> for ObError {
    fn from(err: hyper::error::Error) -> ObError {
        ObError::RequestError(err)
    }
}

impl From<String> for ObError {
    fn from(err: String) -> ObError {
        ObError::Unexpected(err)
    }
}

impl fmt::Display for ObError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ObError::IoError(ref err) => err.fmt(f),
            ObError::ScanError(ref err) => err.fmt(f),
            ObError::ParseError(ref err) => err.fmt(f),
            ObError::BuilderError(ref err) => err.fmt(f),
            ObError::EncoderError(ref err) => err.fmt(f),
            ObError::RequestError(ref err) => err.fmt(f),
            ObError::InvalidConfig => "Configuration file exists but is invalid".fmt(f),
            ObError::Unexpected(ref err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ObError {
    fn description(&self) -> &str {
        match *self {
            ObError::IoError(ref err) => err.description(),
            ObError::ScanError(ref err) => err.description(),
            ObError::ParseError(ref err) => err.description(),
            ObError::BuilderError(ref err) => err.description(),
            ObError::EncoderError(ref err) => err.description(),
            ObError::RequestError(ref err) => err.description(),
            ObError::InvalidConfig => "Invalid config",
            ObError::Unexpected(ref err) => err.as_str(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            ObError::IoError(ref err) => err.cause(),
            ObError::ScanError(ref err) => err.cause(),
            ObError::ParseError(ref err) => err.cause(),
            ObError::BuilderError(ref err) => err.cause(),
            ObError::EncoderError(ref err) => err.cause(),
            ObError::RequestError(ref err) => err.cause(),
            ObError::InvalidConfig => None,
            ObError::Unexpected(_) => None,
        }
    }
}
