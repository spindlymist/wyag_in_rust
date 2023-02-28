use std::{
    path::Path,
    fs::File
};

use crate::{
    Result,
    index::{Index, FileStats},
    workdir::{WorkDir, WorkPathBuf},
    object::{GitObject, ObjectFormat},
};

pub enum Mutation {
    Created {
        path: WorkPathBuf,
        stats: FileStats,
    },
    Deleted {
        path: WorkPathBuf,
    },
    Modified {
        path: WorkPathBuf,
        stats: FileStats,
        object: GitObject,
    },
}

impl Index {
    /// Compares the index to the directory at `path` and enumerates the differences.
    pub(super) fn diff_with_dir<P>(&self, wd: &WorkDir, path: P) -> Result<Vec<Mutation>>
    where
        P: AsRef<Path>
    {
        let path = wd.canonicalize_path(path)?;

        let prev_working_dir = std::env::current_dir()?;
        std::env::set_current_dir(wd.as_path())?;

        let mut changes = vec![];

        self.find_deletions(path.clone(), &mut changes)?;
        self.find_mutations(path, &mut changes)?;

        std::env::set_current_dir(prev_working_dir)?;

        Ok(changes)
    }

    /// Enumerates the files deleted from the directory at `path`.
    fn find_deletions(&self, _path: WorkPathBuf, _changes: &mut Vec<Mutation>) -> Result<()> {
        todo!()
    }

    /// Enumerates the files created or modified in the directory at `path`.
    fn find_mutations(&self, _path: WorkPathBuf, _changes: &mut Vec<Mutation>) -> Result<()>
    {
        todo!()
    }

    /// Determines if the file at `path` is new or has been modified.
    fn _compare_file(&self, path: WorkPathBuf) -> Result<Option<Mutation>> {
        let file = File::open(&path)?;
        let stats = FileStats::from_file(&file)?;

        let mutation = if let Some(entry) = self.entries.get(&path) {
            // If the file is already in the index, we can skip it if its stats
            // haven't changed or if it's been explicitly marked valid by the user
            if entry.flags.get_assume_valid() || stats == entry.stats {
                return Ok(None);
            }

            let object = GitObject::from_stream(file, ObjectFormat::Blob)?;

            // We can also skip it if the contents haven't changed
            let hash = object.hash();
            if hash == entry.hash {
                return Ok(None);
            }

            Mutation::Modified {
                path,
                stats,
                object,
            }
        }
        else {
            // New file
            Mutation::Created {
                path,
                stats,
            }
        };

        Ok(Some(mutation))
    }

}
