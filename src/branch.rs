use std::fs;

use crate::{
    Error,
    Result,
    refs,
    repo::Repository,
    object::{ObjectHash}
};

pub enum Branch {
    Named(String),
    Headless(ObjectHash),
}

pub fn get_current(repo: &Repository) -> Result<Branch> {
    let head_path = repo.git_path("HEAD");
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

pub fn create(name: &str, repo: &Repository, commit_hash: &ObjectHash) -> Result<()> {
    if exists(name, repo)? {
        return Err(Error::BranchAlreadyExists);
    }

    refs::create(repo, format!("heads/{name}"), commit_hash)?;

    Ok(())
}

pub fn delete(_name: &str, _repo: &Repository) -> Result<ObjectHash> {
    todo!()
}

pub fn update(name: &str, repo: &Repository, commit_hash: &ObjectHash) -> Result<()> {
    if !exists(name, repo)? {
        return Err(Error::InvalidRef);
    }

    refs::create(repo, format!("heads/{name}"), commit_hash)?;

    Ok(())
}

pub fn update_current(repo: &Repository, commit_hash: &ObjectHash) -> Result<()> {
    match get_current(repo)? {
        Branch::Named(branch_name) => {
            update(&branch_name, repo, commit_hash)?;
        },
        Branch::Headless(_) => {
            let head_path = repo.git_path("HEAD");
            std::fs::write(head_path, format!("{commit_hash}\n"))?;
        },
    };

    Ok(())
}

pub fn exists(name: &str, repo: &Repository) -> Result<bool> {
    match refs::resolve(repo, format!("refs/heads/{name}")) {
        Ok(_) => Ok(true),
        Err(err) => match err {
            Error::InvalidRef => Ok(false),
            _ => Err(err),
        },
    }
}
