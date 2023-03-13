mod common;
use common::*;

use std::path::PathBuf;

use wyag::commands::{cmd_add, AddArgs};

#[test]
fn add_files_to_index() {
    let test_dir = setup("before_add_files", false).unwrap();

    cmd_add(AddArgs {
        path: PathBuf::from(".")
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_add_files");
}
