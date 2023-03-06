use std::{
    path::{PathBuf},
    str,
};

use sha1::{Sha1, Digest};

use crate::error::Error;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct ObjectHash {
    pub raw: [u8; 20],
}

impl ObjectHash {
    pub fn new(data: impl AsRef<[u8]>) -> ObjectHash {
        let raw = Sha1::new()
            .chain_update(data)
            .finalize()
            .as_slice()
            .try_into()
            .expect("Sha1 hash should always be 20 bytes");

        ObjectHash { raw }
    }

    pub fn to_path(&self) -> PathBuf {
        let string_hash = self.to_string();
        let directory = &string_hash[..2];
        let file = &string_hash[2..];

        [directory, file].iter().collect()
    }
}

impl std::fmt::Display for ObjectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_hash = base16ct::lower::encode_string(&self.raw);
        write!(f, "{string_hash}")
    }
}

impl TryFrom<&str> for ObjectHash {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut raw = [0u8; 20];

        match base16ct::mixed::decode(value, &mut raw) {
            Ok(raw) => {
                if raw.len() != 20 {
                    return Err(Error::InvalidObjectHash.into());
                }
            },
            Err(_) => return Err(Error::InvalidObjectHash.into()),
        };

        Ok(ObjectHash { raw })
    }
}
