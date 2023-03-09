use std::{fs::{self, File}, path::{PathBuf, Path}};

pub use assert_fs::{prelude::*, TempDir};
pub use dir_assert::assert_paths;
use zip::ZipArchive;
use anyhow::{Result, Context};

/// Creates a temporary directory containing a copy of the named snapshot and `cd`s into it.
/// 
/// If `make_subdir` is true, the snapshot will be copied into the subdirectory `name` instead.
/// However, it will still `cd` into the enclosing temporary directory.
pub fn setup_snapshot(name: &str, make_subdir: bool) -> Result<(TempDir, Snapshot)> {
    let snapshot = Snapshot::named(name)
        .context("Error setting up test environment: failed to load snapshot")?;
    let temp_dir = TempDir::new()
        .context("Error setting up test environment: failed to create temporary directory")?;
    std::env::set_current_dir(&temp_dir)
        .context("Error setting up test environment: failed to cd into temporary directory")?;

    if make_subdir {
        temp_dir.child(name)
            .copy_from(&snapshot, &["**"])
            .context("Error setting up test environment: failed to copy snapshot")?;
    }
    else {
        temp_dir.copy_from(&snapshot, &["**"])
            .context("Error setting up test environment: failed to copy snapshot")?;
    }

    Ok((temp_dir, snapshot))
}

/// A file system snapshot from the snapshots directory adjacent to Cargo.toml. Used like a path.
pub struct Snapshot {
    path: PathBuf,
}

impl Snapshot {
    /// Finds the directory called `name` in the snapshots directory or attempts to unpack the archive
    /// `name`.zip to that directory. On success, returns a `Snapshot` for that directory.
    pub fn named(name: &str) -> Result<Snapshot> {
        let path: PathBuf = Self::snapshot_path(name);

        if !path.is_dir() {
            let zip_path: PathBuf = Self::snapshot_path(&format!("{name}.zip"));
            let zip_file = File::open(zip_path)
                .with_context(|| format!("Error loading snapshot `{name}`: no directory or archive with that name"))?;
            let mut zip_archive = ZipArchive::new(zip_file)
                .with_context(|| format!("Error loading snapshot `{name}`: failed to open archive"))?;
            fs::create_dir(&path)
                .with_context(|| format!("Error loading snapshot `{name}`: failed to create directory"))?;
            zip_archive.extract(&path)
                .with_context(|| format!("Error loading snapshot `{name}`: failed to unpack archive"))?;
        }

        Ok(Snapshot {
            path
        })
    }

    /// Appends the path "snapshots/`name`" to the directory containing Cargo.toml.
    fn snapshot_path(name: &str) -> PathBuf {
        [
            env!("CARGO_MANIFEST_DIR"),
            "snapshots",
            name
        ].iter().collect()
    }
}

impl AsRef<Path> for Snapshot {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
