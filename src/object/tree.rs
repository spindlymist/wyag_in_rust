use std::{
    path::Path
};

use crate::{Error, Result, workdir::{WorkDir, WorkPathBuf}, index::Index};
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
        Self::make_subtree(index, wd, &WorkPathBuf::try_from("")?)
    }

    fn make_subtree(_index: &Index, _wd: &WorkDir, _prefix: &WorkPathBuf) -> Result<(ObjectHash, GitObject)> {
        todo!()
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
