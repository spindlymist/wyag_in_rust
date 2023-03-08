use std::path::{PathBuf, Path};

pub use assert_fs::{prelude::*, TempDir};
pub use dir_assert::assert_paths;

/// Creates a temporary directory containing a copy of the named snapshot and `cd`s into it.
/// 
/// If `make_subdir` is true, the snapshot will be copied into `<temp dir>/<snapshot name>` instead.
/// However, it will still `cd` into the enclosing temporary directory.
pub fn setup_snapshot(name: &str, make_subdir: bool) -> (TempDir, Snapshot) {
    let snapshot = Snapshot::named("uninitialized");
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    if make_subdir {
        temp_dir.child(name)
            .copy_from(&snapshot, &["*"]).unwrap();
    }
    else {
        temp_dir.copy_from(&snapshot, &["*"]).unwrap();
    }

    (temp_dir, snapshot)
}

/// A file system snapshot from the `snapshots` directory adjacent to `Cargo.toml`. Used like a path.
pub struct Snapshot {
    path: PathBuf,
}

impl Snapshot {
    pub fn named(name: &str) -> Snapshot {
        Snapshot {
            path: [env!("CARGO_MANIFEST_DIR"), "snapshots", name].iter().collect()
        }
    }
}

impl AsRef<Path> for Snapshot {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
