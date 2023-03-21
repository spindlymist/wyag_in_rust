use std::{fs, path::{Path, PathBuf}};

use anyhow::Context;
use thiserror::Error;

use crate::{
    Result,
    workdir::WorkDir,
    object::ObjectHash,
};

/// Creates a new ref at refs/prefix/name that points to `hash`.
pub fn create(wd: &WorkDir, prefix: &str, name: &str, hash: &ObjectHash) -> Result<()>
{
    let rel_path: PathBuf = ["refs", prefix, name].iter().collect();
    let abs_path = wd.git_path(rel_path);
    fs::write(abs_path, format!("{hash}\n"))?;

    Ok(())
}

/// Determines the hash pointed to by the ref located at refs/prefix/name.
pub fn resolve(wd: &WorkDir, prefix: &str, name: &str) -> Result<ObjectHash>
{
    let rel_path: PathBuf = ["refs", prefix, name].iter().collect();
    resolve_path(wd, rel_path)
}

/// Determines the hash pointed to by the ref located at `rel_path`.
pub fn resolve_path<P>(wd: &WorkDir, rel_path: P) -> Result<ObjectHash>
where
    P: AsRef<Path>
{
    let rel_path = rel_path.as_ref();
    let abs_path = wd.git_path(rel_path);

    if !abs_path.is_file() {
        return Err(RefError::Nonexistent(rel_path.to_owned()).into());
    }

    let ref_contents = fs::read_to_string(&abs_path)
        .with_context(|| format!("Failed to read ref at `{abs_path:?}`"))?;
    let ref_contents = ref_contents.trim();

    // A valid ref is either a hash or the name of another ref
    if let Some(indirect_path) = ref_contents.strip_prefix("ref: ") {
        if indirect_path.is_empty() {
            return Err(RefError::Corrupt {
                ref_path: rel_path.to_owned(),
                ref_contents: ref_contents.to_owned(),
            }.into());
        }

        resolve_path(wd, indirect_path)
            .map_err(|err| match err.downcast::<RefError>() {
                Ok(next_err) => RefError::BadChain {
                    ref_path: rel_path.to_owned(),
                    next: Box::new(next_err),
                }.into(),
                Err(err) => err,
            })
    }
    else if let Ok(hash) = ObjectHash::try_from(ref_contents) {
        Ok(hash)
    }
    else {
        Err(RefError::Corrupt {
            ref_path: rel_path.to_owned(),
            ref_contents: ref_contents.to_owned(),
        }.into())
    }
}

/// Enumerates all of the refs defined in the repo.
pub fn list(wd: &WorkDir) -> Result<Vec<(String, ObjectHash)>> {
    let prev_working_dir = std::env::current_dir()?;
    std::env::set_current_dir(wd.git_path("."))?;

    let mut refs = Vec::new();
    list_recursive(wd, "refs", &mut refs)?;

    std::env::set_current_dir(prev_working_dir)?;

    Ok(refs)
}

/// Enumerates all of the refs defined in the directory at `rel_path`.
fn list_recursive<P>(wd: &WorkDir, rel_path: P, refs: &mut Vec<(String, ObjectHash)>) -> Result<()>
where
    P: AsRef<Path>
{
    for entry in fs::read_dir(&rel_path)? {
        let path = entry?.path();

        if path.is_dir() {
            list_recursive(wd, path, refs)?;
        }
        else {
            let hash = resolve_path(wd, &path)?;
            refs.push((
                path.to_string_lossy().replace('\\', "/"),
                hash,
            ));
        }
    }

    Ok(())
}

pub fn delete(wd: &WorkDir, prefix: &str, name: &str) -> Result<()> {
    let rel_path: PathBuf = ["refs", prefix, name].iter().collect();
    let abs_path = wd.git_path(rel_path);

    if abs_path.is_file() {
        std::fs::remove_file(abs_path)?;
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum RefError {
    #[error("No ref found at `{0:?}`")]
    Nonexistent(PathBuf),
    #[error("The ref `{ref_path:?}` is corrupt (contents: `{ref_contents}`)")]
    Corrupt {
        ref_path: PathBuf,
        ref_contents: String,
    },
    #[error("The ref `{ref_path:?}` points to a bad ref (possibly indirectly)")]
    BadChain {
        ref_path: PathBuf,
        next: Box<RefError>,
    }
}
