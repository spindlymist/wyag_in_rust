use std::{
    path::{Path, PathBuf},
    fs::{self, OpenOptions},
    io::Write,
};
use anyhow::Context;
use ini::Ini;
use path_absolutize::Absolutize;
use thiserror::Error;

use crate::{
    Result,
    workdir::WorkDir,
    index::Index,
    branch,
};

/// A Git repository.
pub struct Repository {
    workdir: WorkDir,
    config: Ini,
}

impl Repository {
    /// Initializes a new git repository in an empty directory.
    pub fn init<P>(dir: P) -> Result<Repository>
    where
        P: AsRef<Path>
    {
        let repo = {
            if !WorkDir::is_valid_path(&dir)? {
                return Err(RepoError::InitPathExists(dir.as_ref().to_owned()).into());
            }
            let workdir = WorkDir::new(dir)?;
            
            // Initialize config
            let mut config = Ini::new();
            config.with_section(Some("core"))
                .set("repositoryformatversion", "0")
                .set("filemode", "false")
                .set("bare", "false");
        
            Repository {
                workdir,
                config,
            }
        };
        
        // Create directories
        fs::create_dir_all(repo.workdir.git_path("."))?;
        repo.workdir.make_git_dir("objects")?;
        repo.workdir.make_git_dir("refs/tags")?;
        repo.workdir.make_git_dir("refs/heads")?;

        // Create files
        {
            let mut options = OpenOptions::new();
            options
                .create(true)
                .append(true);

            repo.workdir.open_git_file("description", Some(&options))?
                .write_all(b"Unnamed repository; edit this file 'description' to name the repository.\n")?;

            repo.workdir.open_git_file("HEAD", Some(&options))?
                .write_all(b"ref: refs/heads/master\n")?;

            let mut config_file = repo.workdir.open_git_file("config", Some(&options))?;
            repo.config.write_to(&mut config_file)?;
        }

        Ok(repo)
    }

    /// Constructs a `Repository` from the repo in an existing directory.
    pub fn from_existing<P>(dir: P) -> Result<Repository>
    where
        P: AsRef<Path>
    {
        if !dir.as_ref().is_dir() {
            return Err(RepoError::UninitializedDirectory(dir.as_ref().to_owned()).into());
        }
        let workdir = WorkDir::new(dir)?;

        let config_file = workdir.git_path("config");
        let config = Ini::load_from_file(config_file)?;

        match config.get_from(Some("core"), "repositoryformatversion") {
            Some("0") => (),
            Some(version) => return Err(RepoError::FmtVersionUnsupported(version.to_owned()).into()),
            None => return Err(RepoError::FmtVersionMissing.into()),
        };

        Ok(Repository {
            workdir,
            config,
        })
    }

    /// Finds the git repository that contains `path` (if it exists).
    pub fn find<P>(path: P) -> Result<Repository>
    where
        P: AsRef<Path>
    {
        let abs_path = path.as_ref().absolutize()?;

        // The existence of a .git directory is considered sufficient
        // evidence of a repository
        if abs_path.join(".git").is_dir() {
            return Repository::from_existing(&abs_path);
        }

        // Recurse up the directory tree
        if let Some(parent_path) = abs_path.parent() {
            Repository::find(parent_path)
        }
        else {
            // Reached root without finding a .git directory
            Err(RepoError::UninitializedDirectory(path.as_ref().to_owned()).into())
        }
    }

    /// Parses (or creates) the repo's index.
    pub fn index(&self) -> Result<Index> {
        let index_path = self.workdir.git_path("index");

        if index_path.is_file() {
            let file = std::fs::File::open(&index_path)
                .with_context(|| format!("Failed to open index file at `{index_path:?}`"))?;
            let mut reader = std::io::BufReader::new(file);
    
            Index::parse(&mut reader)
        }
        else if branch::get_current(&self.workdir)?
            .tip(&self.workdir)?
            .is_some()
        {
            // Index file was deleted or something
            // This shouldn't happen during normal usage
            Err(RepoError::IndexMissing.into())
        }
        else {
            // Repo was just created and there are no commits yet
            // Create an empty index
            Ok(Index::new(None))
        }
    }

    pub fn get_config(&self, section: &str, key: &str) -> Option<&str> {
        // TODO support global config
        self.config.get_from(Some(section), key)
    }

    pub fn set_config(&mut self, section: &str, key: &str, value: String) {
        self.config.set_to(Some(section), key.to_owned(), value)
    }

    pub fn workdir(&self) -> &WorkDir {
        &self.workdir
    }

}

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("Could not initialize repo at `{0:?}` because a file or nonempty directory exists there")]
    InitPathExists(PathBuf),
    #[error("No git repo contains `{0:?}`")]
    UninitializedDirectory(PathBuf),
    #[error("No repo format version was specified")]
    FmtVersionMissing,
    #[error("Repo format version `{0}` is not supported")]
    FmtVersionUnsupported(String),
    #[error("The index file is missing")]
    IndexMissing,
}
