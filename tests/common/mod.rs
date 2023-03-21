use std::{fs::{self, File}, path::{PathBuf, Path}};

pub use assert_fs::{prelude::*, TempDir};

use anyhow::{Result, Context};

pub fn assert_matches_snapshot<P>(actual: P, snapshot: &str)
where
    P: AsRef<Path>
{
    let snapshot = unpack_snapshot(snapshot, false).unwrap();
    assert_paths_match(actual, &snapshot);
}

pub fn assert_paths_match<P1, P2>(actual: P1, expected: P2)
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    assert!(!dir_diff::is_different(actual, expected).unwrap());
}

/// Creates a temporary directory containing a copy of the named snapshot and `cd`s into it.
/// 
/// If `make_subdir` is true, the snapshot will be copied into the subdirectory `name` instead.
/// However, it will still `cd` into the enclosing temporary directory.
pub fn setup(name: &str, make_subdir: bool) -> Result<TempDir> {
    let temp_dir = unpack_snapshot(name, make_subdir)
        .context("Error setting up test environment: failed to unpack snapshot")?;
    std::env::set_current_dir(&temp_dir)
        .context("Error setting up test environment: failed to cd into temporary directory")?;

    Ok(temp_dir)
}

/// Creates an empty temporary directory and `cd`s into it.
#[allow(dead_code)] // not actually dead, but `cargo test` thinks it is
pub fn setup_empty() -> Result<TempDir> {
    let temp_dir = TempDir::new()
        .with_context(|| "Error setting up test environment: failed to create temporary directory".to_string())?;
    std::env::set_current_dir(&temp_dir)
        .context("Error setting up test environment: failed to cd into temporary directory")?;

    Ok(temp_dir)
}

/// Unpacks the snapshot from `name`.7z in the snapshots directory to a temporary directory.
/// 
/// If `make_subdir` is true, the snapshot will be unpacked into a subdirectory called `name`.
pub fn unpack_snapshot(name: &str, make_subdir: bool) -> Result<TempDir> {
    let temp_dir = TempDir::new()
        .with_context(|| format!("Error unpacking snapshot `{name}`: failed to create temporary directory"))?;

    let archive_path: PathBuf = snapshot_path(name);
    let archive_file = File::open(archive_path)
        .with_context(|| format!("Error unpacking snapshot `{name}`: no archive with that name"))?;

    if make_subdir {
        let subdir = temp_dir.child(name);
        fs::create_dir(&subdir)
            .with_context(|| format!("Error unpacking snapshot `{name}`: failed to create subdirectory"))?;
        sevenz_rust::decompress(archive_file, &subdir)
            .with_context(|| format!("Error unpacking snapshot `{name}`: failed to extract archive"))?;
    } else {
        sevenz_rust::decompress(archive_file, &temp_dir)
            .with_context(|| format!("Error unpacking snapshot `{name}`: failed to extract archive"))?;
    }

    Ok(temp_dir)
}

/// Appends the path "snapshots/`name`.7z" to the directory containing Cargo.toml.
fn snapshot_path(name: &str) -> PathBuf {
    [
        env!("CARGO_MANIFEST_DIR"),
        "snapshots",
        &format!("{name}.7z"),
    ].iter().collect()
}
