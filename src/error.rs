use std::convert;
use std::error;
use std::fmt;
use std::io;
use std::process::{ExitStatus};

use eventual;
use glob;

use entry;
use formatter;
use parser;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    FormatError(formatter::Error),
    ParseError(parser::Error),
    EntryMapError(entry::Error),
    AsyncError(eventual::AsyncError<()>),
    PatternError(glob::PatternError),
    CmdFailure(ExitStatus)
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::IoError(ref e) => e.fmt(fmt),
            Error::FormatError(ref e) => e.fmt(fmt),
            Error::ParseError(ref e) => e.fmt(fmt),
            Error::EntryMapError(ref e) => e.fmt(fmt),
            Error::AsyncError(ref e) => write!(fmt, "{:?}", e),
            Error::PatternError(ref e) => e.fmt(fmt),
            Error::CmdFailure(code) =>
                write!(fmt, "Command exited with nonzero code: {}", code)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref e) => e.description(),
            Error::FormatError(ref e) => e.description(),
            Error::ParseError(ref e) => e.description(),
            Error::EntryMapError(ref e) => e.description(),
            Error::PatternError(ref e) => e.description(),
            Error::AsyncError(_) => "aborted",
            Error::CmdFailure(_) => "Command exited with nonzero code"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref e) => Some(e),
            Error::FormatError(ref e) => Some(e),
            Error::ParseError(ref e) => Some(e),
            Error::EntryMapError(ref e) => Some(e),
            Error::PatternError(ref e) => Some(e),
            Error::AsyncError(_) => None,
            Error::CmdFailure(_) => None
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}

impl convert::From<formatter::Error> for Error {
    fn from(e: formatter::Error) -> Self {
        Error::FormatError(e)
    }
}

impl convert::From<parser::Error> for Error {
    fn from(e: parser::Error) -> Self {
        Error::ParseError(e)
    }
}

impl convert::From<entry::Error> for Error {
    fn from(e: entry::Error) -> Self {
        Error::EntryMapError(e)
    }
}

impl convert::From<eventual::AsyncError<()>> for Error {
    fn from(x: eventual::AsyncError<()>) -> Self {
        Error::AsyncError(x)
    }
}

impl convert::From<glob::PatternError> for Error {
    fn from(glob: glob::PatternError) -> Self {
        Error::PatternError(glob)
    }
}
