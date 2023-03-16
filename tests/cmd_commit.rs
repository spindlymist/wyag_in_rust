mod common;
use common::*;

use wyag::commands::{cmd_commit, CommitArgs};

#[test]
fn commit_to_pristine_repo() {
    let test_dir = setup("before_commit_to_pristine_repo", false).unwrap();

    cmd_commit(CommitArgs {
        message: "initial commit".to_owned()
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_commit_to_pristine_repo");
}

#[test]
fn commit() {
    let test_dir = setup("before_commit", false).unwrap();

    cmd_commit(CommitArgs {
        message: "second commit".to_owned()
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_commit");
}
