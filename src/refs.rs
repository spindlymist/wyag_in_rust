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

pub fn create<P>(repo: &Repository, ref_name: P, ref_hash: &ObjectHash) -> Result<()>
where
    P: AsRef<Path>
{
    let ref_path = repo.git_path(PathBuf::from("refs").join(ref_name));
    fs::write(ref_path, format!("{ref_hash}\n"))?;

    Ok(())
}

pub fn resolve<P>(repo: &Repository, ref_path: P) -> Result<ObjectHash>
where
    P: AsRef<Path>
{
    let ref_path = repo.git_path(ref_path);
    let ref_contents = match fs::read_to_string(ref_path) {
        Ok(val) => val,
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => return Err(Error::InvalidRef),
            _ => return Err(err.into()),
        },
    };
    let ref_contents = ref_contents.trim();

    if let Some(indirect_ref_path) = ref_contents.strip_prefix("ref: ") {
        resolve(repo, indirect_ref_path)
    }
    else {
        ObjectHash::try_from(ref_contents)
    }
}

pub fn list(repo: &Repository) -> Result<Vec<(String, ObjectHash)>> {
    let prev_working_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo.git_path("."))?;

    let mut refs = Vec::new();
    list_recursive(repo, "refs", &mut refs)?;

    std::env::set_current_dir(prev_working_dir)?;

    Ok(refs)
}

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
            let ref_hash = resolve(repo, &path)?;
            refs.push((
                path.to_string_lossy().replace('\\', "/"),
                ref_hash,
            ));
        }
    }

    Ok(())
}
