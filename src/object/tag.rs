use crate::error::Error;

pub struct Tag {

}

impl Tag {
    pub fn deserialize(data: Vec<u8>) -> Result<Tag, Error> {
        Ok(Tag { })
    }

    pub fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }

    pub fn serialize_into(self) -> Vec<u8> {
        Vec::new()
    }
}
