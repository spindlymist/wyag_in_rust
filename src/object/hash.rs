use std::{
    path::PathBuf,
    str,
};

use sha1::{Sha1, Digest};

use super::ObjectError;

/// An SHA-1 hash used to identify an object stored in a Git repository.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct ObjectHash {
    pub raw: [u8; 20],
}

impl ObjectHash {
    /// Computes the SHA-1 hash of `data`.
    pub fn new(data: impl AsRef<[u8]>) -> ObjectHash {
        let raw = Sha1::new()
            .chain_update(data)
            .finalize()
            .as_slice()
            .try_into()
            .expect("Sha1 hash should always be 20 bytes");

        ObjectHash { raw }
    }

    /// Constructs the path to the object with this hash relative to a repo's
    /// objects directory. When converted to a hex string, the first two digits
    /// are the subdirectory name and the last 38 are the file name.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_to_string() {
        let hash = ObjectHash::try_from([
            0xfb, 0x8b, 0x51, 0x1f, 0x9a, 0x0b, 0xa8, 0xdd, 0x4a, 0xb9,
            0x8d, 0x13, 0x3f, 0xdf, 0x23, 0x0b, 0xbb, 0x6b, 0xa5, 0xff,
        ].as_slice()).unwrap();
        assert_eq!(hash.to_string(), "fb8b511f9a0ba8dd4ab98d133fdf230bbb6ba5ff");
    }

    #[test]
    fn rejects_short_bytes() {
        let result = ObjectHash::try_from([0; 19].as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_long_bytes() {
        let result = ObjectHash::try_from([0; 21].as_slice());
        assert!(result.is_err());
    }

    #[test]
    fn string_to_bytes() {
        let hash = ObjectHash::try_from("fb8b511f9a0ba8dd4ab98d133fdf230bbb6ba5ff").unwrap();
        assert_eq!(hash.raw, [
            0xfb, 0x8b, 0x51, 0x1f, 0x9a, 0x0b, 0xa8, 0xdd, 0x4a, 0xb9,
            0x8d, 0x13, 0x3f, 0xdf, 0x23, 0x0b, 0xbb, 0x6b, 0xa5, 0xff,
        ]);
    }

    #[test]
    fn rejects_short_string() {
        let result = ObjectHash::try_from(str::repeat("a", 39).as_str());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_long_string() {
        let result = ObjectHash::try_from(str::repeat("a", 41).as_str());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_nonhex_string() {
        let result = ObjectHash::try_from(str::repeat("g", 40).as_str());
        assert!(result.is_err());
    }

    #[test]
    fn to_path() {
        use std::path::Component;

        let hash = ObjectHash::try_from("fb8b511f9a0ba8dd4ab98d133fdf230bbb6ba5ff").unwrap();
        let path = hash.to_path();
        let mut components = path.components();
        assert_eq!(components.next(), Some(Component::Normal("fb".as_ref())));
        assert_eq!(components.next(), Some(Component::Normal("8b511f9a0ba8dd4ab98d133fdf230bbb6ba5ff".as_ref())));
        assert_eq!(components.next(), None);
    }
}
