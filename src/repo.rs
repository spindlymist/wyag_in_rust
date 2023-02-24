use std::{
    path::Path,
    fs::{self, OpenOptions},
    io::Write,
};
use ini::Ini;
use path_absolutize::Absolutize;

use crate::{
    Error,
    Result,
    workdir::WorkDir,
};

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
            let workdir = WorkDir::new(dir)?;

            // Validate working directory
            if workdir.as_path().is_dir()
                && workdir.as_path().read_dir()?.next().is_some()
            {
                return Err(Error::InitDirectoryNotEmpty);
            }
            
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
        repo.workdir.make_git_dir("branches")?;
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
        let workdir = WorkDir::new(dir)?;
        if !workdir.git_path(".").is_dir() {
            return Err(Error::DirectoryNotInitialized);
        }

        let config_file = workdir.git_path("config");
        let config = Ini::load_from_file(config_file)?;

        match config.get_from(Some("core"), "repositoryformatversion") {
            Some("0") => (),
            Some(version) => return Err(Error::UnsupportedRepoFmtVersion(version.to_owned())),
            None => return Err(Error::RepoFmtVersionMissing),
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
            Err(Error::DirectoryNotInitialized)
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
