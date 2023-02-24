use std::{
    error,
    io,
    fmt,
};

use crate::object::{ObjectHash, ObjectFormat};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    WorkingDirectoryInvalid,
    DirectoryNotInitialized,
    RepoFmtVersionMissing,
    UnsupportedRepoFmtVersion(String),
    InitPathIsFile,
    InitDirectoryNotEmpty,
    InvalidObjectHeader(String),
    InvalidObjectHash,
    UnrecognizedObjectFormat,
    NonCommitInGraph,
    ObjectNotTree,
    BadKVLMFormat,
    BadTreeFormat,
    BadCommitFormat,
    BadObjectId,
    AmbiguousObjectId(Vec<ObjectHash>),
    BadIndexFormat(String),
    IndexHasExtensions,
    HashMismatch,
    InvalidPath,
    InvalidRef,
    BranchAlreadyExists,
    UnrecognizedHeadRef,
    MissingConfig(String),
    BranchIsCheckedOut,
    BranchPossiblyUnmerged,
    UnexpectedObjectFormat(ObjectFormat),
    Io(io::Error),
    Ini(ini::Error),
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
