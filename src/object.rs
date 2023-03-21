use std::{
    path::{PathBuf, Path},
    io::{Read, Write},
    str,
};

use anyhow::Context;
use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use regex::Regex;

use crate::{
    Result,
    workdir::WorkDir,
    refs,
    branch,
};

mod error;
pub use error::ObjectError;

mod format;
pub use format::ObjectFormat;

mod blob;
pub use blob::Blob;

mod commit;
pub use commit::Commit;

mod hash;
pub use hash::ObjectHash;

mod meta;
pub use meta::ObjectMetadata;

mod tag;
pub use tag::Tag;

mod tree;
pub use tree::{Tree, TreeEntry};

/// An object saved to a Git repository. This may be a commit, a
/// blob (i.e. a file), a tree (i.e. a directory), or a tag.
pub enum GitObject {
    Blob(Blob),
    Commit(Commit),
    Tag(Tag),
    Tree(Tree),
}

impl GitObject {
    /// Returns the format (blob, commit, tag, or tree) of the object.
    pub fn get_format(&self) -> ObjectFormat {
        match self {
            GitObject::Blob(_) => ObjectFormat::Blob,
            GitObject::Commit(_) => ObjectFormat::Commit,
            GitObject::Tag(_) => ObjectFormat::Tag,
            GitObject::Tree(_) => ObjectFormat::Tree,
        }
    }

