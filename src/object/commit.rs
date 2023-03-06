use anyhow::Context;
use ordered_multimap::ListOrderedMultimap;

use crate::{
    Result,
    workdir::WorkDir,
    index::Index,
    branch, refs,
};

use super::{ObjectError, ObjectFormat, ObjectHash, GitObject, ObjectMetadata, Tree};

pub struct Commit {
    pub map: ListOrderedMultimap<String, String>,
}

impl Commit {
    pub fn create(index: &Index, wd: &WorkDir, meta: ObjectMetadata) -> Result<ObjectHash> {
        let (tree_hash, _) = Tree::create_from_index(index, wd)?;
        let parent_hash = refs::head(wd)?;
    
        let mut map = ListOrderedMultimap::new();
        map.insert("tree".to_owned(), tree_hash.to_string());
        map.insert("parent".to_owned(), parent_hash.to_string());
        map.insert("author".to_owned(), meta.author_line());
        map.insert("committer".to_owned(), meta.author_line());
        map.insert("".to_owned(), meta.message);
    
        let commit = GitObject::Commit(Commit {
            map
        });
        let commit_hash = commit.write(wd)?;
    
        branch::update_current(wd, &commit_hash)?;
    
        Ok(commit_hash)
    }

    pub fn read(wd: &WorkDir, hash: &ObjectHash) -> Result<Commit> {
        match GitObject::read(wd, hash)? {
            GitObject::Commit(commit) => Ok(commit),
            object => Err(ObjectError::UnexpectedFormat {
                format: object.get_format(),
                expected: ObjectFormat::Blob
            }.into()),
        }
    }
    
    pub fn deserialize(data: Vec<u8>) -> Result<Commit> {
        let data = std::str::from_utf8(&data)
            .context("Failed to parse commit")?;
        let map = crate::kvlm::parse(data)?;

        Ok(Commit {
            map,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        crate::kvlm::serialize(&self.map).into_bytes()
    }

    pub fn serialize_into(self) -> Vec<u8> {
        self.serialize()
    }
}
