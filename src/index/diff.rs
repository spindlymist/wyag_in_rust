use std::{
    collections::HashSet,
    fs::File
};

use crate::{
    Result,
    index::{Index, FileStats},
    workdir::{WorkDir, WorkPathBuf, WorkPath},
    object::{GitObject, ObjectHash, Tree, ObjectFormat, TreeEntry},
};

pub enum UnstagedChange {
    Created {
        path: WorkPathBuf,
        stats: FileStats,
        hash: ObjectHash,
    },
    Deleted {
        path: WorkPathBuf,
    },
    Modified {
        path: WorkPathBuf,
        stats: FileStats,
        hash: ObjectHash,
    },
}

pub enum StagedChange {
    Created {
        path: WorkPathBuf,
    },
    Deleted {
        path: WorkPathBuf,
    },
    Modified {
        path: WorkPathBuf,
    },
}

impl Index {
    /// Creates a set of paths from the index entries that match `path`.
    /// 
    /// If `path` is present in the index, the set will contain just that path.
    /// Otherwise, the set will contain all paths that have `path` as an ancestor.
    /// If no such path exists, the set will be empty.
    pub fn expected_keys_for_path<'a>(&'a self, path: &'a WorkPathBuf) -> HashSet<&'a WorkPathBuf> {
        if self.entries.contains_key(path) {
            [path].into()
        }
        else {
            self.entries_in_dir(path)
                .map(|(name, _)| name)
                .collect()
        }
    }

    /// Compares the index to the file or directory at `path` and enumerates the differences.
    /// If `write` is true, new/modified files will be stored in the repo at `wd`.
    pub fn list_unstaged_changes(&self, wd: &WorkDir, path: &WorkPathBuf, write: bool) -> Result<Vec<UnstagedChange>> {
        // cd to the working directory to reduce the amount of path manipulation required
        let prev_working_dir = std::env::current_dir()?;
        std::env::set_current_dir(wd.as_path())?;

        // Create a "checklist" of matching paths in the index to mark off as they are found in the file system
        let mut expected = self.expected_keys_for_path(path);
        let mut changes = vec![];

        // Compare to the file system
        if path.is_empty() {
            for entry in std::fs::read_dir(".")? {
                let path = match WorkPathBuf::try_from(entry?.file_name()) {
                    Ok(val) => val,
                    Err(err) => match err.downcast_ref::<crate::workdir::WorkDirError>() {
                        Some(crate::workdir::WorkDirError::ForbiddenComponent { .. }) => continue,
                        Some(_) | None => return Err(err),
                    },
                };
                self.unstaged_compare_path(wd, path, &mut changes, &mut expected, write)?;
            }
        }
        else {
            self.unstaged_compare_path(wd, path.clone(), &mut changes, &mut expected, write)?;
        }
        
        // Any files that we didn't see while enumerating the file system must have been deleted
        {
            let deletions =
                expected.into_iter().cloned()
                .map(|path| UnstagedChange::Deleted { path });
            changes.extend(deletions);
        }

        // Don't forget to restore the original working directory
        std::env::set_current_dir(prev_working_dir)?;

        Ok(changes)
    }

    /// Lists new/modified file(s) at `path`, appending them to `changes` and removing them from `expected`.
    fn unstaged_compare_path(&self, wd: &WorkDir, path: WorkPathBuf, changes: &mut Vec<UnstagedChange>, expected: &mut HashSet<&WorkPathBuf>, write: bool) -> Result<()> {
        if self.is_path_ignored(&path) {
            return Ok(());
        }

        if path.as_ref().is_file() {
            // Mark this path seen and compare to the index
            expected.remove(&path);
            if let Some(change) = self.unstaged_compare_file(wd, &path, write)? {
                changes.push(change);
            }
        }
        else if path.as_ref().is_dir() {
            // Recurse on each path in the directory
            for entry in std::fs::read_dir(path)? {
                let path = WorkPathBuf::try_from(entry?.path())?;
                self.unstaged_compare_path(wd, path, changes, expected, write)?;
            }
        }

        Ok(())
    }

