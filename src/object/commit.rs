use ordered_multimap::ListOrderedMultimap;

use crate::{
    Error,
    Result,
    repo::Repository,
    index::Index,
    branch, refs,
};

use super::{Tree, ObjectHash, GitObject, ObjectMetadata};

pub struct Commit {
    pub map: ListOrderedMultimap<String, String>,
}

impl Commit {
    pub fn create(index: &Index, repo: &Repository, meta: ObjectMetadata) -> Result<ObjectHash> {
        let (tree_hash, _) = Tree::create_from_index(index, repo)?;
        let parent_hash = refs::head(repo)?;
    
        let mut map = ListOrderedMultimap::new();
        map.insert("tree".to_owned(), tree_hash.to_string());
        map.insert("parent".to_owned(), parent_hash.to_string());
        map.insert("author".to_owned(), meta.author_line());
        map.insert("committer".to_owned(), meta.author_line());
        map.insert("".to_owned(), meta.message);
    
        let commit = GitObject::Commit(Commit {
            map
        });
        let commit_hash = commit.write(repo)?;
    
        branch::update_current(repo, &commit_hash)?;
    
        Ok(commit_hash)
    }

    pub fn read(repo: &Repository, hash: &ObjectHash) -> Result<Commit> {
        match GitObject::read(repo, &hash)? {
            GitObject::Commit(commit) => Ok(commit),
            object => Err(Error::UnexpectedObjectFormat(object.get_format())),
        }
    }
    
    pub fn deserialize(data: Vec<u8>) -> Result<Commit> {
        let data = match String::from_utf8(data) {
            Ok(data) => data,
            Err(_) => return Err(Error::BadKVLMFormat),
        };
        let map = crate::kvlm::parse(&data)?;

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
