use std::fs;

use crate::{
    error::Error,
    refs::{ref_resolve, ref_create},
    repo::{GitRepository, repo_path},
    object::{ObjectHash}
};

pub enum Branch {
    Named(String),
    Headless(ObjectHash),
}

pub fn branch_get_current(repo: &GitRepository) -> Result<Branch, Error> {
    let head_path = repo_path(repo, "HEAD");
    let head_contents = fs::read_to_string(head_path)?;
    let head_contents = head_contents.trim();

    if !head_contents.starts_with("ref: ") {
        let commit_hash = ObjectHash::try_from(head_contents)?;
        Ok(Branch::Headless(commit_hash))
    }
    else if let Some(branch_name) = head_contents.strip_prefix("ref: refs/heads/") {
        if branch_name.is_empty() {
            return Err(Error::InvalidRef);
        }
        Ok(Branch::Named(String::from(branch_name)))
    }
    else {
        Err(Error::UnrecognizedHeadRef)
    }
}

pub fn branch_create(name: &str, repo: &GitRepository, commit_hash: &ObjectHash) -> Result<(), Error> {
    if branch_exists(name, repo)? {
        return Err(Error::BranchAlreadyExists);
    }

    ref_create(repo, format!("heads/{name}"), commit_hash)?;

    Ok(())
}

pub fn branch_delete(_name: &str, _repo: &GitRepository) -> Result<ObjectHash, Error> {
    todo!()
}

pub fn branch_update(name: &str, repo: &GitRepository, commit_hash: &ObjectHash) -> Result<(), Error> {
    if !branch_exists(name, repo)? {
        return Err(Error::InvalidRef);
    }

    ref_create(repo, format!("heads/{name}"), commit_hash)?;

    Ok(())
}

pub fn branch_update_current(repo: &GitRepository, commit_hash: &ObjectHash) -> Result<(), Error> {
    match branch_get_current(repo)? {
        Branch::Named(branch_name) => {
            branch_update(&branch_name, repo, commit_hash)?;
        },
        Branch::Headless(_) => {
            let head_path = repo_path(repo, "HEAD");
            std::fs::write(head_path, format!("{commit_hash}\n"))?;
        },
    };

    Ok(())
}

pub fn branch_exists(name: &str, repo: &GitRepository) -> Result<bool, Error> {
    match ref_resolve(repo, format!("refs/heads/{name}")) {
        Ok(_) => Ok(true),
        Err(err) => match err {
            Error::InvalidRef => Ok(false),
            _ => Err(err),
        },
    }
}
