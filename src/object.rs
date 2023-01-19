use std::{
    path::{PathBuf},
    io::{Read, Write},
    str, hash::Hash,
};

use ordered_multimap::ListOrderedMultimap;
use sha1::{Sha1, Digest};
use flate2::{read::ZlibDecoder, write::ZlibEncoder};

use crate::{
    error::Error,
    repo::{GitRepository, repo_open_file},
};

pub enum GitObject {
    Commit {
        // TODO replace with a better model, following tutorial for now
        map: ListOrderedMultimap<String, String>,
    },
    Tree,
    Tag,
    Blob {
        data: Vec<u8>,
    },
}

impl GitObject {
    pub fn get_format(&self) -> &'static str {
        match self {
            GitObject::Commit {..} => "commit",
            GitObject::Tree => "tree",
            GitObject::Tag => "tag",
            GitObject::Blob {..} => "blob",
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            GitObject::Commit { map } => self.serialize_commit(map),
            GitObject::Tree => self.serialize_tree(),
            GitObject::Tag => self.serialize_tag(),
            GitObject::Blob { data } => self.serialize_blob(data),
        }
    }

    fn serialize_commit(&self, map: &ListOrderedMultimap<String, String>) -> Vec<u8> {
        crate::kvlm::kvlm_serialize(&map).into_bytes()
    }

    fn serialize_tree(&self) -> Vec<u8> {
        Vec::new()
    }

    fn serialize_tag(&self) -> Vec<u8> {
        Vec::new()
    }

    fn serialize_blob(&self, data: &Vec<u8>) -> Vec<u8> {
        data.clone()
    }

    pub fn deserialize(format: &str, data: Vec<u8>) -> Result<Self, Error> {
        match format {
            "commit" => Self::deserialize_commit(data),
            "tree" => Self::deserialize_tree(data),
            "tag" => Self::deserialize_tag(data),
            "blob" => Self::deserialize_blob(data),
            _ => Err(Error::UnrecognizedObjectFormat),
        }
    }

    fn deserialize_commit(data: Vec<u8>) -> Result<Self, Error> {
        let data = match String::from_utf8(data) {
            Ok(data) => data,
            Err(_) => return Err(Error::BadKVLMFormat),
        };
        let map = crate::kvlm::kvlm_parse(&data)?;

        Ok(GitObject::Commit {
            map,
        })
    }

    fn deserialize_tree(_data: Vec<u8>) -> Result<Self, Error> {
        Ok(GitObject::Tree)
    }

    fn deserialize_tag(_data: Vec<u8>) -> Result<Self, Error> {
        Ok(GitObject::Tag)
    }

    fn deserialize_blob(data: Vec<u8>) -> Result<Self, Error> {
        Ok(GitObject::Blob {
            data
        })
    }
}

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

    fn to_path(&self) -> PathBuf {
        let string_hash = self.to_string();
        let directory = &string_hash[..2];
        let file = &string_hash[2..];

        [directory, file].iter().collect()
    }
}

impl std::fmt::Display for ObjectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_hash = base16ct::lower::encode_string(&self.raw);
        write!(f, "{}", string_hash)
    }
}

impl TryFrom<&str> for ObjectHash {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut raw = [0u8; 20];

        match base16ct::mixed::decode(value, &mut raw) {
            Ok(raw) => {
                if raw.len() != 20 {
                    return Err(Error::InvalidObjectHash);
                }
            },
            Err(_) => return Err(Error::InvalidObjectHash),
        };

        Ok(ObjectHash { raw })
    }
}

/// Finds the object in `repo` identified by `id`.
pub fn object_find(_repo: &GitRepository, id: &str) -> Result<ObjectHash, Error> {
    // For now, just try to parse id as an object hash
    ObjectHash::try_from(id)
}

/// Read the object that hashes to `hash` from `repo`.
pub fn object_read(repo: &GitRepository, hash: &ObjectHash) -> Result<GitObject, Error> {
    let mut buf = Vec::new(); // TODO perhaps reserve some capacity here?

    // Read and decompress
    {
        let path = PathBuf::from("objects").join(hash.to_path());
        let object_file = repo_open_file(&repo, path, None)?;
        let mut decoder = ZlibDecoder::new(object_file);
        decoder.read_to_end(&mut buf)?;
    }

    let mut iter = buf.into_iter();

    // Parse header "<format> <size>\0" where
    //     <format> is one of those accepted by GitObject::deserialize()
    //     <size> is in ASCII base 10
    let header: Vec<u8> =
        iter.by_ref()
        .take_while(|ch| *ch != 0)
        .collect();

    let header = match str::from_utf8(&header) {
        Ok(val) => val,
        Err(_) => return Err(Error::InvalidObjectHeader(format!("Malformed object {}: couldn't parse header", hash))),
    };

    let (format, size) = match header.split_once(' ') {
        Some((left, right)) => (left, right),
        None => return Err(Error::InvalidObjectHeader(format!("Malformed object {}: not enough parts", hash))),
    };

    let size = match usize::from_str_radix(size, 10) {
        Ok(val) => val,
        Err(_) => return Err(Error::InvalidObjectHeader(format!("Malformed object {}: invalid length", hash))),
    };

    // Validate size
    let data: Vec<u8> = iter.collect();
    if data.len() != size {
        return Err(Error::InvalidObjectHeader(format!("Malformed object {}: incorrect length", hash)));
    }

    GitObject::deserialize(format, data)
}

const COMPRESSION_LEVEL: u32 = 6;

pub fn object_write(repo: &GitRepository, object: &GitObject) -> Result<ObjectHash, Error> {
    let data = object.serialize();
    let format = object.get_format();
    let size = data.len();
    let header = format!("{format} {size}\0").into_bytes();
    let hash = ObjectHash::new(&data);

    let mut options = std::fs::OpenOptions::new();
    options
        .create(true)
        .write(true)
        .truncate(true);
    let path = PathBuf::from("objects").join(hash.to_path());
    let object_file = repo_open_file(&repo, path, Some(&options))?;

    let mut encoder = ZlibEncoder::new(object_file, flate2::Compression::new(COMPRESSION_LEVEL));
    encoder.write_all(&header)?;
    encoder.write_all(&data)?;

    Ok(hash)
}
