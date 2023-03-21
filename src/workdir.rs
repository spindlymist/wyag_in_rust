use std::{
    path::{Path, PathBuf},
    fs::{self, File, OpenOptions},
};
use path_absolutize::Absolutize;

use crate::Result;

mod error;
pub use error::WorkDirError;

mod workpath;
pub use workpath::WorkPath;
pub use workpath::WorkPathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkDir(PathBuf);

impl WorkDir {
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>
    {
        Ok(Self(
            path.as_ref().absolutize()?.into()
        ))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn is_valid_path<P>(path: P) -> Result<bool>
    where
        P: AsRef<Path>
    {
        if path.as_ref().is_file() {
            Ok(false)
        }
        else if path.as_ref().is_dir() {
            let is_empty = path.as_ref().read_dir()?.next().is_none();
            Ok(is_empty)
        }
        else {
            Ok(true)
        }
    }

    /// Translates a path within the repo to its canonical name.
    /// 
    /// The canonical name is relative to the working directory, uses `/` for the path separator,
    /// and does not begin or end with a slash.
    pub fn canonicalize_path<P>(&self, path: P) -> Result<WorkPathBuf>
    where
        P: AsRef<Path>
    {
        let abs_path = path.as_ref().absolutize()?;
        let rel_path = match abs_path.strip_prefix(&self.0) {
            Ok(val) => val,
            Err(_) => return Err(WorkDirError::OutsideWorkingDir(path.as_ref().to_owned()).into()),
        };

        WorkPathBuf::try_from(rel_path)
    }

    /// Appends a relative path to the repo's .git directory.
    pub fn git_path<P>(&self, rel_path: P) -> PathBuf
    where
        P: AsRef<Path>
    {
        let mut path = self.0.join(".git");
        path.push(rel_path);

        path
    }

    /// Opens a file in the repo's .git directory.
    pub fn open_git_file<P>(&self, rel_path: P, options: Option<&OpenOptions>) -> Result<File>
    where
        P: AsRef<Path>
    {    
        if let Some(parent_path) = rel_path.as_ref().parent() {
            self.make_git_dir(parent_path)?;
        }
        
        let abs_path = self.git_path(rel_path);

        if let Some(options) = options {
            Ok(options.open(abs_path)?)
        }
        else {
            Ok(File::open(abs_path)?)
        }
    }

    /// Creates a directory in the repo's .git directory.
    pub fn make_git_dir<P>(&self, rel_path: P) -> Result<PathBuf>
    where
        P: AsRef<Path>
    {
        let abs_path = self.git_path(rel_path);
        fs::create_dir_all(&abs_path)?;
        
        Ok(abs_path)
    }

    /// Removes the file or directory at `path` from the file system.
    pub fn remove_path(&self, path: &WorkPath) -> Result<()> {
        let abs_path = self.0.join(path);

        if path.is_empty() {
            // Delete everything except the .git directory (if present)
            // Note that any .git directories in subdirectories will be deleted
            for entry in abs_path.read_dir()? {
                let entry = entry?;
                let entry_path = entry.path();
                
                if entry_path.is_file() {
                    std::fs::remove_file(&entry_path)?;
                }
                else if entry_path.is_dir() && entry.file_name() != ".git" {
                    std::fs::remove_dir_all(&entry_path)?;
                }
            }
        }
        else if abs_path.is_dir() {
            std::fs::remove_dir_all(&abs_path)?;
        }
        else if abs_path.is_file() {
            std::fs::remove_file(&abs_path)?;
        }

        Ok(())
    }
}

impl TryFrom<PathBuf> for WorkDir {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self> {
        WorkDir::new(value)
    }
}

impl TryFrom<&Path> for WorkDir {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self> {
        WorkDir::new(value)
    }
}

impl TryFrom<String> for WorkDir {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        WorkDir::new(value)
    }
}

impl TryFrom<&str> for WorkDir {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        WorkDir::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalize_rel_path() {
        let wd = WorkDir::new("my_work_dir").unwrap();
        let path = wd.canonicalize_path("my_work_dir/src/main.rs").unwrap();
        assert_eq!(path.as_str(), "src/main.rs");
    }

    #[test]
    fn canonicalize_abs_path() {
        let wd = WorkDir::new(r"C:\my_work_dir").unwrap();
        let path = wd.canonicalize_path(r"C:\my_work_dir\src\main.rs").unwrap();
        assert_eq!(path.as_str(), "src/main.rs");
    }

    #[test]
    fn canonicalize_rejects_rel_path_outside_workdir() {
        let wd = WorkDir::new(r"my_work_dir").unwrap();
        let path = wd.canonicalize_path("src/main.rs");
        assert!(path.is_err());
    }

    #[test]
    fn canonicalize_rejects_abs_path_outside_workdir() {
        let wd = WorkDir::new(r"C:\my_work_dir").unwrap();
        let path = wd.canonicalize_path(r"C:\my_other_dir\src\main.rs");
        assert!(path.is_err());
    }

    #[test]
    fn git_path() {
        let wd = WorkDir::new(r"C:\my_work_dir").unwrap();
        let path = wd.git_path(r"refs/heads/main");
        let components: Vec<_> = path.components().collect();
        let expected_components: Vec<_> = {
            let expected_path: &Path = "C:/my_work_dir/.git/refs/heads/main".as_ref();
            expected_path.components().collect()
        };

        assert_eq!(components, expected_components);
    }
}
