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
    Blob {
        data: Vec<u8>,
    },
}

impl GitObject {
    pub fn get_format(&self) -> &'static str {
        match self {
            GitObject::Commit => "commit",
            GitObject::Tree => "tree",
            GitObject::Tag => "tag",
            GitObject::Blob {..} => "blob",
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let format = self.get_format();
        let mut data = match self {
            GitObject::Commit => self.serialize_commit(),
            GitObject::Tree => self.serialize_tree(),
            GitObject::Tag => self.serialize_tag(),
            GitObject::Blob { data } => self.serialize_blob(data),
        };
        
        let size = data.len();

        let header = format!("{format} {size}\0").into_bytes().into_iter();
        data.splice(0..0, header);

        data
    }

    fn serialize_commit(&self) -> Vec<u8> {
        Vec::new()
    }

    fn serialize_tree(&self) -> Vec<u8> {
        Vec::new()
    }

    fn serialize_tag(&self) -> Vec<u8> {
        Vec::new()
    }

    fn serialize_blob(&self, data: &Vec<u8>) -> Vec<u8> {
        Vec::new()
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
        Ok(GitObject::Blob {
            data: Vec::from(data)
        })
    }
}

pub struct ObjectHash {
    raw: [u8; 20],
    string: String,
    path: PathBuf,
}

impl ObjectHash {
    pub fn new(data: impl AsRef<[u8]>) -> ObjectHash {
        let raw = Sha1::new()
            .chain_update(data)
            .finalize()
            .as_slice()
            .try_into()
            .expect("Sha1 hash should always be 20 bytes");
        let string = Self::make_string(&raw);
        let path = Self::make_path(&string);

        ObjectHash { raw, string, path, }
    }

    fn make_string(raw_hash: &[u8; 20]) -> String {
        base16ct::upper::encode_string(raw_hash)
    }

    fn make_path(string_hash: &String) -> PathBuf {
        let directory = &string_hash[..2];
        let file = &string_hash[2..];

        [directory, file].iter().collect()
    }
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

const COMPRESSION_LEVEL: u32 = 6;

fn object_write(repo: &GitRepository, object: &GitObject) -> Result<ObjectHash, Error> {
    let data = object.serialize();
    let hash = ObjectHash::new(&data);

    let mut options = std::fs::OpenOptions::new();
    options
        .create(true)
        .write(true)
        .truncate(true);
    let object_file = repo_open_file(&repo, &hash.path, Some(&options))?;

    let mut encoder = ZlibEncoder::new(object_file, flate2::Compression::new(COMPRESSION_LEVEL));
    encoder.write_all(&data)?;

    Ok(hash)
}
