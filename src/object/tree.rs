use std::{
    collections::{HashSet, BTreeMap}
};

use anyhow::{Context, bail};

use crate::{Result, workdir::{WorkDir, WorkPathBuf, WorkPath}, index::Index};
use super::{ObjectError, ObjectHash, ObjectFormat, GitObject, Blob};

/// A tree represents one level (directory) in a file hierarchy. Files and subdirectories are recorded
/// as hashes which map to blobs and trees, respectively.
pub struct Tree {
    pub entries: BTreeMap<WorkPathBuf, TreeEntry>,
}

/// A single entry in a [`Tree`], which may represent a file (blob) or subdirectory (tree).
#[derive(Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub hash: ObjectHash,
}

impl Tree {
    /// Copies files from the repository to the working directory at `target`.
    fn restore_at_path(&self, wd: &WorkDir, target: &WorkPath) -> Result<()> {
        let abs_path = wd.as_path().join(target);
        wd.remove_path(target)?;
        std::fs::create_dir_all(&abs_path)?;

        for (name, entry) in &self.entries {
            let object_path = target.to_owned().join(name);
        
            match GitObject::read(wd, &entry.hash)? {
                GitObject::Blob(blob) => {
                    let object_abs_path = wd.as_path().join(object_path);
                    std::fs::write(object_abs_path, blob.serialize_into())?;
                },
                GitObject::Tree(tree) => {
                    tree.restore_at_path(wd, &object_path)?;
                },
                object => bail!("Failed to parse tree (expected tree or blob, got {})", object.get_format()),
            };
        }

        Ok(())
    }

    /// Updates the working directory at path `target` to match the tree associated with the specified commit.
    /// The existing file or directory at `target` (if any) will be deleted.
    pub fn restore_from_commit(wd: &WorkDir, commit_hash: &ObjectHash, target: &WorkPath) -> Result<()> {
        let root_tree = Tree::read_from_commit(wd, commit_hash)?;
        
        if target.is_empty() {
            // Case 1: restore root tree
            root_tree.restore_at_path(wd, target)?;
        }
        else if let Some(entry) = root_tree.find_entry(wd, target)? {
            if entry.is_dir() {
                // Case 2: restore subtree
                let tree = Tree::read(wd, &entry.hash)?;
                tree.restore_at_path(wd, target)?;
            }
            else {
                // Case 3: restore file
                wd.remove_path(target)?;

                let abs_path = wd.as_path().join(target);
                if let Some(dir_path) = abs_path.parent() {
                    std::fs::create_dir_all(dir_path)?;
                }

                let blob = Blob::read(wd, &entry.hash)?;
                std::fs::write(abs_path, blob.serialize_into())?;
            }
        }

        Ok(())
    }

    /// Constructs an [`Index`] from this tree.
    pub fn to_index(&self, wd: &WorkDir, version: Option<u32>) -> Result<Index> {
        let mut index = Index::new(version);
        self.add_to_index_recursive(wd, &mut index, &WorkPathBuf::root())?;

        Ok(index)
    }

    /// Adds the entries in this tree to `index` under the path `prefix`.
    fn add_to_index_recursive(&self, wd: &WorkDir, index: &mut Index, prefix: &WorkPath) -> Result<()> {
        for (name, entry) in &self.entries {
            let mut full_path = prefix.to_owned();
            full_path.push(name);

            if entry.is_dir() {
                let tree = Tree::read(wd, &entry.hash)?;
                tree.add_to_index_recursive(wd, index, &full_path)?;
            }
            else {
                let blob = Blob::read(wd, &entry.hash)?;
                let size = blob.size().try_into().unwrap_or(u32::MAX);
                index.entries.insert(full_path, crate::index::IndexEntry {
                    stats: crate::index::stats::FileStats::from_size(size),
                    hash: entry.hash,
                    flags: crate::index::flags::EntryFlags::new(name.as_str()),
                });
            }
        }

        Ok(())
    }

    /// Generates a tree from `index` and stores it in the repository.
    pub fn create_from_index(index: &Index, wd: &WorkDir) -> Result<(ObjectHash, GitObject)> {
        let prefix = WorkPathBuf::try_from("")?;
        Self::make_subtree(index, wd, &prefix)
    }

    /// Generates a tree from the entries in `index` under the path `prefix` and stores it in the repository.
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

    /// Finds the entry associated with `path` relative to this tree. Returns `None`
    /// if no entry is found.
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

    /// Reads and parses the tree with the given hash from the repo.
    pub fn read(wd: &WorkDir, hash: &ObjectHash) -> Result<Tree> {
        match GitObject::read(wd, hash)? {
            GitObject::Tree(tree) => Ok(tree),
            object => Err(ObjectError::UnexpectedFormat {
                format: object.get_format(),
                expected: ObjectFormat::Tree,
            }.into()),
        }
    }

    /// Reads and parses the tree associated with the commit with the given hash from the repo.
    pub fn read_from_commit(wd: &WorkDir, commit_hash: &ObjectHash) -> Result<Tree> {
        let commit = super::Commit::read(wd, commit_hash)?;

        Self::read(wd, commit.tree())
    }

    /// Parses a `Tree` from a sequence of bytes.
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

    /// Converts the tree into a sequence of bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = vec![];

        for (path, entry) in &self.entries {
            data.extend(format!("{} {}\0", entry.mode, path).into_bytes());
            data.extend(entry.hash.raw);
        }

        data
    }

    /// Consumes the tree and converts it into a sequence of bytes.
    pub fn serialize_into(self) -> Vec<u8> {
        self.serialize()
    }
}

impl TreeEntry {
    pub fn is_dir(&self) -> bool {
        self.mode == "40000"
    }
}