    /// Converts the object into a sequence of bytes.
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            GitObject::Blob(inner) => inner.serialize(),
            GitObject::Commit(inner) => inner.serialize(),
            GitObject::Tag(inner) => inner.serialize(),
            GitObject::Tree(inner) => inner.serialize(),
        }
    }

    /// Consumes the object and converts it into a sequence of bytes.
    pub fn serialize_into(self) -> Vec<u8> {
        match self {
            GitObject::Blob(inner) => inner.serialize_into(),
            GitObject::Commit(inner) => inner.serialize_into(),
            GitObject::Tag(inner) => inner.serialize_into(),
            GitObject::Tree(inner) => inner.serialize_into(),
        }
    }

    /// Constructs a `GitObject` from a sequence of bytes.
    pub fn deserialize(data: Vec<u8>, format: ObjectFormat) -> Result<GitObject> {
        Ok(match format {
            ObjectFormat::Blob => GitObject::Blob(Blob::deserialize(data)?),
            ObjectFormat::Commit => GitObject::Commit(Commit::deserialize(data)?),
            ObjectFormat::Tag => GitObject::Tag(Tag::deserialize(data)?),
            ObjectFormat::Tree => GitObject::Tree(Tree::deserialize(data)?),
        })
    }

    /// Reads and deserializes the object stored at `path`.
    pub fn from_path<P>(path: P, format: ObjectFormat) -> Result<GitObject>
    where
        P: AsRef<Path>
    {
        Self::from_stream(std::fs::File::open(path)?, format)
    }

    /// Constructs a `GitObject` from a byte stream.
    pub fn from_stream<R>(mut stream: R, format: ObjectFormat) -> Result<GitObject>
    where
        R: Read
    {
        let mut data = Vec::new();
        stream.read_to_end(&mut data)?;

        Self::deserialize(data, format)
    }

    /// Finds the object uniquely identified by `id`.
    /// 
    /// The identifier may be a (possibly abbreviated) hash, a branch name, a tag, or `"HEAD"`.
    pub fn find(wd: &WorkDir, id: &str) -> Result<ObjectHash> {
        let matches = Self::resolve(wd, id)?;

        match matches.len() {
            1 => Ok(matches[0]),
            0 => Err(ObjectError::InvalidId(id.to_owned()).into()),
            _ => Err(ObjectError::AmbiguousId {
                id: id.to_owned(),
                matches,
            }.into()),
        }
    }

    /// Finds all object hashes that `id` could refer to.
    /// 
    /// The identifier may be a (possibly abbreviated) hash, a branch name, a tag, or `"HEAD"`.
    fn resolve(wd: &WorkDir, id: &str) -> Result<Vec<ObjectHash>> {
        let mut candidates = vec![];

        // TODO there should be some way to make this regex static
        let hash_regex: Regex = Regex::new("^[0-9a-fA-F]{4,40}$").expect("Regex should be valid");
        if hash_regex.is_match(id) {
            if id.len() == 40 {
                if let Ok(hash) = ObjectHash::try_from(id) {
                    candidates.push(hash);
                }
            }
            else {
                let dir_name = &id[..2];
                let dir_path = wd.git_path(format!("objects/{dir_name}"));
                if dir_path.exists() {
                    let hashes: Vec<ObjectHash> = std::fs::read_dir(dir_path)?
                        .collect::<core::result::Result<Vec<std::fs::DirEntry>, _>>()?
                        .into_iter()
                        .map(|file| format!("{dir_name}{}", file.file_name().to_string_lossy()))
                        .filter(|hash_string| hash_string.starts_with(id))
                        .filter_map(|hash_string| ObjectHash::try_from(&hash_string[..]).ok())
                        .collect();
                    candidates.extend(hashes);
                }
            }
        }

        if id == "HEAD" {
            let head = branch::get_current(wd)?.tip(wd)?;

            if let Some(head_hash) = head {
                candidates.push(head_hash);
            }
            else {
                return Err(ObjectError::InvalidId(id.to_owned()))
                    .context("HEAD ref could not be resolved. Have you committed to the current branch?");
            }
        }

        if let Ok(local_branch) = refs::resolve(wd, "heads", id) {
            candidates.push(local_branch);
        }

        if let Ok(remote_branch) = refs::resolve(wd, "remotes", id) {
            candidates.push(remote_branch);
        }

        if let Ok(tag) = refs::resolve(wd, "tags", id) {
            candidates.push(tag);
        }

        Ok(candidates)
    }

    /// Reads and parses the object with the given hash from the repo.
    pub fn read(wd: &WorkDir, hash: &ObjectHash) -> Result<GitObject> {
        // Read and decompress
        let mut bytes = {
            let mut buf = Vec::new(); // TODO perhaps reserve some capacity here?
            let path = PathBuf::from("objects").join(hash.to_path());
            let object_file = wd.open_git_file(path, None)?;
            let mut decoder = ZlibDecoder::new(object_file);
            decoder.read_to_end(&mut buf)?;

            buf.into_iter()
        };

        // Parse header
        let (format, size) = {
            let header_bytes: Vec<u8> =
                bytes.by_ref()
                .take_while(|ch| *ch != 0)
                .collect();

            Self::parse_header(&header_bytes)
                .map_err(|problem| ObjectError::MalformedHeader {
                    hash: *hash,
                    problem
                })?
        };

        // Validate size
        let data: Vec<u8> = bytes.collect();
        if data.len() != size {
            return Err(ObjectError::MalformedHeader{
                hash: *hash,
                problem: format!("mismatched size (expected {size}, found {})", data.len()),
            }.into());
        }

        Self::deserialize(data, format)
    }

    /// Parses an object header. The format is `format size\0` where
    /// - `format` is the type of object as one of the followed strings: `"blob"`, `"commit"`, `"tag"`, or `"tree"`
    /// - `size` is the byte size of the object written as a string in base 10
    fn parse_header(bytes: &[u8]) -> core::result::Result<(ObjectFormat, usize), String> {
        let header = str::from_utf8(bytes)
            .map_err(|_| "invalid Utf-8 sequence".to_owned())?;

        if let Some((left, right)) = header.split_once(' ') {
            let format = ObjectFormat::try_from(left)
                .map_err(|err| err.to_string())?;

            let size = str::parse(right)
                .map_err(|_| "failed to parse size".to_owned())?;
    
            Ok((format, size))
        }
        else {
            Err("missing separator".to_owned())
        }
    }

    /// Computes the hash for this object.
    pub fn hash(&self) -> ObjectHash {
        let (hash, _) = self.prepare_for_storage();

        hash
    }

    /// Store the object in the repo.
    pub fn write(&self, wd: &WorkDir) -> Result<ObjectHash> {
        let (hash, data) = self.prepare_for_storage();

        // Skip writing if the file for this hash already exists
        // The contents will be unchanged unless the compression level is changed
        // or in the extremely unlikely event of a hash collision
        let path = PathBuf::from("objects").join(hash.to_path());
        if !wd.git_path(&path).exists() {
            // Compress and write to disk
            let mut options = std::fs::OpenOptions::new();
            options.create(true).write(true);
            let object_file = wd.open_git_file(path, Some(&options))?;
    
            const COMPRESSION_LEVEL: u32 = 6;
            let mut encoder = ZlibEncoder::new(object_file, flate2::Compression::new(COMPRESSION_LEVEL));
            encoder.write_all(&data)?;
        }

        Ok(hash)
    }

    /// Transforms the object to its stored form and computes the hash.
    fn prepare_for_storage(&self) -> (ObjectHash, Vec<u8>) {
        let body = self.serialize();

        let mut data = {
            let format = self.get_format();
            let size = body.len();

            format!("{format} {size}\0").into_bytes()
        };
        data.extend(body);

        let hash = ObjectHash::new(&data);

        (hash, data) // TODO refactor so data buffer doesn't have to be copied
                     // perhaps with VecDeque or have serialize return Write
    }

}
