use super::{ObjectFormat, ObjectHash};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObjectError {
    #[error("Malformed object header {hash}: {problem}")]
    MalformedHeader {
        hash: ObjectHash,
        problem: String,
    },
    #[error("Unrecognized object format `{0}`")]
    UnrecognizedFormat(String),
    #[error("Unexpected object format `{format}` (expected `{expected}`)")]
    UnexpectedFormat {
        format: ObjectFormat,
        expected: ObjectFormat,
    },
    #[error("The identifier `{0}` does not refer to an object")]
    InvalidId(String),
    #[error("The identifier `{id}` is ambiguous ({} matches)", matches.len())]
    AmbiguousId {
        id: String,
        matches: Vec<ObjectHash>,
    },
    #[error("Invalid hash `{hash_string}`: {problem}")]
    InvalidHashString {
        hash_string: String,
        problem: String,
    },
    #[error("Invalid hash bytes: {bytes:?}")]
    InvalidHashBytes {
        bytes: Vec<u8>,
    }
}
