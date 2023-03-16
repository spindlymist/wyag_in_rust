mod common;
use common::*;

use std::path::PathBuf;

use wyag::commands::{cmd_add, AddArgs};

#[test]
fn add_all() {
    let test_dir = setup("before_add_all", false).unwrap();

    cmd_add(AddArgs {
        path: PathBuf::from(".")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_add_all");
}

#[test]
fn add_file() {
    let test_dir = setup("before_add_file", false).unwrap();

    cmd_add(AddArgs {
        path: PathBuf::from("c/d/e.txt")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_add_file");
}

#[test]
fn add_directory() {
    let test_dir = setup("before_add_directory", false).unwrap();

    cmd_add(AddArgs {
        path: PathBuf::from("a/b")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_add_directory");
}

#[test]
fn add_all_removed() {
    let test_dir = setup("before_add_all_removed", false).unwrap();

    cmd_add(AddArgs {
        path: PathBuf::from(".")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_add_all_removed");
}

#[test]
fn add_file_removed() {
    let test_dir = setup("before_add_file_removed", false).unwrap();

    cmd_add(AddArgs {
        path: PathBuf::from("x.txt")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_add_file_removed");
}

#[test]
fn add_directory_removed() {
    let test_dir = setup("before_add_directory_removed", false).unwrap();

    cmd_add(AddArgs {
        path: PathBuf::from("a/b")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_add_directory_removed");
}
