use ordered_multimap::ListOrderedMultimap;

use crate::error::Error;

pub struct Commit {
    pub map: ListOrderedMultimap<String, String>,
}

impl Commit {
    pub fn deserialize(data: Vec<u8>) -> Result<Commit, Error> {
        let data = match String::from_utf8(data) {
            Ok(data) => data,
            Err(_) => return Err(Error::BadKVLMFormat),
        };
        let map = crate::kvlm::kvlm_parse(&data)?;

        Ok(Commit {
            map,
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        crate::kvlm::kvlm_serialize(&self.map).into_bytes()
    }

    pub fn serialize_into(self) -> Vec<u8> {
        self.serialize()
    }
}
