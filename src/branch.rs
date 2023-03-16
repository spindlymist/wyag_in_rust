use std::{fs, collections::VecDeque};

use thiserror::Error;

use crate::{
    Result,
    refs::{self, RefError},
    workdir::WorkDir,
    object::{ObjectHash, GitObject, ObjectFormat}
};

pub enum Branch {
    Named(String),
    Headless(ObjectHash),
}

impl Branch {
    pub fn tip(&self, wd: &WorkDir) -> Result<Option<ObjectHash>> {
        match self {
            Branch::Named(name) => match refs::resolve(wd, "heads", name) {
                Ok(hash) => Ok(Some(hash)),
                Err(err) => match err.downcast_ref::<RefError>() {
                    Some(RefError::Nonexistent(_)) => Ok(None),
                    Some(_) | None => Err(err),
                }
            },
            Branch::Headless(hash) => Ok(Some(*hash))
        }
    }
}

/// Determines the branch pointed to by the HEAD of `repo`.
pub fn get_current(wd: &WorkDir) -> Result<Branch> {
    let head_path = wd.git_path("HEAD");
    let head_contents = fs::read_to_string(head_path)?;
    let head_contents = head_contents.trim();

    // HEAD should either be a ref or a commit hash
    if !head_contents.starts_with("ref: ") {
        let commit_hash = ObjectHash::try_from(head_contents)?;
        Ok(Branch::Headless(commit_hash))
    }
    else if let Some(branch_name) = head_contents.strip_prefix("ref: refs/heads/") {
        if branch_name.is_empty() {
            return Err(BranchError::UnrecognizedHeadRef(head_contents.to_owned()).into());
        }

        Ok(Branch::Named(String::from(branch_name)))
    }
    else {
        // Could be a remote ref which is currently unsupported
        Err(BranchError::UnrecognizedHeadRef(head_contents.to_owned()).into())
    }
}

/// Creates a new branch called `name`. The tip of the branch will be the commit
/// identified by `commit_hash`.
pub fn create(name: &str, wd: &WorkDir, commit_hash: &ObjectHash) -> Result<()> {
    if exists(name, wd)? {
        return Err(BranchError::AlreadyExists(name.to_owned()).into());
    }

    refs::create(wd, "heads", name, commit_hash)?;

    Ok(())
}

/// Deletes the branch called `name`.
pub fn delete(name: &str, wd: &WorkDir) -> Result<()> {
    let current_branch = get_current(wd)?;

    if let Branch::Named(current_name) = current_branch {
        if name == current_name {
            return Err(BranchError::CheckedOut(name.to_owned()).into());
        }

        if !is_merged(name, &current_name, wd)? {
            return Err(BranchError::PossiblyUnmerged(name.to_owned()).into());
        }

        refs::delete(wd, "heads", name)
    }
    else {
        Err(BranchError::PossiblyUnmerged(name.to_owned()).into())
    }
}

/// Moves the tip of the branch called `name` to the commit identified by `commit_hash`.
pub fn update(name: &str, wd: &WorkDir, commit_hash: &ObjectHash) -> Result<()> {
    refs::create(wd, "heads", name, commit_hash)?;

    Ok(())
}

/// Moves the tip of the current branch to the commit identified by `commit_hash`.
pub fn update_current(wd: &WorkDir, commit_hash: &ObjectHash) -> Result<()> {
    match get_current(wd)? {
        Branch::Named(branch_name) => {
            update(&branch_name, wd, commit_hash)?;
        },
        Branch::Headless(_) => {
            let head_path = wd.git_path("HEAD");
            std::fs::write(head_path, format!("{commit_hash}\n"))?;
        },
    };

    Ok(())
}

/// Returns true if the branch called `name` exists.
pub fn exists(name: &str, wd: &WorkDir) -> Result<bool> {
    match refs::resolve(wd, "heads", name) {
        Ok(_) => Ok(true),
        Err(err) => match err.downcast_ref::<RefError>() {
            Some(RefError::Nonexistent(_)) => Ok(false),
            Some(_) | None => Err(err),
        },
    }
}

/// Determines if the branch `name` has been merged into `into_branch`.
pub fn is_merged(name: &str, into_branch: &str, wd: &WorkDir) -> Result<bool> {
    let our_tip = refs::resolve(wd, "heads", name)?;
    let their_tip = refs::resolve(wd, "heads", into_branch)?;
    
    // Conduct a breadth-first search for our_tip in the commit graph of into_branch
    let mut open_hashes = VecDeque::new();
    open_hashes.push_back(their_tip);

    while let Some(hash) = open_hashes.pop_front() {
        if hash == our_tip {
            return Ok(true);
        }

        let commit = match GitObject::read(wd, &hash)? {
            GitObject::Commit(commit) => commit,
            object => return Err(BranchError::BrokenCommitGraph(object.get_format()).into()),
        };

        open_hashes.extend(commit.parents());
    }

    Ok(false)
}

#[derive(Error, Debug)]
pub enum BranchError {
    #[error("There is no branch called `{0}`")]
    Nonexistent(String),
    #[error("Cannot create a branch `{0}` because one already exists")]
    AlreadyExists(String),
    #[error("Cannot delete the branch `{0}` because it is currently checked out")]
    CheckedOut(String),
    #[error("Cannot delete the branch `{0}` because it may contain unmerged changes")]
    PossiblyUnmerged(String),
    #[error("The HEAD ref `{0}` was not recognized (remotes are unsupported)")]
    UnrecognizedHeadRef(String),
    #[error("The commit graph contains a {0}")]
    BrokenCommitGraph(ObjectFormat),
}
