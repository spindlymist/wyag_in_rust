use std::{path::PathBuf, ffi::OsString};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkDirError {
    #[error("Forbidden path component `{component}` in `{path}`")]
    ForbiddenComponent {
        path: PathBuf,
        component: String,
    },
    #[error("Invalid unicode in path `{0:?}`")]
    InvalidUnicode(OsString),
    #[error("Workpaths must be relative, but `{0:?}` is absolute")]
    AbsolutePath(PathBuf),
    #[error("The path `{0:?}` is outside of the working directory")]
    OutsideWorkingDir(PathBuf),
}
