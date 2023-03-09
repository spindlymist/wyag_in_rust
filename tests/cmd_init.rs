mod common;
use common::*;

use wyag::{
    commands::{cmd_init, InitArgs},
    repo::RepoError
};

#[test]
fn init_fails_on_nonempty_directory() {
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
