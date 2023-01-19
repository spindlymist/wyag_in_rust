use crate::error::Error;

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn deserialize(data: Vec<u8>) -> Result<Blob, Error> {
        Ok(Blob { data })
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn serialize_into(self) -> Vec<u8> {
        self.data
    }
}
