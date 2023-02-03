use std::{
    fs::File,
    time::SystemTime,
};

use crate::Result;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct FileStats {
    pub (super) ctime_s: u32,
    pub (super) ctime_ns: u32,
    pub (super) mtime_s: u32,
    pub (super) mtime_ns: u32,
    pub (super) dev: u32,
    pub (super) ino: u32,
    pub (super) mode: u32,
    pub (super) uid: u32,
    pub (super) gid: u32,
    pub (super) size: u32,
}

impl FileStats {
    pub fn from_file(file: &File) -> Result<FileStats> {
        let meta = file.metadata()?;

        // ctime does NOT mean creation time on *nix, but git on windows
        // uses the creation time here
        let created_time = meta.created()?
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Timestamp should be after UNIX epoch");

        let modified_time = meta.modified()?
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Timestamp should be after UNIX epoch");

        let size: u32 = meta.len().try_into().expect("File size should fit into u32");

        Ok(FileStats {
            ctime_s: created_time.as_secs().try_into().expect("Timestamp should fit into u32"),
            ctime_ns: created_time.subsec_nanos(),
            mtime_s: modified_time.as_secs().try_into().expect("Timestamp should fit into u32"),
            mtime_ns: modified_time.subsec_nanos(),
            dev: 0, // only used on *nix
            ino: 0, // only used on *nix
            mode: 33188, // TODO figure out how git fills this field on Windows
            uid: 0, // only used on *nix
            gid: 0, // only used on *nix
            size,
        })
    }

    pub fn get_mode_string(&self) -> String {
        format!("{:06o}", self.mode)
    }
}
