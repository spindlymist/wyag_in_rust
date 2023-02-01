use std::{
    io::{BufRead, Seek, Write},
    path::PathBuf,
    collections::BTreeMap,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    error::Error,
    object::ObjectHash,
};

#[derive(PartialEq, Eq)]
pub struct FileStats {
    pub ctime_s: u32,
    pub ctime_ns: u32,
    pub mtime_s: u32,
    pub mtime_ns: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
}

///   A 16-bit 'flags' field split into (high to low bits)
/// 
///     1-bit assume-valid flag
///     1-bit extended flag (must be zero in version 2)
///     2-bit stage (during merge)
///     12-bit name length if the length is less than 0xFFF; otherwise 0xFFF
///     is stored in this field.
///     (Version 3 or later) A 16-bit field, only applicable if the
///     "extended flag" above is 1, split into (high to low bits).
///     1-bit reserved for future
///     1-bit skip-worktree flag (used by sparse checkout)
///     1-bit intent-to-add flag (used by "git add -N")
///     13-bit unused, must be zero
pub struct EntryFlags {
    pub basic_flags: u16,
    pub ext_flags: Option<u16>,
}

const MASK_ASSUME_VALID: u16      = 0b1000_0000_0000_0000;
const MASK_EXTENDED: u16          = 0b0100_0000_0000_0000;
const MASK_STAGE: u16             = 0b0011_0000_0000_0000;
const MASK_NAME_LEN: u16          = 0b0000_1111_1111_1111;
// const MASK_EXT_RESERVED: u16   = 0b1000_0000_0000_0000;
const MASK_EXT_SKIP_WORKTREE: u16 = 0b0100_0000_0000_0000;
const MASK_EXT_INTENT_TO_ADD: u16 = 0b0010_0000_0000_0000;
// const MASK_EXT_UNUSED: u16     = 0b0001_1111_1111_1111;

impl EntryFlags {
    pub fn get_assume_valid(&self) -> bool {
        return (self.basic_flags & MASK_ASSUME_VALID) != 0;
    }

    pub fn set_assume_valid(&mut self) {
        self.basic_flags |= MASK_ASSUME_VALID;
    }

    pub fn clear_assume_valid(&mut self) {
        self.basic_flags &= !MASK_ASSUME_VALID;
    }

    pub fn get_extended(&self) -> bool {
        return (self.basic_flags & MASK_EXTENDED) != 0;
    }

    pub fn set_extended(&mut self) {
        self.basic_flags |= MASK_EXTENDED;
        self.ext_flags = Some(0);
    }

    pub fn clear_extended(&mut self) {
        self.basic_flags &= !MASK_EXTENDED;
        self.ext_flags = None;
    }

    pub fn get_stage(&self) -> () {
        match self.basic_flags & MASK_STAGE {
            0b0000_0000_0000_0000 => (),
            0b0001_0000_0000_0000 => (),
            0b0010_0000_0000_0000 => (),
            0b0011_0000_0000_0000 => (),
            _ => (),
        }
    }

    pub fn set_stage(&mut self, _stage: ()) {
        panic!("not implemented");
    }

    pub fn get_name_len(&self) -> u16 {
        return self.basic_flags & MASK_NAME_LEN;
    }

    pub fn set_name_len(&mut self, value: u16) {
        if value > 0x0FFF {
            panic!("Name len cannot be more than 12 bits");
        }

        self.basic_flags &= !MASK_NAME_LEN;
        self.basic_flags |= value;
    }

    pub fn get_skip_worktree(&self) -> bool {
        return (self.ext_flags.unwrap() & MASK_EXT_SKIP_WORKTREE) != 0;
    }

    pub fn set_skip_worktree(&mut self) {
        *self.ext_flags.as_mut().unwrap() |= MASK_EXT_SKIP_WORKTREE;
    }

    pub fn clear_skip_worktree(&mut self) {
        *self.ext_flags.as_mut().unwrap() &= !MASK_EXT_SKIP_WORKTREE;
    }

    pub fn get_intent_to_add(&self) -> bool {
        return (self.ext_flags.unwrap() & MASK_EXT_INTENT_TO_ADD) != 0;
    }

    pub fn set_intent_to_add(&mut self) {
        *self.ext_flags.as_mut().unwrap() |= MASK_EXT_INTENT_TO_ADD;
    }

    pub fn clear_intent_to_add(&mut self) {
        *self.ext_flags.as_mut().unwrap() &= !MASK_EXT_INTENT_TO_ADD;
    }
}

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

const INDEX_SIGNATURE: [u8; 4] = [b'D', b'I', b'R', b'C'];

