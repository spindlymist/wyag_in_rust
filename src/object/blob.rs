use crate::{Result, workdir::WorkDir};

use super::{GitObject, ObjectHash, ObjectError, ObjectFormat};

pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn read(wd: &WorkDir, hash: &ObjectHash) -> Result<Self> {
        match GitObject::read(wd, hash)? {
            GitObject::Blob(blob) => Ok(blob),
            object => Err(ObjectError::UnexpectedFormat {
                format: object.get_format(),
                expected: ObjectFormat::Blob,
            }.into()),
        }
    }

    pub fn deserialize(data: Vec<u8>) -> Result<Blob> {
        Ok(Blob { data })
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn serialize_into(self) -> Vec<u8> {
        self.data
    }
}