    /// Determines if the file at `path` is new or has been modified.
    fn unstaged_compare_file(&self, wd: &WorkDir, path: &WorkPath, write: bool) -> Result<Option<UnstagedChange>> {
        let file = File::open(path)?;
        let stats = FileStats::from_file(&file)?;

        if let Some(entry) = self.entries.get(path) {
            // File already exists in the index

            // We can skip it if its stats haven't changed, or if
            // it's been explicitly marked valid by the user
            if entry.flags.get_assume_valid()
                || stats == entry.stats
            {
                return Ok(None);
            }

            // The stats have changed, so we'll check the file's contents
            let object = GitObject::from_stream(file, ObjectFormat::Blob)?;
            let hash = if write {
                object.write(wd)?
            }
            else {
                object.hash()
            };
            
            // Even if the stats are different, this file doesn't count if its
            // contents haven't changed
            if hash == entry.hash {
                return Ok(None);
            }

            
            Ok(Some(UnstagedChange::Modified {
                path: path.to_owned(),
                stats,
                hash,
            }))
        }
        else {
            // New file

            let object = GitObject::from_stream(file, ObjectFormat::Blob)?;
            let hash = if write {
                object.write(wd)?
            }
            else {
                object.hash()
            };

            Ok(Some(UnstagedChange::Created {
                path: path.to_owned(),
                stats,
                hash
            }))
        }
    }

    /// Compares the index to the commit tree identified by `commit_hash` and enumerates the differences.
    pub fn list_staged_changes(&self, wd: &WorkDir, commit_hash: &ObjectHash, path: &WorkPathBuf) -> Result<Vec<StagedChange>>
    {
        let root_tree = Tree::read_from_commit(wd, commit_hash)?;

        // Create a "checklist" of matching paths in the index to mark off as they are found in the commit tree
        let mut expected = self.expected_keys_for_path(path);
        let mut changes = vec![];

        // If no path was provided, start at the root. Otherwise, find the tree that contains
        // the entry associated with that path
        if path.is_empty() {
            for (name, entry) in root_tree.entries {
                self.staged_compare_path(wd, (name, &entry), &mut changes, &mut expected)?;
            }
        }
        else if let Some(entry) = root_tree.find_entry(wd, path)? {
            self.staged_compare_path(wd, (path.clone(), &entry), &mut changes, &mut expected)?;
        }

        // Any files that we didn't see while enumerating the commit tree must be new
        {
            let creations =
                expected.into_iter().cloned()
                .map(|path| StagedChange::Created {
                    path,
                });
            changes.extend(creations);
        }

        Ok(changes)
    }

    /// Lists new/modified file(s) in `tree_entry`, appending them to `changes` and removing them from `expected`.
    /// `path` is the path to this entry relative to the working directory. `tree_entry` may represent a file (blob)
    /// or a directory (tree).
    fn staged_compare_path(&self, wd: &WorkDir, (path, tree_entry): (WorkPathBuf, &TreeEntry), changes: &mut Vec<StagedChange>, expected: &mut HashSet<&WorkPathBuf>) -> Result<()> {
        if tree_entry.is_dir() {
            // Load the subtree from the repo and recurse on each entry
            let subtree = Tree::read(wd, &tree_entry.hash)?;
            for (name, entry) in subtree.entries {
                let subtree_path = path.join(&name);
                self.staged_compare_path(wd, (subtree_path, &entry), changes, expected)?;
            }
        }
        else {
            // Mark this path seen and compare to the index
            expected.remove(&path);
            if let Some(change) = self.staged_compare_file((path, tree_entry)) {
                changes.push(change);
            }
        }

        Ok(())
    }

    /// Determines if the file represented by `tree_entry` has been modified or deleted.
    fn staged_compare_file(&self, (path, tree_entry): (WorkPathBuf, &TreeEntry)) -> Option<StagedChange> {
        if let Some(index_entry) = self.entries.get(&path) {
            // File already exists in the index

            // We can skip it if its contents are unchanged
            if tree_entry.hash == index_entry.hash {
                return None;
            }

            Some(StagedChange::Modified {
                path,
            })
        }
        else {
            // Deleted file
            Some(StagedChange::Deleted {
                path,
            })
        }
    }

    /// Determines if `path` should be excluded from the index.
    /// 
    /// Currently, this just ignores files or directories named .git, but eventually
    /// it should observe the repo's .gitignore file.
    fn is_path_ignored(&self, path: &WorkPath) -> bool {
        path.file_name() == ".git"
    }
}