pub fn index_parse<R>(reader: &mut R) -> Result<Index, Error>
where
    R: BufRead + Seek
{
    // Validate signature
    {
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;

        if signature != INDEX_SIGNATURE {
            return Err(Error::BadIndexFormat("Invalid signature".to_owned()));
        }
    }

    let version = reader.read_u32::<BigEndian>()?;
    if version > 3 {
        return Err(Error::BadIndexFormat(format!("Unsupported version: {version}")));
    }

    let entry_count = reader.read_u32::<BigEndian>()?;

    let mut entries = BTreeMap::new();
    for _ in 0..entry_count {
        let entry = parse_next_entry(reader)?;
        entries.insert(entry.path.to_string_lossy().to_string(), entry);
    }

    let mut ext_data = Vec::new();
    reader.read_to_end(&mut ext_data)?;

    Ok(Index {
        version,
        entries,
        ext_data,
    })
}

fn parse_next_entry<R>(reader: &mut R) -> Result<IndexEntry, Error>
where
    R: BufRead + Seek
{
    let start_pos = reader.stream_position()?;

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

    let hash = {
        let mut raw = [0u8; 20];
        reader.read_exact(&mut raw)?;

        ObjectHash { raw }
    };

    let flags = {
        let basic_flags = reader.read_u16::<BigEndian>()?;
        let has_ext_flags = (basic_flags & MASK_EXTENDED) != 0;
        let ext_flags = match has_ext_flags {
            true => Some(reader.read_u16::<BigEndian>()?),
            false => None,
        };

        EntryFlags {
            basic_flags,
            ext_flags,
        }
    };

    let path = {
        let mut bytes = vec![];
        if reader.read_until(0, &mut bytes)? < 2 {
            // should read at least one byte + NULL terminator
            return Err(Error::BadIndexFormat("Path is missing".to_owned()));
        }

        let bytes = &bytes[..bytes.len() - 1]; // ignore NULL terminator

        let path = match std::str::from_utf8(bytes) {
            Ok(value) => value,
            Err(_) => return Err(Error::BadIndexFormat("Path is not valid utf8".to_owned())),
        };

        match path {
            ".git" => return Err(Error::BadIndexFormat("Forbidden path".to_owned())),
            ".." => return Err(Error::BadIndexFormat("Forbidden path".to_owned())),
            "." => return Err(Error::BadIndexFormat("Forbidden path".to_owned())),
            _ => (),
        };

        if path.ends_with("/") {
            return Err(Error::BadIndexFormat("Trailing slash is forbidden".to_owned()));
        }

        PathBuf::from(path)
    };

    // entry should end with 0-7 additional NULL bytes (maintaining 8 byte alignment)
    {
        let entry_len: usize = (reader.stream_position()? - start_pos)
            .try_into()
            .expect("Entry length should not exceed usize::MAX");
        let padding = calc_padding_len(entry_len, true);
        reader.seek(std::io::SeekFrom::Current(padding as i64))?;
    }

    Ok(IndexEntry {
        stats,
        hash,
        flags,
        path,
    })
}

const fn calc_padding_len(len: usize, includes_trailing_null: bool) -> usize {
    if includes_trailing_null {
        calc_padding_len(len - 1, false) - 1
    }
    else {
        8 - (len % 8)
    }
}

pub fn index_serialize(index: &Index) -> Result<Vec<u8>, Error> {
    let min_size = index_size_lower_bound(&index);
    let mut data: Vec<u8> = Vec::with_capacity(min_size);

    // serialize header
    data.write_all(&INDEX_SIGNATURE)?;
    data.write_u32::<BigEndian>(index.version)?;
    data.write_u32::<BigEndian>(index.entries.len() as u32)?;

    // serialize entries
    for (_, entry) in &index.entries {
        let start_len = data.len();

        // file stats
        // this could be done faster as a simple memcpy
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
        
        // hash
        data.write_all(&entry.hash.raw)?;

        // flags
        data.write_u16::<BigEndian>(entry.flags.basic_flags)?;
        if let Some(ext_flags) = entry.flags.ext_flags {
            data.write_u16::<BigEndian>(ext_flags)?;
        }

        // path
        data.write_all(entry.path.to_string_lossy().as_bytes())?;

        // padding
        let len = data.len() - start_len;
        let padding = calc_padding_len(len, false);
        data.write_all(&[0; 8][..padding])?;
    }

    // extensions
    data.extend(&index.ext_data);

    Ok(data)
}

fn index_size_lower_bound(index: &Index) -> usize {
    const HEADER_SIZE: usize = {
        const SIGNATURE_SIZE: usize = INDEX_SIGNATURE.len();
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

        ENTRY_MIN_SIZE + calc_padding_len(ENTRY_MIN_SIZE, false)
    };

    HEADER_SIZE + (ENTRY_MIN_SIZE * index.entries.len())
}
