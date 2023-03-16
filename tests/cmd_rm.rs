mod common;
use common::*;

use std::path::PathBuf;

use wyag::{commands::{cmd_rm, RmArgs}, index::IndexError};

#[test]
fn rm_file() {
    let test_dir = setup("before_rm_file", false).unwrap();

    cmd_rm(RmArgs {
        path: PathBuf::from("x.txt")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_rm_file");
}

#[test]
fn rm_directory() {
    let test_dir = setup("before_rm_directory", false).unwrap();

    cmd_rm(RmArgs {
        path: PathBuf::from("a/b")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_rm_directory");
}

#[test]
fn rm_rejects_unstaged_changes() {
    let test_dir = setup("before_rm_rejects_unstaged_changes", false).unwrap();

    let err = cmd_rm(RmArgs {
        path: PathBuf::from("a/b")
    })
        .unwrap_err()
        .downcast::<IndexError>()
        .unwrap();

    assert!(matches!(err, IndexError::UncommittedChanges));
    assert_matches_snapshot(test_dir, "after_rm_rejects_unstaged_changes");
}

#[test]
fn rm_rejects_staged_changes() {
    let test_dir = setup("before_rm_rejects_staged_changes", false).unwrap();

    let err = cmd_rm(RmArgs {
        path: PathBuf::from("a/b")
    })
        .unwrap_err()
        .downcast::<IndexError>()
        .unwrap();

    assert!(matches!(err, IndexError::UncommittedChanges));
    assert_matches_snapshot(test_dir, "after_rm_rejects_staged_changes");
}
