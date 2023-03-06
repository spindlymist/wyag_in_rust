use std::{
    path::{PathBuf},
    str,
};

use sha1::{Sha1, Digest};

use super::ObjectError;

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
        let hash_string = self.to_string();
        let directory = &hash_string[..2];
        let file = &hash_string[2..];

        [directory, file].iter().collect()
    }
}

impl std::fmt::Display for ObjectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash_string = base16ct::lower::encode_string(&self.raw);
        write!(f, "{hash_string}")
    }
}

impl TryFrom<&str> for ObjectHash {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut raw = [0u8; 20];

        match base16ct::mixed::decode(value, &mut raw) {
            Ok(raw) => {
                if raw.len() != 20 {
                    return Err(ObjectError::InvalidHashString {
                        hash_string: value.to_owned(),
                        problem: format!("expected 20 bytes, got {}", raw.len())
                    }.into());
                }
            },
            Err(_) => return Err(ObjectError::InvalidHashString {
                hash_string: value.to_owned(),
                problem: "not hexadecimal".to_owned(),
            }.into()),
        };

        Ok(ObjectHash { raw })
    }
}

impl TryFrom<&[u8]> for ObjectHash {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let raw: [u8; 20] = value.try_into()
            .map_err(|_| ObjectError::InvalidHashBytes {
                bytes: value.to_owned()
            })?;

        Ok(ObjectHash { raw })
    }
}
