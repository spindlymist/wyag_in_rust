use std::{
    path::{Path, PathBuf},
    fs::{self, File, OpenOptions},
    io::{Write},
};
use ini::Ini;
use path_absolutize::Absolutize;

use crate::{
    error::Error,
};

pub struct GitRepository {
    working_dir: PathBuf,
    git_dir: PathBuf,
    config: Ini,
}

impl GitRepository {
    pub fn init<P>(dir: P) -> Result<GitRepository, Error>
    where
        P: AsRef<Path>
    {
        let working_dir = dir.as_ref().absolutize()?.to_path_buf();

        if working_dir.is_file() {
            return Err(Error::InitPathIsFile);
        }
        else if working_dir.is_dir() {
            match working_dir.read_dir() {
                Ok(mut files) => {
                    if files.next().is_some() {
                        return Err(Error::InitDirectoryNotEmpty);
                    }
                },
                Err(err) => return Err(err.into()),
            };
        }
        
        let git_dir = working_dir.join(".git");
        fs::create_dir_all(&git_dir)?;

        let mut config = Ini::new();
        config.with_section(Some("core"))
            .set("repositoryformatversion", "0")
            .set("filemode", "false")
            .set("bare", "false");

        let repo = GitRepository {
            working_dir,
            git_dir,
            config,
        };
        
        repo_make_dir(&repo, "branches")?;
        repo_make_dir(&repo, "objects")?;
        repo_make_dir(&repo, "refs/tags")?;
        repo_make_dir(&repo, "refs/heads")?;

        let mut options = OpenOptions::new();
        options
            .create(true)
            .append(true);

        let mut description_file = repo_open_file(&repo, "description", Some(&options))?;
        description_file.write_all(b"Unnamed repository; edit this file 'description' to name the repository.\n")?;

        let mut head_file = repo_open_file(&repo, "HEAD", Some(&options))?;
        head_file.write_all(b"ref: refs/heads/master\n")?;

        let mut config_file = repo_open_file(&repo, "config", Some(&options))?;
        repo.config.write_to(&mut config_file)?;

        Ok(repo)
    }

    pub fn from_dir<P>(dir: P) -> Result<GitRepository, Error>
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
            Some(version) => return Err(Error::UnsupportedRepoFmtVersion(String::from(version))),
            None => return Err(Error::RepoFmtVersionMissing),
        };

        Ok(GitRepository {
            working_dir,
            git_dir,
            config,
        })
    }
}

pub fn repo_canonicalize<P>(repo: &GitRepository, path: P) -> Result<String, Error>
where
    P: AsRef<Path>
{
    let abs_path = path.as_ref().absolutize()?;

    let mut name = match abs_path.strip_prefix(&repo.working_dir) {
        Ok(path) => path.to_string_lossy().replace("\\", "/"),
        Err(_) => return Err(Error::InvalidPath),
    };

    if name.ends_with("/") {
        name.truncate(name.len() - 1);
    }

    Ok(name)
}

pub fn repo_path<P>(repo: &GitRepository, rel_path: P) -> PathBuf
where
    P: AsRef<Path>
{
    repo.git_dir.join(rel_path)
}

pub fn repo_working_path<P>(repo: &GitRepository, rel_path: P) -> PathBuf
where
    P: AsRef<Path>
{
    repo.working_dir.join(rel_path)
}

pub fn repo_open_file<P>(repo: &GitRepository, rel_path: P, options: Option<&OpenOptions>) -> Result<File, Error>
where
    P: AsRef<Path>
{    
    if let Some(parent_path) = rel_path.as_ref().parent() {
        repo_make_dir(repo, parent_path)?;
    }
    
    let abs_path = repo_path(&repo, rel_path);

    if let Some(options) = options {
        Ok(options.open(abs_path)?)
    }
    else {
        Ok(File::open(abs_path)?)
    }
}

pub fn repo_make_dir<P>(repo: &GitRepository, rel_path: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path>
{
    let abs_path = repo_path(&repo, rel_path);

    if !abs_path.is_dir() {
        fs::create_dir_all(&abs_path)?;
    }
    
    Ok(abs_path)
}

/// Finds the git repository that contains `path` (if it exists).
pub fn repo_find<P>(path: P) -> Result<GitRepository, Error>
where
    P: AsRef<Path>
{
    let abs_path = path.as_ref().absolutize()?;

    // The existence of a .git directory is considered sufficient
    // evidence of a repository
    if abs_path.join(".git").is_dir() {
        return GitRepository::from_dir(&abs_path);
    }

    // Recurse up the directory tree
    if let Some(parent_path) = abs_path.parent() {
        repo_find(parent_path)
    }
    else {
        // Reached root without finding a .git directory
        Err(Error::DirectoryNotInitialized)
    }
}
