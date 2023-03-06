use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Index is corrupt: {problem}")]
    Corrupt {
        problem: String,
    },
    #[error("Unsupported index version {0}")]
    UnsupportedVersion(u32),
    #[error("There are uncommited changes in the index or working directory")]
    UncommittedChanges,
}
