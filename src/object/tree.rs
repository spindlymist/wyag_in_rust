use std::{
    path::Path, collections::{HashSet, BTreeMap}
};

use crate::{Error, Result, workdir::{WorkDir, WorkPathBuf, WorkPath}, index::Index};
use super::{ObjectHash, GitObject};

pub struct Tree {
    pub entries: BTreeMap<WorkPathBuf, TreeEntry>,
}

#[derive(Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub hash: ObjectHash,
}

impl Tree {
    pub fn checkout<P>(&self, wd: &WorkDir, path: P) -> Result<()>
    where
        P: AsRef<Path>
    {
        for (name, entry) in &self.entries {
            let object_path = path.as_ref().join(name);

            match GitObject::read(wd, &entry.hash)? {
                GitObject::Blob(blob) => {
                    std::fs::write(object_path, blob.serialize_into())?;
                },
                GitObject::Tree(tree) => {
                    std::fs::create_dir(&object_path)?;
                    tree.checkout(wd, object_path)?;
                },
                _ => return Err(Error::BadTreeFormat.into()),
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
        
        let index_entries = index.range_from_prefix(prefix);
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
                    mode: "040000".to_owned(),
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
            object => Err(Error::UnexpectedObjectFormat(object.get_format()).into()),
        }
    }

    pub fn read_from_commit(wd: &WorkDir, commit_hash: &ObjectHash) -> Result<Tree> {
        let commit = super::Commit::read(wd, commit_hash)?;

        let tree_hash = match commit.map.get("tree") {
            Some(val) => ObjectHash::try_from(&val[..])?,
            None => return Err(Error::BadCommitFormat.into()),
        };

        Self::read(wd, &tree_hash)
    }

    pub fn deserialize(data: Vec<u8>) -> Result<Tree> {
        let mut entries = BTreeMap::new();
        let mut iter = data.into_iter();

        loop {
            let mode = {
                let mode_bytes: Vec<u8> = iter.by_ref()
                    .take_while(|ch| *ch != b' ')
                    .collect();
                String::from_utf8(mode_bytes)?
            };

            if mode.is_empty() {
                break;
            }

            let path = {
                let path: Vec<u8> = iter.by_ref()
                    .take_while(|ch| *ch != 0)
                    .collect();

                WorkPathBuf::try_from(&path[..])?
            };

            let hash = {
                let hash_bytes: Vec<u8> = iter.by_ref().take(20).collect();
                let hash: [u8; 20] = match hash_bytes.try_into() {
                    Ok(val) => val,
                    Err(_) => return Err(Error::BadTreeFormat.into()),
                };

                ObjectHash { raw: hash }
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
        self.mode == "040000"
    }
}
