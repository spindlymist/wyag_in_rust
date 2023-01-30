use std::{
    io::{BufRead, Seek},
    path::PathBuf,
};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    error::Error,
    object::ObjectHash,
};

#[derive(PartialEq, Eq)]
pub struct FileStats {
    ctime_s: u32,
    ctime_ns: u32,
    mtime_s: u32,
    mtime_ns: u32,
    dev: u32,
    ino: u32,
    mode: u32,
    uid: u32,
    gid: u32,
    size: u32,
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
    basic_flags: u16,
    ext_flags: Option<u16>,
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

pub struct Index {
    pub version: u32,
    pub entries: Vec<IndexEntry>,
    // not supported: extension data
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
    let entry_count = reader.read_u32::<BigEndian>()?;

    let mut entries = vec![];
    for _ in 0..entry_count {
        entries.push(parse_next_entry(reader)?);
    }

    Ok(Index {
        version,
        entries,
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

        PathBuf::from(path)
    };

    // entry should end with 0-7 additional NULL bytes (maintaining 8 byte alignment)
    {
        let entry_len = reader.stream_position()? - start_pos - 1;
        let entry_len: i64 = entry_len.try_into().expect("Entry should not be longer than i64::MAX bytes");
        let padding = (8 - (entry_len % 8)) - 1; // -1 because we already read one NULL byte above
        reader.seek(std::io::SeekFrom::Current(padding))?;
    }

    Ok(IndexEntry {
        stats,
        hash,
        flags,
        path
    })
}
