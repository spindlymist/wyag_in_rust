mod common;
use common::*;

use wyag::{
    commands::{cmd_init, InitArgs},
    repo::RepoError
};

#[test]
fn init_fails_on_nonempty_directory() {
    let (temp_dir, snapshot) = setup_snapshot("uninitialized", false).unwrap();

    let err = cmd_init(InitArgs {
            path: None,
        })
        .unwrap_err()
        .downcast::<RepoError>()
        .unwrap();
    assert!(matches!(err, RepoError::InitPathExists(_)));

    // The init directory should be unchanged
    assert_paths!(temp_dir, snapshot);
}

#[test]
fn init_fails_on_nonempty_subdirectory() {
    let (temp_dir, snapshot) = setup_snapshot("uninitialized", true).unwrap();

    let err = cmd_init(InitArgs {
            path: Some("uninitialized".into()),
        })
        .unwrap_err()
        .downcast::<RepoError>()
        .unwrap();
    assert!(matches!(err, RepoError::InitPathExists(_)));

    // The init directory should be unchanged
    assert_paths!(temp_dir.child("uninitialized"), snapshot);
}

#[test]
fn init_succeeds_on_empty_directory() {
    let (temp_dir, _) = setup_snapshot("empty", false).unwrap();

    cmd_init(InitArgs { path: None }).unwrap();

    assert_paths!(temp_dir, Snapshot::named("initialized").unwrap());
}

#[test]
fn init_succeeds_on_empty_subdirectory() {
    let (temp_dir, _) = setup_snapshot("empty", true).unwrap();

    cmd_init(InitArgs { path: Some("empty".into()) }).unwrap();

    assert_paths!(temp_dir.child("empty"), Snapshot::named("initialized").unwrap());
}
