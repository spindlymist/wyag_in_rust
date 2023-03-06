use std::{
    error,
    io,
    fmt,
    str,
    string,
};

use crate::object::{ObjectHash, ObjectFormat};

pub type Result<T> = anyhow::Result<T>;

#[derive(Debug)]
pub enum Error {
    // object
    InvalidObjectHeader(String),
    UnrecognizedObjectFormat,
    BadObjectId,
    AmbiguousObjectId(Vec<ObjectHash>),
    UnexpectedObjectFormat(ObjectFormat),

    // hash
    InvalidObjectHash,
    
    // kvlm
    BadKVLMFormat,

    // meta
    MissingConfig(String),

    // commit
    NonCommitInGraph,
    BadCommitFormat,

    // tree
    ObjectNotTree,
    BadTreeFormat,

    // index
    BadIndexFormat(String),
    IndexHasExtensions,
    UncommittedChanges,

    // common
    Io(io::Error),
    Ini(ini::Error),
    Utf8(str::Utf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<ini::Error> for Error {
    fn from(value: ini::Error) -> Self {
        Error::Ini(value)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(value: str::Utf8Error) -> Self {
        Error::Utf8(value)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(value: string::FromUtf8Error) -> Self {
        Error::Utf8(value.utf8_error())
    }
}
