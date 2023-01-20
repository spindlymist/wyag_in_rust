use std::{
    fs,
    io::{Write},
    path::{Path, PathBuf},
};

use crate::{
    error::Error,
    repo::{GitRepository, repo_path, repo_open_file},
    object::ObjectHash,
};

pub fn ref_create<P>(repo: &GitRepository, ref_name: P, ref_hash: &ObjectHash) -> Result<(), Error>
where
    P: AsRef<Path>
{
    let mut options = fs::OpenOptions::new();
    options
        .create(true)
        .write(true)
        .truncate(true);
    let mut ref_file = repo_open_file(&repo, PathBuf::from("refs").join(ref_name), Some(&options))?;
    write!(ref_file, "{ref_hash}\n")?;

    Ok(())
}

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

pub fn ref_list(repo: &GitRepository) -> Result<Vec<(String, ObjectHash)>, Error> {
    let prev_working_dir = std::env::current_dir()?;
    std::env::set_current_dir(repo_path(&repo, "."))?;

    let mut refs = Vec::new();
    ref_list_recursive(&repo, "refs", &mut refs)?;

    std::env::set_current_dir(prev_working_dir)?;

    Ok(refs)
}

fn ref_list_recursive<P>(repo: &GitRepository, rel_path: P, refs: &mut Vec<(String, ObjectHash)>) -> Result<(), Error>
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
                path.to_string_lossy().replace("\\", "/"),
                ref_hash,
            ));
        }
    }

    Ok(())
}
