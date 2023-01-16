use std::{
    path::{PathBuf},
    io::{Read, Write},
    str,
};

use sha1::{Sha1, Digest};
use flate2::{read::ZlibDecoder, write::ZlibEncoder};

use crate::{
    error::Error,
    repo::{GitRepository, repo_open_file},
};

pub enum GitObject {
    Commit,
    Tree,
    Tag,
    Blob,
}

impl GitObject {
    pub fn serialize(&self) -> Vec<u8> {
        vec![]
    }

    pub fn deserialize(format: &str, data: &[u8]) -> Result<Self, Error> {
        match format {
            "commit" => Self::deserialize_commit(data),
            "tree" => Self::deserialize_tree(data),
            "tag" => Self::deserialize_tag(data),
            "blob" => Self::deserialize_blob(data),
            _ => Err(Error::UnrecognizedObjectFormat),
        }
    }

    fn deserialize_commit(data: &[u8]) -> Result<Self, Error> {
        Ok(GitObject::Commit)
    }

    fn deserialize_tree(data: &[u8]) -> Result<Self, Error> {
        Ok(GitObject::Tree)
    }

    fn deserialize_tag(data: &[u8]) -> Result<Self, Error> {
        Ok(GitObject::Tag)
    }

    fn deserialize_blob(data: &[u8]) -> Result<Self, Error> {
        Ok(GitObject::Blob)
    }
}

pub struct ObjectHash {
    raw: [u8; 20],
    string: String,
    path: PathBuf,
}

/// Read the object that hashes to `hash` from `repo`.
pub fn object_read(repo: &GitRepository, hash: &ObjectHash) -> Result<GitObject, Error> {
    let mut buf = Vec::new(); // perhaps reserve some capacity here?

    // Read and decompress
    {
        let object_file = repo_open_file(&repo, &hash.path, None)?;
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
        Err(_) => return Err(Error::InvalidObjectHeader(format!("Malformed object {}: couldn't parse header", hash.string))),
    };

    let (format, size) = match header.split_once(' ') {
        Some((left, right)) => (left, right),
        None => return Err(Error::InvalidObjectHeader(format!("Malformed object {}: not enough parts", hash.string))),
    };

    let size = match usize::from_str_radix(size, 10) {
        Ok(val) => val,
        Err(_) => return Err(Error::InvalidObjectHeader(format!("Malformed object {}: invalid length", hash.string))),
    };

    // Validate size
    let data: Vec<u8> = iter.skip(1).collect();
    if data.len() != size {
        return Err(Error::InvalidObjectHeader(format!("Malformed object {}: incorrect length", hash.string)));
    }

    GitObject::deserialize(format, &data)
}
