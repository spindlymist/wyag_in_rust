use crate::error::Error;

pub struct Tree {

}

impl Tree {
    pub fn deserialize(data: Vec<u8>) -> Result<Tree, Error> {
        Ok(Tree { })
    }

    pub fn serialize(&self) -> Vec<u8> {
        Vec::new()
    }

    pub fn serialize_into(self) -> Vec<u8> {
        Vec::new()
    }
}
