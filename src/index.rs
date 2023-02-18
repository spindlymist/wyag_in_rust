use std::{
    fs::{File, OpenOptions},
    io::{BufRead, Seek, Write},
    path::{PathBuf, Path},
    collections::BTreeMap,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    Error,
    Result,
    object::{ObjectHash, GitObject, ObjectFormat},
    repo::Repository,
};

pub mod flags;
pub mod stats;
pub use flags::EntryFlags;
pub use stats::FileStats;

/// Data on a single file stored in the index.
pub struct IndexEntry {
    pub stats: FileStats,
    pub hash: ObjectHash,
    pub flags: EntryFlags,
    pub path: PathBuf,
}

/// The index file (or staging area) that git uses to prepare the next commit.
/// 
/// See [the git docs](https://github.com/git/git/blob/master/Documentation/gitformat-index.txt)
/// for detailed information.
/// 
/// This representation supports version 1-3. It does not support version 4.
/// Extensions are not supported.
pub struct Index {
    pub version: u32,
    pub entries: BTreeMap<String, IndexEntry>,
    pub ext_data: Vec<u8>,
}

impl Index {
    /// 4-byte signature that begins a valid index file.
    const INDEX_SIGNATURE: [u8; 4] = [b'D', b'I', b'R', b'C'];

    /// Parses the index file in `repo`.
    pub fn from_repo(repo: &Repository) -> Result<Index> {
        let file = repo.open_git_file("index", None)?;
        let mut buf_reader = std::io::BufReader::new(file);

        Self::parse(&mut buf_reader)
    }

    /// Constructs an `Index` from a byte stream.
    pub fn parse<R>(reader: &mut R) -> Result<Index>
    where
        R: BufRead + Seek
    {
        // Validate signature
        {
            let mut signature = [0u8; 4];
            reader.read_exact(&mut signature)?;

            if signature != Self::INDEX_SIGNATURE {
                return Err(Error::BadIndexFormat("Invalid signature".to_owned()));
            }
        }

        // Signature is followed by version number
        let version = reader.read_u32::<BigEndian>()?;
        if version > 3 {
            return Err(Error::BadIndexFormat(format!("Unsupported version: {version}")));
        }

        // Version number is followed by the number of entries
        let entry_count = reader.read_u32::<BigEndian>()?;

        // Parse entries
        let mut entries = BTreeMap::new();
        for _ in 0..entry_count {
            let entry = Self::parse_next_entry(reader)?;
            entries.insert(entry.path.to_string_lossy().to_string(), entry);
        }

        // Any remaining data is for extensions
        let mut ext_data = Vec::new();
        reader.read_to_end(&mut ext_data)?;

        Ok(Index {
            version,
            entries,
            ext_data,
        })
    }

