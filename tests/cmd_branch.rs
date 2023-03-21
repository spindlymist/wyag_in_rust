mod common;
use common::*;

use wyag::{commands::{cmd_branch, BranchArgs}, branch::BranchError};

#[test]
fn create_branch() {
    let test_dir = setup("before_create_branch", false).unwrap();

    cmd_branch(BranchArgs {
        delete: false,
        branch_name: Some("test_branch".to_owned()),
        start_point: "HEAD".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_create_branch");
}

#[test]
fn create_branch_with_starting_point() {
    let test_dir = setup("before_create_branch_with_starting_point", false).unwrap();

    cmd_branch(BranchArgs {
        delete: false,
        branch_name: Some("test_branch".to_owned()),
        start_point: "starting_point".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_create_branch_with_starting_point");
}

#[test]
fn delete_branch() {
    let test_dir = setup("before_delete_branch", false).unwrap();

    cmd_branch(BranchArgs {
        delete: true,
        branch_name: Some("test_branch".to_owned()),
        start_point: "HEAD".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_delete_branch");
}

#[test]
fn delete_fails_with_unmerged_branch() {
    let test_dir = setup("before_delete_fails_with_unmerged_branch", false).unwrap();

    let err = cmd_branch(BranchArgs {
            delete: true,
            branch_name: Some("test_branch".to_owned()),
            start_point: "HEAD".to_owned(),
        })
        .unwrap_err()
        .downcast::<BranchError>()
        .unwrap();

    assert!(matches!(err, BranchError::PossiblyUnmerged(_)));
    assert_matches_snapshot(test_dir, "after_delete_fails_with_unmerged_branch");
}

#[test]
fn delete_fails_with_current_branch() {
    let test_dir = setup("before_delete_fails_with_current_branch", false).unwrap();

    let err = cmd_branch(BranchArgs {
            delete: true,
            branch_name: Some("test_branch".to_owned()),
            start_point: "HEAD".to_owned(),
        })
        .unwrap_err()
        .downcast::<BranchError>()
        .unwrap();

    assert!(matches!(err, BranchError::CheckedOut(_)));
    assert_matches_snapshot(test_dir, "after_delete_fails_with_current_branch");
}
