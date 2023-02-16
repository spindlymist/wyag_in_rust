use std::{
    path::{PathBuf, Path},
    io::{Read, Write},
    str,
};

use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use regex::Regex;

use crate::{
    Error,
    Result,
    repo::GitRepository, refs::ref_resolve,
};

mod format;
mod blob;
mod commit;
mod hash;
mod tag;
mod tree;

pub use format::ObjectFormat;
pub use blob::Blob;
pub use commit::Commit;
pub use hash::ObjectHash;
pub use tag::Tag;
pub use tree::Tree;

pub enum GitObject {
    Blob(Blob),
    Commit(Commit),
    Tag(Tag),
    Tree(Tree),
}

impl GitObject {
    pub fn get_format(&self) -> ObjectFormat {
        match self {
            GitObject::Blob(_) => ObjectFormat::Blob,
            GitObject::Commit(_) => ObjectFormat::Commit,
            GitObject::Tag(_) => ObjectFormat::Tag,
            GitObject::Tree(_) => ObjectFormat::Tree,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            GitObject::Blob(inner) => inner.serialize(),
            GitObject::Commit(inner) => inner.serialize(),
            GitObject::Tag(inner) => inner.serialize(),
            GitObject::Tree(inner) => inner.serialize(),
        }
    }

    pub fn serialize_into(self) -> Vec<u8> {
        match self {
            GitObject::Blob(inner) => inner.serialize_into(),
            GitObject::Commit(inner) => inner.serialize_into(),
            GitObject::Tag(inner) => inner.serialize_into(),
            GitObject::Tree(inner) => inner.serialize_into(),
        }
    }

    pub fn deserialize(data: Vec<u8>, format: ObjectFormat) -> Result<GitObject> {
        Ok(match format {
            ObjectFormat::Blob => GitObject::Blob(Blob::deserialize(data)?),
            ObjectFormat::Commit => GitObject::Commit(Commit::deserialize(data)?),
            ObjectFormat::Tag => GitObject::Tag(Tag::deserialize(data)?),
            ObjectFormat::Tree => GitObject::Tree(Tree::deserialize(data)?),
        })
    }

    pub fn from_path<P>(path: P, format: ObjectFormat) -> Result<GitObject>
    where
        P: AsRef<Path>
    {
        Self::from_stream(std::fs::File::open(path)?, format)
    }

    pub fn from_stream<R>(mut stream: R, format: ObjectFormat) -> Result<GitObject>
    where
        R: Read
    {
        let mut data = Vec::new();
        stream.read_to_end(&mut data)?;

        Self::deserialize(data, format)
    }

    /// Resolves a `name` to one or more object hashes.
    fn resolve(repo: &GitRepository, id: &str) -> Result<Vec<ObjectHash>> {
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
                let object_dir_name = &id[..2];
                let dir = repo.path(PathBuf::from("objects").join(object_dir_name));
                if dir.exists() {
                    let hashes: Vec<ObjectHash> = std::fs::read_dir(dir)?
                        .collect::<core::result::Result<Vec<std::fs::DirEntry>, _>>()?
                        .into_iter()
                        .map(|file| format!("{object_dir_name}{}", file.file_name().to_string_lossy()))
                        .filter(|hash_string| hash_string.starts_with(id))
                        .filter_map(|hash_string| ObjectHash::try_from(&hash_string[..]).ok())
                        .collect();
                    candidates.extend(hashes);
                }
            }
        }

        if id == "HEAD" {
            candidates.push(ref_resolve(repo, "HEAD")?);
        }

        if let Ok(local_branch) = ref_resolve(repo, PathBuf::from("refs/heads").join(id)) {
            candidates.push(local_branch);
        }

        if let Ok(remote_branch) = ref_resolve(repo, PathBuf::from("refs/remotes").join(id)) {
            candidates.push(remote_branch);
        }

        if let Ok(tag) = ref_resolve(repo, PathBuf::from("refs/tags").join(id)) {
            candidates.push(tag);
        }

        Ok(candidates)
    }

    /// Finds the object in `repo` identified by `id`.
    pub fn find(repo: &GitRepository, id: &str) -> Result<ObjectHash> {
        let candidates = Self::resolve(repo, id)?;

        match candidates.len() {
            0 => Err(Error::BadObjectId),
            1 => Ok(candidates[0]),
            _ => Err(Error::AmbiguousObjectId(candidates)),
        }
    }

    /// Read the object that hashes to `hash` from `repo`.
    pub fn read(repo: &GitRepository, hash: &ObjectHash) -> Result<GitObject> {
        let mut buf = Vec::new(); // TODO perhaps reserve some capacity here?

        // Read and decompress
        {
            let path = PathBuf::from("objects").join(hash.to_path());
            let object_file = repo.open_file(path, None)?;
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
            Err(_) => return Err(Error::InvalidObjectHeader(format!("Malformed object {hash}: couldn't parse header"))),
        };

        let (format, size) = match header.split_once(' ') {
            Some((left, right)) => (ObjectFormat::try_from(left)?, right),
            None => return Err(Error::InvalidObjectHeader(format!("Malformed object {hash}: not enough parts"))),
        };

        let size = match str::parse(size) {
            Ok(val) => val,
            Err(_) => return Err(Error::InvalidObjectHeader(format!("Malformed object {hash}: invalid length"))),
        };

        // Validate size
        let data: Vec<u8> = iter.collect();
        if data.len() != size {
            return Err(Error::InvalidObjectHeader(format!("Malformed object {hash}: incorrect length")));
        }

        Self::deserialize(data, format)
    }

    pub fn hash(&self) -> ObjectHash {
        let (hash, _) = self.prepare_for_storage();

        hash
    }

    pub fn write(&self, repo: &GitRepository) -> Result<ObjectHash> {
        let (hash, data) = self.prepare_for_storage();

        let mut options = std::fs::OpenOptions::new();
        options
            .create(true)
            .write(true)
            .truncate(true);
        let path = PathBuf::from("objects").join(hash.to_path());
        let object_file = repo.open_file(path, Some(&options))?;

        const COMPRESSION_LEVEL: u32 = 6;
        let mut encoder = ZlibEncoder::new(object_file, flate2::Compression::new(COMPRESSION_LEVEL));
        encoder.write_all(&data)?;

        Ok(hash)
    }

    fn prepare_for_storage(&self) -> (ObjectHash, Vec<u8>) {
        let body = self.serialize();

        let format = self.get_format();
        let size = body.len();
        let mut data = format!("{format} {size}\0").into_bytes();
        data.extend(body);

        let hash = ObjectHash::new(&data);

        (hash, data) // TODO refactor so data buffer doesn't have to be copied
                     // perhaps with VecDeque or have serialize return Write
    }

}
