use std::{
    path::{Path, PathBuf},
    fs::{self, File, OpenOptions},
    io::{Write},
};
use ini::Ini;
use path_absolutize::Absolutize;

use crate::{
    Error,
    Result,
};

pub struct Repository {
    working_dir: PathBuf,
    git_dir: PathBuf,
    config: Ini,
}

impl Repository {
    /// Initializes a new git repository in an empty directory.
    pub fn init<P>(dir: P) -> Result<Repository>
    where
        P: AsRef<Path>
    {
        let repo = {
            let working_dir = dir.as_ref().absolutize()?.to_path_buf();

            // Validate working directory
            if working_dir.is_file() {
                return Err(Error::InitPathIsFile);
            }
            else if working_dir.is_dir() {
                let mut files = working_dir.read_dir()?;
                if files.next().is_some() {
                    return Err(Error::InitDirectoryNotEmpty);
                }
            }
            
            let git_dir = working_dir.join(".git");
            
            // Initialize config
            let mut config = Ini::new();
            config.with_section(Some("core"))
                .set("repositoryformatversion", "0")
                .set("filemode", "false")
                .set("bare", "false");
        
            Repository {
                working_dir,
                git_dir,
                config,
            }
        };
        
        // Create directories
        fs::create_dir_all(&repo.git_dir)?;
        repo.make_git_dir("branches")?;
        repo.make_git_dir("objects")?;
        repo.make_git_dir("refs/tags")?;
        repo.make_git_dir("refs/heads")?;

        // Create files
        {
            let mut options = OpenOptions::new();
            options
                .create(true)
                .append(true);

            repo.open_git_file("description", Some(&options))?
                .write_all(b"Unnamed repository; edit this file 'description' to name the repository.\n")?;

            repo.open_git_file("HEAD", Some(&options))?
                .write_all(b"ref: refs/heads/master\n")?;

            let mut config_file = repo.open_git_file("config", Some(&options))?;
            repo.config.write_to(&mut config_file)?;
        }

        Ok(repo)
    }

    /// Constructs a `Repository` from the repo in an existing directory.
    pub fn from_existing<P>(dir: P) -> Result<Repository>
    where
        P: AsRef<Path>
    {
        let working_dir = PathBuf::from(dir.as_ref().absolutize()?);
        if !working_dir.is_dir() {
            return Err(Error::WorkingDirectoryInvalid);
        }

        let git_dir = working_dir.join(".git");
        if !git_dir.is_dir() {
            return Err(Error::DirectoryNotInitialized);
        }

        let config_file = git_dir.join("config");
        let config = Ini::load_from_file(config_file)?;

        match config.get_from(Some("core"), "repositoryformatversion") {
            Some("0") => (),
            Some(version) => return Err(Error::UnsupportedRepoFmtVersion(version.to_owned())),
            None => return Err(Error::RepoFmtVersionMissing),
        };

        Ok(Repository {
            working_dir,
            git_dir,
            config,
        })
    }
    
    /// Translates a path within the repo to its canonical name.
    /// 
    /// The canonical name is relative to the working directory, uses `/` for the path separator,
    /// and does not begin or end with a slash.
    pub fn canonicalize_path<P>(&self, path: P) -> Result<String>
    where
        P: AsRef<Path>
    {
        let abs_path = path.as_ref().absolutize()?;

        let mut name = match abs_path.strip_prefix(&self.working_dir) {
            Ok(path) => path.to_string_lossy().replace('\\', "/"),
            Err(_) => return Err(Error::InvalidPath),
        };

        if name.ends_with('/') {
            name.truncate(name.len() - 1);
        }

        Ok(name)
    }

    /// Appends a relative path to the repo's .git directory.
    pub fn git_path<P>(&self, rel_path: P) -> PathBuf
    where
        P: AsRef<Path>
    {
        self.git_dir.join(rel_path)
    }

    /// Appends a relative path to the repo's working directory.
    pub fn working_path<P>(&self, rel_path: P) -> PathBuf
    where
        P: AsRef<Path>
    {
        self.working_dir.join(rel_path)
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

}