    /// Parses one index entry from `reader`.
    fn parse_next_entry<R>(reader: &mut R) -> Result<IndexEntry>
    where
        R: BufRead + Seek
    {
        let start_pos = reader.stream_position()?; // used to calculate entry length later

        // Entry begins with file stats
        let stats = FileStats {
            ctime_s: reader.read_u32::<BigEndian>()?,
            ctime_ns: reader.read_u32::<BigEndian>()?,
            mtime_s: reader.read_u32::<BigEndian>()?,
            mtime_ns: reader.read_u32::<BigEndian>()?,
            dev: reader.read_u32::<BigEndian>()?,
            ino: reader.read_u32::<BigEndian>()?,
            mode: reader.read_u32::<BigEndian>()?,
            uid: reader.read_u32::<BigEndian>()?,
            gid: reader.read_u32::<BigEndian>()?,
            size: reader.read_u32::<BigEndian>()?,
        };

        // Stats are followed by the object hash
        let hash = {
            let mut raw = [0u8; 20];
            reader.read_exact(&mut raw)?;

            ObjectHash { raw }
        };

        // Hash is followed by 2-4 bytes of flags
        let flags = {
            let basic_flags = reader.read_u16::<BigEndian>()?;

            let has_ext_flags = (basic_flags & flags::MASK_EXTENDED) != 0;
            let ext_flags = match has_ext_flags {
                true => Some(reader.read_u16::<BigEndian>()?),
                false => None,
            };

            EntryFlags {
                basic_flags,
                ext_flags,
            }
        };

        // Flags are followed by a null-terminated path
        let path = {
            let mut bytes = vec![];
            if reader.read_until(0, &mut bytes)? < 2 {
                // should have read at least one byte + null terminator
                return Err(Error::BadIndexFormat("Path is missing".to_owned()));
            }

            let bytes = &bytes[..bytes.len() - 1]; // drop the null terminator

            let path = match std::str::from_utf8(bytes) {
                Ok(value) => value,
                Err(_) => return Err(Error::BadIndexFormat("Path is not valid utf8".to_owned())),
            };

            match path {
                ".git" => return Err(Error::BadIndexFormat("Forbidden path .git".to_owned())),
                ".." => return Err(Error::BadIndexFormat("Forbidden path ..".to_owned())),
                "." => return Err(Error::BadIndexFormat("Forbidden path .".to_owned())),
                _ => (),
            };

            if path.ends_with('/') {
                return Err(Error::BadIndexFormat("Trailing slash is forbidden".to_owned()));
            }

            PathBuf::from(path)
        };

        // Each entry ends with 0-7 additional NULL bytes to maintain 8-byte alignment
        {
            let entry_len: usize = (reader.stream_position()? - start_pos)
                .try_into()
                .expect("Entry length should not exceed usize::MAX");
            let padding = Self::calc_padding_len(entry_len, true);
            reader.seek(std::io::SeekFrom::Current(padding as i64))?;
        }

        Ok(IndexEntry {
            stats,
            hash,
            flags,
            path,
        })
    }

    /// Calculates the number of null bytes that should follow an index entry of
    /// `len` bytes to maintain 8-byte alignment.
    /// 
    /// `includes_trailing_null` should be true if `len` includes the null terminator
    /// for the path.
    const fn calc_padding_len(len: usize, includes_trailing_null: bool) -> usize {
        if includes_trailing_null {
            Self::calc_padding_len(len - 1, false) - 1
        }
        else {
            8 - (len % 8)
        }
    }

    /// Converts the index into a sequence of bytes.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let min_size = self.size_lower_bound();
        let mut data: Vec<u8> = Vec::with_capacity(min_size);

        // Serialize header
        data.write_all(&Self::INDEX_SIGNATURE)?;
        data.write_u32::<BigEndian>(self.version)?;
        data.write_u32::<BigEndian>(self.entries.len() as u32)?;

        // Serialize entries
        for entry in self.entries.values() {
            let start_len = data.len();

            // File stats
            data.write_u32::<BigEndian>(entry.stats.ctime_s)?;
            data.write_u32::<BigEndian>(entry.stats.ctime_ns)?;
            data.write_u32::<BigEndian>(entry.stats.mtime_s)?;
            data.write_u32::<BigEndian>(entry.stats.mtime_ns)?;
            data.write_u32::<BigEndian>(entry.stats.dev)?;
            data.write_u32::<BigEndian>(entry.stats.ino)?;
            data.write_u32::<BigEndian>(entry.stats.mode)?;
            data.write_u32::<BigEndian>(entry.stats.uid)?;
            data.write_u32::<BigEndian>(entry.stats.gid)?;
            data.write_u32::<BigEndian>(entry.stats.size)?;
            
            // Hash
            data.write_all(&entry.hash.raw)?;

            // Flags
            data.write_u16::<BigEndian>(entry.flags.basic_flags)?;
            if let Some(ext_flags) = entry.flags.ext_flags {
                data.write_u16::<BigEndian>(ext_flags)?;
            }

            // Path
            data.write_all(entry.path.to_string_lossy().as_bytes())?;

            // Padding
            let len = data.len() - start_len;
            let padding = Self::calc_padding_len(len, false);
            data.write_all(&[0; 8][..padding])?;
        }

        // Extensions
        // data.extend(&index.ext_data);

