use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    error::Error,
    repo::{GitRepository, repo_path},
    object::ObjectHash,
};

pub fn ref_resolve<P>(repo: &GitRepository, ref_path: P) -> Result<ObjectHash, Error>
where
    P: AsRef<Path>
{
    let ref_path = repo_path(&repo, ref_path);
    let ref_contents = fs::read_to_string(ref_path)?;
    let ref_contents = ref_contents.trim();

    if ref_contents.starts_with("ref: ") {
        let indirect_ref_path = &ref_contents["ref: ".len()..];
        ref_resolve(&repo, indirect_ref_path)
    }
    else {
        ObjectHash::try_from(ref_contents)
    }
}

pub fn ref_list(repo: &GitRepository) -> Result<Vec<(PathBuf, ObjectHash)>, Error> {
    let prev_working_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo_path(&repo, "."))?;

    let mut refs = Vec::new();
    ref_list_recursive(&repo, "refs", &mut refs)?;

    std::env::set_current_dir(prev_working_dir)?;

    Ok(refs)
}

fn ref_list_recursive<P>(repo: &GitRepository, rel_path: P, refs: &mut Vec<(PathBuf, ObjectHash)>) -> Result<(), Error>
where
    P: AsRef<Path>
{
    for entry in fs::read_dir(&rel_path)? {
        let path = entry?.path();

        if path.is_dir() {
            ref_list_recursive(&repo, path, refs)?;
        }
        else {
            let ref_hash = ref_resolve(&repo, &path)?;
            refs.push((
                path,
                ref_hash,
            ));
        }
    }

    Ok(())
}
