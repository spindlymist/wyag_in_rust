use ordered_multimap::ListOrderedMultimap;

use crate::{
    Error,
    Result,
    workdir::WorkDir,
    refs,
};

use super::{ObjectHash, GitObject, ObjectMetadata};

pub struct Tag {
    pub map: ListOrderedMultimap<String, String>,
}

impl Tag {
    pub fn create(wd: &WorkDir, name: &str, hash: &ObjectHash, meta: ObjectMetadata) -> Result<Tag>
    {
        let mut map = ListOrderedMultimap::new();
    
        map.insert("object".to_owned(), hash.to_string());
        map.insert("type".to_owned(), "commit".to_owned());
        map.insert("tag".to_owned(), name.to_owned());
        map.insert("tagger".to_owned(), meta.author_line());
        map.insert("".to_owned(), meta.message);
    
        let tag_object = GitObject::Tag(Tag {
            map
        });
        let tag_hash = tag_object.write(wd)?;
    
        Self::create_lightweight(wd, name, &tag_hash)?;
    
        match tag_object {
            GitObject::Tag(tag) => Ok(tag),
            _ => panic!("tag_object should be GitObject::Tag"),
        }
    }
    
    pub fn create_lightweight(wd: &WorkDir, name: &str, hash: &ObjectHash) -> Result<()>
    {
        refs::create(wd, "tags", name, hash)?;
    
        Ok(())
    }

    pub fn delete(wd: &WorkDir, name: &str) -> Result<()> {
        refs::delete(wd, "tags", name)
    }

    pub fn deserialize(data: Vec<u8>) -> Result<Tag> {
        let data = match String::from_utf8(data) {
            Ok(data) => data,
            Err(_) => return Err(Error::BadKVLMFormat),
        };
        let map = crate::kvlm::parse(&data)?;

        Ok(Tag {
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
