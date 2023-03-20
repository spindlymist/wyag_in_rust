use std::{
    fs::OpenOptions,
    io::{BufRead, Seek, Write},
    path::Path,
    collections::BTreeMap,
};

use anyhow::Context;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    Result,
    object::ObjectHash,
    workdir::{WorkDir, WorkPathBuf, WorkPath}, branch,
};

mod error;
pub use error::IndexError;

pub mod flags;
pub use flags::EntryFlags;

pub mod stats;
pub use stats::FileStats;

pub mod diff;
pub use diff::UnstagedChange;
pub use diff::StagedChange;

/// Data on a single file stored in the index.
pub struct IndexEntry {
    pub stats: FileStats,
    pub hash: ObjectHash,
    pub flags: EntryFlags,
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
    pub entries: BTreeMap<WorkPathBuf, IndexEntry>,
    pub ext_data: Vec<u8>,
}

pub type IndexRange<'a> = std::collections::btree_map::Range<'a, WorkPathBuf, IndexEntry>;

impl Index {

    /// 4-byte signature that begins a valid index file.
    const INDEX_SIGNATURE: [u8; 4] = [b'D', b'I', b'R', b'C'];

    /// Constructs an empty `Index` with the specified version.
    pub fn new(version: Option<u32>) -> Index {
        Index {
            version: version.unwrap_or(2),
            entries: BTreeMap::new(),
            ext_data: Vec::new(),
        }
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
                return Err(IndexError::Corrupt {
                    problem: format!("invalid signature {signature:?} (expected {:?})", Self::INDEX_SIGNATURE),
                }.into());
            }
        }

        // Signature is followed by version number
        let version = reader.read_u32::<BigEndian>()?;
        if version > 3 {
            return Err(IndexError::UnsupportedVersion(version).into());
        }

        // Version number is followed by the number of entries
        let entry_count = reader.read_u32::<BigEndian>()?;

        // Parse entries
        let mut entries = BTreeMap::new();
        for _ in 0..entry_count {
            let (path, entry) = Self::parse_next_entry(reader)?;
            entries.insert(path, entry);
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
    fn parse_next_entry<R>(reader: &mut R) -> Result<(WorkPathBuf, IndexEntry)>
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
                return Err(IndexError::Corrupt {
                    problem: "empty path".to_owned()
                }.into());
            }

            let bytes = &bytes[..bytes.len() - 1]; // drop the null terminator
            
            WorkPathBuf::try_from(bytes)
                .with_context(|| "Invalid path in index".to_owned())?
        };

        // Each entry ends with 0-7 additional NULL bytes to maintain 8-byte alignment
        {
            let entry_len: usize = (reader.stream_position()? - start_pos)
                .try_into()
                .expect("Entry length should not exceed usize::MAX");
            let padding = Self::calc_padding_len(entry_len, true);
            reader.seek(std::io::SeekFrom::Current(padding as i64))?;
        }

        Ok((path, IndexEntry {
            stats,
            hash,
            flags,
        }))
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

    pub fn entries_in_dir(&self, dir: &WorkPath) -> IndexRange {
        if dir.is_empty() {
            return self.entries.range::<WorkPathBuf, std::ops::RangeFull>(..);
        }

        let range_start = std::ops::Bound::Excluded(format!("{dir}/"));
        let range_end = std::ops::Bound::Excluded(format!("{dir}0"));

        self.entries.range((range_start, range_end))
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
        for (path, entry) in &self.entries {
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
            data.write_all(path.as_bytes())?;

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
    pub fn add<P>(&mut self, wd: &WorkDir, path: P) -> Result<()>
    where
        P: AsRef<Path>
    {
        let path = wd.canonicalize_path(path)?;
        let changes = self.list_unstaged_changes(wd, &path, true)?;

        for change in changes.into_iter() {
            match change {
                UnstagedChange::Created { path, stats, hash } => {
                    let flags = EntryFlags::new(path.as_str());
                    self.entries.insert(path, IndexEntry {
                        stats,
                        hash,
                        flags,
                    });
                },
                UnstagedChange::Deleted { path } => {
                    self.entries.remove(&path);
                },
                UnstagedChange::Modified { path, stats, hash } => {
                    let entry = self.entries.get_mut(&path).expect("Path should already exist in index");
                    entry.stats = stats;
                    entry.hash = hash;
                },
            };
        }

        Ok(())
    }

    /// Removes the file or directory at `path` from the index and deletes it from the file system.
    /// 
    /// The index and working directory are required to match the tip of the current branch.
    /// Subdirectories are removed recursively.
    pub fn remove<P>(&mut self, wd: &WorkDir, path: P) -> Result<()>
    where
        P: AsRef<Path>
    {
        let path = wd.canonicalize_path(path)?;

        // Abort if there are staged or unstaged changes
        {
            let unstaged_changes = self.list_unstaged_changes(wd, &path, false)?;
            if !unstaged_changes.is_empty() {
                return Err(IndexError::UncommittedChanges.into());
            }
        }

        {
            let commit_hash = branch::get_current(wd)?.tip(wd)?;
            let staged_changes = self.list_staged_changes(wd, commit_hash.as_ref(), &path)?;
            if !staged_changes.is_empty() {
                return Err(IndexError::UncommittedChanges.into());
            }
        }

        // Delete files and remove them from the index
        if path.as_ref().is_dir() {
            std::fs::remove_dir_all(&path)?;
        }
        else {
            std::fs::remove_file(&path)?;
        }

        if self.entries.contains_key(&path) {
            self.entries.remove(&path);
        }
        else {
            let keys_to_remove: Vec<_> =
                self.entries_in_dir(&path)
                .map(|(key, _)| key)
                .cloned()
                .collect();

            for key in keys_to_remove {
                self.entries.remove(&key);
            }
        }

        Ok(())
    }

    /// Overwrites the index file of `repo` with this index.
    pub fn write(&self, wd: &WorkDir) -> Result<()> {
        if self.entries.is_empty() {
            return Err(IndexError::EmptyIndex.into());
        }

        let mut options = OpenOptions::new();
        options.write(true)
            .create(true)
            .truncate(true);
        let mut index_file = wd.open_git_file("index", Some(&options))?;
    
        let data = self.serialize()?;
        index_file.write_all(&data)?;

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn fake_entry() -> IndexEntry {
        IndexEntry {
            stats: FileStats {
                ctime_s: 0,
                ctime_ns: 0,
                mtime_s: 0,
                mtime_ns: 0,
                dev: 0,
                ino: 0,
                mode: 0,
                uid: 0,
                gid: 0,
                size: 0,
            },
            hash: ObjectHash::new([]),
            flags: EntryFlags::new("hello_world.rs"),
        }
    }

    fn insert_fake_entry(index: &mut Index, path: &str) {
        index.entries.insert(path.try_into().unwrap(), fake_entry());
    }

    #[test]
    fn pad_no_null() {
        let padding = Index::calc_padding_len(8 * 64 - 3, false);
        assert_eq!(padding, 3);
    }

    #[test]
    fn pad_with_null() {
        let padding = Index::calc_padding_len(8 * 64 - 3, true);
        assert_eq!(padding, 3);
    }

    #[test]
    fn pad_multiple_of_8_no_null() {
        let padding = Index::calc_padding_len(8 * 64, false);
        assert_eq!(padding, 8);
    }

    #[test]
    fn pad_multiple_of_8_with_null() {
        let padding = Index::calc_padding_len(8 * 64, true);
        assert_eq!(padding, 0);
    }

    #[test]
    fn entries_in_extant_dir() {
        let mut index = Index {
            version: 0,
            entries: BTreeMap::new(),
            ext_data: vec![]
        };
        insert_fake_entry(&mut index, "main.rs");
        insert_fake_entry(&mut index, "goodbye/world.rs");
        insert_fake_entry(&mut index, "hello/world/ ");
        insert_fake_entry(&mut index, "hello/world/foo.rs");
        insert_fake_entry(&mut index, "hello/world/bar/foo.rs");
        insert_fake_entry(&mut index, "hello/world/bar/baz.rs");
        insert_fake_entry(&mut index, "hello/world/~~~~~");
        insert_fake_entry(&mut index, "hello/world0");
        insert_fake_entry(&mut index, "hello/friend.rs");

        let dir = WorkPathBuf::try_from("hello/world").unwrap();
        let entries: HashSet<_> = index.entries_in_dir(&dir)
            .map(|(key, _)| key)
            .cloned()
            .collect();
    
        let expected: HashSet<WorkPathBuf> = [
            "hello/world/ ".try_into().unwrap(),
            "hello/world/foo.rs".try_into().unwrap(),
            "hello/world/bar/foo.rs".try_into().unwrap(),
            "hello/world/bar/baz.rs".try_into().unwrap(),
            "hello/world/~~~~~".try_into().unwrap(),
        ].into();

        assert_eq!(entries, expected);
    }

    #[test]
    fn entries_in_nonexistent_dir() {
        let mut index = Index {
            version: 0,
            entries: BTreeMap::new(),
            ext_data: vec![]
        };
        insert_fake_entry(&mut index, "main.rs");
        insert_fake_entry(&mut index, "goodbye/world.rs");
        insert_fake_entry(&mut index, "hello/world");
        insert_fake_entry(&mut index, "hello/friend.rs");

        let dir = WorkPathBuf::try_from("hello/world").unwrap();
        let mut entries = index.entries_in_dir(&dir);

        assert!(entries.next().is_none());
    }
}