        Ok(data)
    }

    /// Calculates the lower bound on the number of bytes the index will
    /// be serialized into.
    /// 
    /// Assumes that every path is 1 byte.
    fn size_lower_bound(&self) -> usize {
        const HEADER_SIZE: usize = {
            const SIGNATURE_SIZE: usize = Index::INDEX_SIGNATURE.len();
            const VERSION_SIZE: usize = 4;
            const ENTRY_COUNT_SIZE: usize = 4;

            SIGNATURE_SIZE + VERSION_SIZE + ENTRY_COUNT_SIZE
        };

        const ENTRY_MIN_SIZE: usize = {
            const STATS_SIZE: usize = 10 * 4;
            const HASH_SIZE: usize = 20;
            const FLAGS_MIN_SIZE: usize = 2;
            const PATH_MIN_SIZE: usize = 1;

            const ENTRY_MIN_SIZE: usize = STATS_SIZE + HASH_SIZE + FLAGS_MIN_SIZE + PATH_MIN_SIZE;

            ENTRY_MIN_SIZE + Index::calc_padding_len(ENTRY_MIN_SIZE, false)
        };

        HEADER_SIZE + (ENTRY_MIN_SIZE * self.entries.len())
    }

    /// Adds the file or directory at `path` to the index.
    /// 
    /// If `path` is a directory, files in the index that no longer exist
    /// will be removed. Subdirectories will be added recursively.
    pub fn add<P>(&mut self, repo: &Repository, path: P) -> Result<()>
    where
        P: AsRef<Path>
    {
        self.prune_deleted_files(repo, &path)?;
        self.add_path(repo, path)?;

        Ok(())
    }

    /// Removes files from the index that no longer exist.
    fn prune_deleted_files<P>(&mut self, repo: &Repository, path: P) -> Result<()>
    where
        P: AsRef<Path>
    {
        if !path.as_ref().is_dir() {
            return Ok(());
        }

        let dir_name = repo.canonicalize_path(path)?;

        self.entries.retain(|name, _| {
            !name.starts_with(&dir_name)
            || repo.working_path(name).is_file()
        });

        Ok(())
    }

    /// Adds the file or directory at `path` to the index.
    fn add_path<P>(&mut self, repo: &Repository, path: P) -> Result<()>
    where
        P: AsRef<Path>
    {
        if path.as_ref().file_name().unwrap_or_default() == ".git" {
            // Any directories named `.git` should be ignored
            Ok(())
        }
        else if path.as_ref().is_file() {
            self.add_file(repo, path)
        }
        else if path.as_ref().is_dir() {
            for entry in path.as_ref().read_dir()? {
                self.add_path(repo, &entry?.path())?;
            }
            Ok(())
        }
        else {
            Err(Error::InvalidPath)
        }
    }

    /// Adds the file at `path` to the index.
    fn add_file<P>(&mut self, repo: &Repository, path: P) -> Result<()>
    where
        P: AsRef<Path>
    {
        let name = repo.canonicalize_path(&path)?;

        // TODO observe .gitignore
        if name.starts_with("target/") {
            return Ok(());
        }

        let file = File::open(&path)?;
        let stats = FileStats::from_file(&file)?;

        let (object, flags) = if let Some(entry) = self.entries.get(&name) {
            // If the file is already in the index, we can skip it if its stats
            // haven't changed or if it's been explicitly marked valid by the user
            if entry.flags.get_assume_valid() || stats == entry.stats {
                return Ok(());
            }

            let object = GitObject::from_stream(file, ObjectFormat::Blob)?;

            // We can also skip it if the contents haven't changed
            let hash = object.hash();
            if hash == entry.hash {
                return Ok(());
            }

            (object, entry.flags)
        }
        else {
            // New file
            let object = GitObject::from_stream(file, ObjectFormat::Blob)?;
            let flags = EntryFlags::new(&name);

            (object, flags)
        };

        // File is new or modified
        // Store it in the repo and add it to the index
        let hash = object.write(repo)?;
        let entry = IndexEntry {
            stats,
            hash,
            flags,
            path: PathBuf::from(&name),
        };
        self.entries.insert(name, entry);

        Ok(())
    }

    /// Overwrites the index file of `repo` with this index.
    pub fn write(&self, repo: &Repository) -> Result<()> {
        let mut options = OpenOptions::new();
        options.write(true)
            .create(true)
            .truncate(true);
        let mut index_file = repo.open_git_file("index", Some(&options))?;
    
        let data = self.serialize()?;
        index_file.write_all(&data)?;

        Ok(())
    }

}
