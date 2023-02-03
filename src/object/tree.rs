use std::{path::Path, collections::HashSet};

use crate::{Error, Result, repo::GitRepository, index::Index};
use super::{ObjectHash, object_read, GitObject, object_write};

pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: ObjectHash,
}

impl Tree {
    pub fn deserialize(data: Vec<u8>) -> Result<Tree> {
        let mut entries = vec![];
        let mut iter = data.into_iter();

        loop {
            let mode: Vec<u8> = iter.by_ref()
                .take_while(|ch| *ch != b' ')
                .collect();
            if mode.is_empty() { break; }
            let mode = match String::from_utf8(mode) {
                Ok(val) => val,
                Err(_) => return Err(Error::BadTreeFormat),
            };

            let name: Vec<u8> = iter.by_ref()
                .take_while(|ch| *ch != 0)
                .collect();
            let name = match String::from_utf8(name) {
                Ok(val) => val,
                Err(_) => return Err(Error::BadTreeFormat),
            };

            let hash: Vec<u8> = iter.by_ref().take(20).collect();
            let hash: [u8; 20] = match hash.try_into() {
                Ok(val) => val,
                Err(_) => return Err(Error::BadTreeFormat),
            };
            let hash = ObjectHash { raw: hash };

            entries.push(TreeEntry {
                mode,
                name,
                hash
            });
        }

        Ok(Tree { entries })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = vec![];

        for entry in &self.entries {
            data.extend(format!("{} {}\0", entry.mode, entry.name).into_bytes());
            data.extend(entry.hash.raw);
        }

        data
    }

    pub fn serialize_into(self) -> Vec<u8> {
        self.serialize()
    }
}

pub fn tree_checkout<P>(repo: &GitRepository, tree: &Tree, path: P) -> Result<()>
where
    P: AsRef<Path>
{
    for entry in &tree.entries {
        let object_path = path.as_ref().join(&entry.name);

        match object_read(repo, &entry.hash)? {
            GitObject::Blob(blob) => {
                std::fs::write(object_path, blob.serialize_into())?;
            },
            GitObject::Tree(tree) => {
                std::fs::create_dir(&object_path)?;
                tree_checkout(repo, &tree, object_path)?;
            },
            _ => return Err(Error::BadTreeFormat),
        };
    }

    Ok(())
}

pub fn tree_create_from_index(index: &Index, repo: &GitRepository) -> Result<ObjectHash> {
    make_subtree(index, repo, "")
}

fn make_subtree(index: &Index, repo: &GitRepository, prefix: &str) -> Result<ObjectHash> {
    let mut entries = vec![];
    let mut prefixes_handled: HashSet<&str> = HashSet::new();

    for (name, index_entry) in &index.entries {
        if let Some(suffix) = name.strip_prefix(prefix) {
            if let Some(slash_idx) = suffix.find('/') {
                let new_prefix = &name[..=prefix.len() + slash_idx];

                if prefixes_handled.contains(new_prefix) {
                    continue;
                }
                else {
                    prefixes_handled.insert(new_prefix);
                }

                let subtree_hash = make_subtree(index, repo, new_prefix)?;
                let tree_entry = TreeEntry {
                    mode: "040000".to_owned(),
                    name: String::from(&suffix[..slash_idx]),
                    hash: subtree_hash,
                };
                entries.push(tree_entry);
            }
            else {
                let tree_entry = TreeEntry {
                    mode: index_entry.stats.get_mode_string(),
                    name: String::from(suffix),
                    hash: index_entry.hash,
                };
                entries.push(tree_entry);
            }
        }
    }

    let tree = GitObject::Tree(Tree { entries });
    let hash = object_write(repo, &tree)?;

    Ok(hash)
}
