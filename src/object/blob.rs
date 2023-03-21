use crate::{Result, workdir::WorkDir};

use super::{GitObject, ObjectHash, ObjectError, ObjectFormat};

/// A blob of data (usually the contents of a file) that may be stored in and retrieved from a Git repository.
pub struct Blob {
    data: Vec<u8>,
}

impl Blob {
    /// Returns number of bytes stored in the blob.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Reads and parses the blob with the given hash from the repo.
    pub fn read(wd: &WorkDir, hash: &ObjectHash) -> Result<Self> {
        match GitObject::read(wd, hash)? {
            GitObject::Blob(blob) => Ok(blob),
            object => Err(ObjectError::UnexpectedFormat {
                format: object.get_format(),
                expected: ObjectFormat::Blob,
            }.into()),
        }
    }

    /// Parses a `Blob` from a sequence of bytes.
    pub fn deserialize(data: Vec<u8>) -> Result<Blob> {
        Ok(Blob { data })
    }

    /// Converts the blob into a sequence of bytes.
    pub fn serialize(&self) -> Vec<u8> {
        self.data.clone()
    }
    
    /// Consumes the blob and converts it into a sequence of bytes.
    pub fn serialize_into(self) -> Vec<u8> {
        self.data
    }
}
