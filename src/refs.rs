use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use crate::{
    Error,
    Result,
    repo::Repository,
    object::ObjectHash,
};

/// Creates a new ref at refs/prefix/name that points to `hash`.
pub fn create(repo: &Repository, prefix: &str, name: &str, hash: &ObjectHash) -> Result<()>
{
    let rel_path: PathBuf = ["refs", prefix, name].iter().collect();
    let abs_path = repo.git_path(rel_path);
    fs::write(abs_path, format!("{hash}\n"))?;

    Ok(())
}

/// Determines the hash pointed to by the ref located at refs/prefix/name.
pub fn resolve(repo: &Repository, prefix: &str, name: &str) -> Result<ObjectHash>
{
    let rel_path: PathBuf = ["refs", prefix, name].iter().collect();
    resolve_path(repo, rel_path)
}

/// Determines the hash pointed to by the ref located at `rel_path`.
pub fn resolve_path<P>(repo: &Repository, rel_path: P) -> Result<ObjectHash>
where
    P: AsRef<Path>
{
    let abs_path = repo.git_path(rel_path);
    let ref_contents = match fs::read_to_string(abs_path) {
        Ok(val) => val,
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => return Err(Error::InvalidRef),
            _ => return Err(err.into()),
        },
    };
    let ref_contents = ref_contents.trim();

    // This ref may refer to another ref
    if let Some(indirect_path) = ref_contents.strip_prefix("ref: ") {
        resolve_path(repo, indirect_path)
    }
    else {
        ObjectHash::try_from(ref_contents)
    }
}

/// Determines the hash pointed to by the HEAD ref of `repo`.
pub fn head(repo: &Repository) -> Result<ObjectHash> {
    resolve_path(repo, "HEAD")
}

/// Enumerates all of the refs defined in `repo`.
pub fn list(repo: &Repository) -> Result<Vec<(String, ObjectHash)>> {
    let prev_working_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo.git_path("."))?;

    let mut refs = Vec::new();
    list_recursive(repo, "refs", &mut refs)?;

    std::env::set_current_dir(prev_working_dir)?;

    Ok(refs)
}

/// Enumerates all of the refs defined in the directory at `rel_path`.
fn list_recursive<P>(repo: &Repository, rel_path: P, refs: &mut Vec<(String, ObjectHash)>) -> Result<()>
where
    P: AsRef<Path>
{
    for entry in fs::read_dir(&rel_path)? {
        let path = entry?.path();

        if path.is_dir() {
            list_recursive(repo, path, refs)?;
        }
        else {
            let hash = resolve_path(repo, &path)?;
            refs.push((
                path.to_string_lossy().replace('\\', "/"),
                hash,
            ));
        }
    }

    Ok(())
}
