use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use crate::{
    Error,
    Result,
    repo::{GitRepository, repo_path},
    object::ObjectHash,
};

pub fn ref_create<P>(repo: &GitRepository, ref_name: P, ref_hash: &ObjectHash) -> Result<()>
where
    P: AsRef<Path>
{
    let ref_path = repo_path(repo, PathBuf::from("refs").join(ref_name));
    fs::write(ref_path, format!("{ref_hash}\n"))?;

    Ok(())
}

pub fn ref_resolve<P>(repo: &GitRepository, ref_path: P) -> Result<ObjectHash>
where
    P: AsRef<Path>
{
    let ref_path = repo_path(repo, ref_path);
    let ref_contents = match fs::read_to_string(ref_path) {
        Ok(val) => val,
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => return Err(Error::InvalidRef),
            _ => return Err(err.into()),
        },
    };
    let ref_contents = ref_contents.trim();

    if let Some(indirect_ref_path) = ref_contents.strip_prefix("ref: ") {
        ref_resolve(repo, indirect_ref_path)
    }
    else {
        ObjectHash::try_from(ref_contents)
    }
}

pub fn ref_list(repo: &GitRepository) -> Result<Vec<(String, ObjectHash)>> {
    let prev_working_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo_path(repo, "."))?;

    let mut refs = Vec::new();
    ref_list_recursive(repo, "refs", &mut refs)?;

    std::env::set_current_dir(prev_working_dir)?;

    Ok(refs)
}

fn ref_list_recursive<P>(repo: &GitRepository, rel_path: P, refs: &mut Vec<(String, ObjectHash)>) -> Result<()>
where
    P: AsRef<Path>
{
    for entry in fs::read_dir(&rel_path)? {
        let path = entry?.path();

        if path.is_dir() {
            ref_list_recursive(repo, path, refs)?;
        }
        else {
            let ref_hash = ref_resolve(repo, &path)?;
            refs.push((
                path.to_string_lossy().replace('\\', "/"),
                ref_hash,
            ));
        }
    }

    Ok(())
}
