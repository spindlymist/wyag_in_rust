use std::{
    collections::{HashSet, BTreeMap}
};

use anyhow::{Context, bail};

use crate::{Result, workdir::{WorkDir, WorkPathBuf, WorkPath}, index::Index};
use super::{ObjectError, ObjectHash, ObjectFormat, GitObject};

pub struct Tree {
    pub entries: BTreeMap<WorkPathBuf, TreeEntry>,
}

#[derive(Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub hash: ObjectHash,
}

impl Tree {
    /// Updates the working directory at path `target` to match this tree.
    /// The existing file or directory at `target` (if any) will be deleted.
    pub fn restore(&self, wd: &WorkDir, target: &WorkPath) -> Result<()> {
        let abs_path = wd.as_path().join(target);

        // Remove existing file(s)
        if abs_path.is_file() {
            std::fs::remove_file(&abs_path)?;
        }
        else if abs_path.is_dir() {
            // Delete everything except the .git directory (if present)
            // Note that any .git directories in subdirectories will be deleted
            for entry in abs_path.read_dir()? {
                let entry = entry?;
                let entry_path = entry.path();
                
                if entry_path.is_file() {
                    std::fs::remove_file(&entry_path)?;
                }
                else if entry_path.is_dir() && entry.file_name() != ".git" {
                    std::fs::remove_dir_all(&entry_path)?;
                }
            }
        }
        else {
            std::fs::create_dir(&abs_path)?;
        }

        // Copy files from the repo to the working directory
        for (name, entry) in &self.entries {
            let object_path = target.to_owned().join(name);
        
            match GitObject::read(wd, &entry.hash)? {
                GitObject::Blob(blob) => {
                    let object_abs_path = wd.as_path().join(object_path);
                    std::fs::write(object_abs_path, blob.serialize_into())?;
                },
                GitObject::Tree(tree) => {
                    tree.restore(wd, &object_path)?;
                },
                object => bail!("Failed to parse tree (expected tree or blob, got {})", object.get_format()),
            };
        }

        Ok(())
    }

    pub fn create_from_index(index: &Index, wd: &WorkDir) -> Result<(ObjectHash, GitObject)> {
        let prefix = WorkPathBuf::try_from("")?;
        Self::make_subtree(index, wd, &prefix)
    }

    fn make_subtree(index: &Index, wd: &WorkDir, prefix: &WorkPath) -> Result<(ObjectHash, GitObject)> {
        let mut entries = BTreeMap::new();
        let mut subtrees_handled: HashSet<&WorkPath> = HashSet::new();
        
        let index_entries = index.entries_in_dir(prefix);
        for (path, index_entry) in index_entries {
            let (name, subpath) =
                path.strip_prefix(prefix)
                .expect("Prefix should be present because it's used to construct range")
                .partition();

            if let Some(subpath) = subpath {
                let subtree_prefix = path.strip_suffix(subpath).expect("rest should be a suffix of path");
                if !subtrees_handled.insert(subtree_prefix) {
                    continue;
                }

                let (subtree_hash, _) = Self::make_subtree(index, wd, subtree_prefix)?;
                let tree_entry = TreeEntry {
                    mode: "40000".to_owned(), // git drops the leading 0 when storing a tree
                    hash: subtree_hash,
                };
                entries.insert(name.to_owned(), tree_entry);
            }
            else {
                let tree_entry = TreeEntry {
                    mode: index_entry.stats.get_mode_string(),
                    hash: index_entry.hash,
                };
                entries.insert(name.to_owned(), tree_entry);
            }
        }

        let tree = GitObject::Tree(Tree { entries });
        let hash = tree.write(wd)?;

        Ok((hash, tree))
    }

    pub fn find_entry(&self, wd: &WorkDir, path: &WorkPath) -> Result<Option<TreeEntry>> {
        if let Some(entry) = self.entries.get(path) {
            Ok(Some(entry.clone()))
        }
        else {
            let (first, rest) = path.partition();

            if let Some(rest) = rest {
                if let Some(entry) = self.entries.get(first) {
                    let subtree = Tree::read(wd, &entry.hash)?;
                    return subtree.find_entry(wd, rest);
                }
            }

            Ok(None)
        }
    }

    pub fn read(wd: &WorkDir, hash: &ObjectHash) -> Result<Tree> {
        match GitObject::read(wd, hash)? {
            GitObject::Tree(tree) => Ok(tree),
            object => Err(ObjectError::UnexpectedFormat {
                format: object.get_format(),
                expected: ObjectFormat::Tree,
            }.into()),
        }
    }

    pub fn read_from_commit(wd: &WorkDir, commit_hash: &ObjectHash) -> Result<Tree> {
        let commit = super::Commit::read(wd, commit_hash)?;

        Self::read(wd, commit.tree())
    }

    pub fn deserialize(data: Vec<u8>) -> Result<Tree> {
        let mut entries = BTreeMap::new();
        let mut iter = data.into_iter();

        loop {
            let mode = {
                let mode_bytes: Vec<u8> = iter.by_ref()
                    .take_while(|ch| *ch != b' ')
                    .collect();
                String::from_utf8(mode_bytes)
                    .context("Failed to parse tree (invalid Utf-8)")?
            };

            if mode.is_empty() {
                break;
            }

            let path = {
                let path: Vec<u8> = iter.by_ref()
                    .take_while(|ch| *ch != 0)
                    .collect();

                WorkPathBuf::try_from(&path[..])
                    .context("Failed to parse tree (invalid path)")?
            };

            let hash = {
                let hash_bytes: Vec<u8> = iter.by_ref()
                    .take(20)
                    .collect();

                ObjectHash::try_from(&hash_bytes[..])
                    .context("Failed to parse tree (invalid hash)")?
            };

            entries.insert(path, TreeEntry {
                mode,
                hash
            });
        }

        Ok(Tree { entries })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = vec![];

        for (path, entry) in &self.entries {
            data.extend(format!("{} {}\0", entry.mode, path).into_bytes());
            data.extend(entry.hash.raw);
        }

        data
    }

    pub fn serialize_into(self) -> Vec<u8> {
        self.serialize()
    }
}

impl TreeEntry {
    pub fn is_dir(&self) -> bool {
        self.mode == "40000"
    }
}
