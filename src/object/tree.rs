use std::{
    path::Path, collections::HashSet
};

use crate::{Error, Result, workdir::{WorkDir, WorkPathBuf, WorkPath}, index::Index};
use super::{ObjectHash, GitObject};

pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: ObjectHash,
}

impl Tree {
    pub fn checkout<P>(&self, wd: &WorkDir, path: P) -> Result<()>
    where
        P: AsRef<Path>
    {
        for entry in &self.entries {
            let object_path = path.as_ref().join(&entry.name);

            match GitObject::read(wd, &entry.hash)? {
                GitObject::Blob(blob) => {
                    std::fs::write(object_path, blob.serialize_into())?;
                },
                GitObject::Tree(tree) => {
                    std::fs::create_dir(&object_path)?;
                    tree.checkout(wd, object_path)?;
                },
                _ => return Err(Error::BadTreeFormat),
            };
        }

        Ok(())
    }

    pub fn create_from_index(index: &Index, wd: &WorkDir) -> Result<(ObjectHash, GitObject)> {
        let prefix = WorkPathBuf::try_from("")?;
        Self::make_subtree(index, wd, &prefix)
    }

    fn make_subtree(index: &Index, wd: &WorkDir, prefix: &WorkPath) -> Result<(ObjectHash, GitObject)> {
        let mut entries = vec![];
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
                    name: name.to_string(),
                    hash: subtree_hash,
                };
                entries.push(tree_entry);
            }
            else {
                let tree_entry = TreeEntry {
                    mode: index_entry.stats.get_mode_string(),
                    name: name.to_string(),
                    hash: index_entry.hash,
                };
                entries.push(tree_entry);
            }
        }

        let tree = GitObject::Tree(Tree { entries });
        let hash = tree.write(wd)?;

        Ok((hash, tree))
    }

    pub fn read(wd: &WorkDir, hash: &ObjectHash) -> Result<Tree> {
        match GitObject::read(wd, hash)? {
            GitObject::Tree(tree) => Ok(tree),
            object => Err(Error::UnexpectedObjectFormat(object.get_format())),
        }
    }

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
