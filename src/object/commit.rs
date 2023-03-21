use anyhow::Context;
use ordered_multimap::ListOrderedMultimap;

use crate::{
    Result,
    workdir::WorkDir,
    index::Index,
    branch,
};

use super::{ObjectError, ObjectFormat, ObjectHash, GitObject, ObjectMetadata, Tree};

pub struct Commit {
    map: ListOrderedMultimap<String, String>,
    tree: ObjectHash,
    parents: Vec<ObjectHash>,
}

impl Commit {
    pub fn create(index: &Index, wd: &WorkDir, meta: ObjectMetadata) -> Result<ObjectHash> {
        if index.entries.is_empty() {
            return Err(ObjectError::EmptyIndex.into());
        }

        let (tree_hash, _) = Tree::create_from_index(index, wd)?;

        let parent_hash = branch::get_current(wd)?.tip(wd)?;
        let mut parents = Vec::new();
    
        let mut map = ListOrderedMultimap::new();
        map.insert("tree".to_owned(), tree_hash.to_string());
        if let Some(parent_hash) = parent_hash {
            map.insert("parent".to_owned(), parent_hash.to_string());
            parents.push(parent_hash);
        }
        map.insert("author".to_owned(), meta.author_line());
        map.insert("committer".to_owned(), meta.author_line());
        map.insert("".to_owned(), meta.message);
    
        let commit = GitObject::Commit(Commit {
            map,
            tree: tree_hash,
            parents,
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

    pub fn tree(&self) -> &ObjectHash {
        &self.tree
    }

    pub fn parents(&self) -> &[ObjectHash] {
        &self.parents
    }
    
    pub fn deserialize(data: Vec<u8>) -> Result<Commit> {
        let data = std::str::from_utf8(&data)
            .context("Failed to parse commit (invalid Utf-8)")?;
        let map = crate::kvlm::parse(data)?;

        let tree = {
            let hash_string = map.get("tree").context("Failed to parse commit (missing tree)")?;
            ObjectHash::try_from(hash_string.as_str())
                .context("Failed to parse commit (invalid tree hash)")?
        };

        let parents = map.get_all("parent")
            .map(|hash_string| ObjectHash::try_from(hash_string.as_str()))
            .collect::<Result<Vec<_>>>()
            .context("Failed to parse commit (invalid parent hash)")?;

        Ok(Commit {
            map,
            tree,
            parents,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        crate::kvlm::serialize(&self.map).into_bytes()
    }

    pub fn serialize_into(self) -> Vec<u8> {
        self.serialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_tree_hash() {
        let tree_hash = "bf42a97e57f4f7e090ee62e5967e94fc4331dabb";
        let commit_text: Vec<u8> = format!("\
tree {tree_hash}
author spindlymist <ocrobin@gmail.com> 1673643222 -0800
committer spindlymist <ocrobin@gmail.com> 1673643222 -0800

add dependencies and cli skeleton").into();

        let commit = Commit::deserialize(commit_text).unwrap();
        let expected_hash = ObjectHash::try_from(tree_hash).unwrap();
        assert_eq!(commit.tree(), &expected_hash);
    }

    #[test]
    fn rejects_missing_tree() {
        let commit_text = "\
author spindlymist <ocrobin@gmail.com> 1673643222 -0800
committer spindlymist <ocrobin@gmail.com> 1673643222 -0800

add dependencies and cli skeleton".as_bytes().to_owned();

        let result = Commit::deserialize(commit_text);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_tree_hash() {
        let commit_text = "\
tree invalid_hash
author spindlymist <ocrobin@gmail.com> 1673643222 -0800
committer spindlymist <ocrobin@gmail.com> 1673643222 -0800

add dependencies and cli skeleton".as_bytes().to_owned();

        let result = Commit::deserialize(commit_text);
        assert!(result.is_err());
    }

    #[test]
    fn extracts_parent_hashes() {
        use std::collections::HashSet;
        
        let parent1_hash = "0d96eca9c7072cae8f8425e1ffa1ad9c55b75bfe";
        let parent2_hash = "025bfe6f28e0cb39fc982ba8b631bed61cc8a8af";
        let commit_text: Vec<u8> = format!("\
tree 44b9ee4ad7dcff749880b916fc6ee3258cc5e764
parent {parent1_hash}
parent {parent2_hash}
author spindlymist <ocrobin@gmail.com> 1678233745 -0800
committer spindlymist <ocrobin@gmail.com> 1678233745 -0800

add tests for object::hash").into();

        let commit = Commit::deserialize(commit_text).unwrap();

        // Convert to set - order doesn't matter
        let parent_hashes: HashSet<ObjectHash> = commit.parents().iter().cloned().collect();
        let expected_hashes: HashSet<ObjectHash> = [
            ObjectHash::try_from(parent1_hash).unwrap(),
            ObjectHash::try_from(parent2_hash).unwrap()
        ].into();

        assert_eq!(parent_hashes, expected_hashes);
    }

    #[test]
    fn rejects_invalid_parent_hash() {
        let commit_text: Vec<u8> = "\
tree 44b9ee4ad7dcff749880b916fc6ee3258cc5e764
parent invalid_hash
author spindlymist <ocrobin@gmail.com> 1678233745 -0800
committer spindlymist <ocrobin@gmail.com> 1678233745 -0800

add tests for object::hash".to_string().into();

        let result = Commit::deserialize(commit_text);
        assert!(result.is_err());
    }
}
