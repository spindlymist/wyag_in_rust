use std::{fs, collections::VecDeque};

use crate::{
    Error,
    Result,
    refs,
    workdir::WorkDir,
    object::{ObjectHash, GitObject}
};

pub enum Branch {
    Named(String),
    Headless(ObjectHash),
}

impl Branch {
    pub fn tip(&self, wd: &WorkDir) -> Result<ObjectHash> {
        match self {
            Branch::Named(name) => refs::resolve(wd, "heads", name),
            Branch::Headless(hash) => Ok(*hash)
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
            return Err(Error::InvalidRef.into());
        }

        Ok(Branch::Named(String::from(branch_name)))
    }
    else {
        // Could be a remote ref which is currently unsupported
        Err(Error::UnrecognizedHeadRef.into())
    }
}

/// Creates a new branch called `name`. The tip of the branch will be the commit
/// identified by `commit_hash`.
pub fn create(name: &str, wd: &WorkDir, commit_hash: &ObjectHash) -> Result<()> {
    if exists(name, wd)? {
        return Err(Error::BranchAlreadyExists.into());
    }

    refs::create(wd, "heads", name, commit_hash)?;

    Ok(())
}

/// Deletes the branch called `name`.
pub fn delete(name: &str, wd: &WorkDir) -> Result<()> {
    let current_branch = get_current(wd)?;

    if let Branch::Named(current_name) = current_branch {
        if name == current_name {
            return Err(Error::BranchIsCheckedOut.into());
        }

        if !is_merged(name, &current_name, wd)? {
            return Err(Error::BranchPossiblyUnmerged.into());
        }

        refs::delete(wd, "heads", name)
    }
    else {
        Err(Error::BranchPossiblyUnmerged.into())
    }
}

/// Moves the tip of the branch called `name` to the commit identified by `commit_hash`.
pub fn update(name: &str, wd: &WorkDir, commit_hash: &ObjectHash) -> Result<()> {
    if !exists(name, wd)? {
        return Err(Error::InvalidRef.into());
    }

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
        Err(err) => match err.downcast_ref::<Error>() {
            Some(Error::InvalidRef) => Ok(false),
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
            _ => return Err(Error::NonCommitInGraph.into()),
        };

        for parent in commit.map.get_all("parent") {
            let parent_hash = ObjectHash::try_from(&parent[..])?;
            open_hashes.push_back(parent_hash);
        }
    }

    Ok(false)
}
