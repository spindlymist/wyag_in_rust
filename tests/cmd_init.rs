mod common;
use common::*;

use wyag::{
    commands::{cmd_init, InitArgs},
    repo::RepoError
};

#[test]
fn init_fails_on_nonempty_directory() {
    let test_dir = setup("uninitialized", false).unwrap();

    let err = cmd_init(InitArgs {
            path: None,
        })
        .unwrap_err()
        .downcast::<RepoError>()
        .unwrap();
    assert!(matches!(err, RepoError::InitPathExists(_)));

    // The init directory should be unchanged
    assert_matches_snapshot(test_dir, "uninitialized");
}

#[test]
fn init_fails_on_nonempty_subdirectory() {
    let test_dir = setup("uninitialized", true).unwrap();

    let err = cmd_init(InitArgs {
            path: Some("uninitialized".into()),
        })
        .unwrap_err()
        .downcast::<RepoError>()
        .unwrap();
    assert!(matches!(err, RepoError::InitPathExists(_)));

    // The init directory should be unchanged
    assert_matches_snapshot(test_dir.child("uninitialized"), "uninitialized");
}

#[test]
fn init_succeeds_on_empty_directory() {
    let test_dir = setup_empty().unwrap();

    cmd_init(InitArgs {
        path: None
    }).unwrap();

    assert_matches_snapshot(&test_dir, "initialized");
}

#[test]
fn init_succeeds_on_empty_subdirectory() {
    let test_dir = setup_empty().unwrap();

    cmd_init(InitArgs {
        path: Some("empty".into())
    }).unwrap();

    assert_matches_snapshot(test_dir.child("empty"), "initialized");
}
