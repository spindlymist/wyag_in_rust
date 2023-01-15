use std::{
    path::{Path, PathBuf},
    fs,
};
use ini::Ini;

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
        let working_dir = PathBuf::from(dir.as_ref());
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
            }
        }

        let git_dir = working_dir.join(".git");
        fs::create_dir_all(&git_dir)?;

        let mut config = Ini::new();
        config.with_section(Some("core"))
            .set("repositoryformatversion", "0")
            .set("filemode", "false")
            .set("bare", "false");
        config.write_to_file(git_dir.join("config"))?;

        let repo = GitRepository {
            working_dir,
            git_dir,
            config,
        };

        repo_dir(&repo, "branches")?;
        repo_dir(&repo, "objects")?;
        repo_dir(&repo, "refs/tags")?;
        repo_dir(&repo, "refs/heads")?;

        {
            let description_file = repo_file(&repo, "description")?;
            fs::write(description_file, "Unnamed repository; edit this file 'description' to name the repository.\n")?;
        }

        {
            let head_file = repo_file(&repo, "HEAD")?;
            fs::write(head_file, "ref: refs/heads/master\n")?;
        }

        Ok(repo)
    }

    pub fn from_dir<P>(dir: P) -> Result<GitRepository, Error>
    where
        P: AsRef<Path>
    {
        let working_dir = PathBuf::from(dir.as_ref());
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

pub fn repo_path<P>(repo: &GitRepository, path: P) -> PathBuf
where
    P: AsRef<Path>
{
    repo.git_dir.join(path)
}

pub fn repo_file<P>(repo: &GitRepository, path: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path>
{
    if let Some(parent_path) = path.as_ref().parent() {
        repo_dir(repo, parent_path)?;
    }

    Ok(repo_path(repo, path))
}

pub fn repo_dir<P>(repo: &GitRepository, path: P) -> Result<PathBuf, Error>
where
    P: AsRef<Path>
{
    let path = repo_path(&repo, path.as_ref());

    if !path.is_dir() {
        fs::create_dir_all(&path)?;
    }
    
    Ok(path)
}

/// Finds the git repository that contains `path` (if it exists).
pub fn repo_find<P>(path: P) -> Result<GitRepository, Error>
where
    P: AsRef<Path>
{
    let abs_path = fs::canonicalize(path)?;

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
