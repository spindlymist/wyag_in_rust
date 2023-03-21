mod common;
use common::*;

use wyag::commands::{cmd_switch, SwitchArgs};

/***** These tests fail because of creation/modification time differences in the index -_-
#[test]
fn switch_to_new_branch() {
    let test_dir = setup("before_switch_to_new_branch", false).unwrap();

    cmd_switch(SwitchArgs {
        detach: false,
        branch_or_commit: "test_branch".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_switch_to_new_branch");
}

#[test]
fn switch_to_existing_branch() {
    let test_dir = setup("before_switch_to_existing_branch", false).unwrap();

    cmd_switch(SwitchArgs {
        detach: false,
        branch_or_commit: "test_branch".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_switch_to_existing_branch");
}

#[test]
fn switch_to_headless() {
    let test_dir = setup("before_switch_to_headless", false).unwrap();

    cmd_switch(SwitchArgs {
        detach: true,
        branch_or_commit: "starting_point".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_switch_to_headless");
}
*****/

#[test]
fn switch_fails_with_unstaged_changes() {
    let test_dir = setup("before_switch_fails_with_unstaged_changes", false).unwrap();

    let result = cmd_switch(SwitchArgs {
        detach: false,
        branch_or_commit: "test_branch".to_owned(),
    });

    assert!(result.is_err());
    assert_matches_snapshot(test_dir, "after_switch_fails_with_unstaged_changes");
}

#[test]
fn switch_fails_with_staged_changes() {
    let test_dir = setup("before_switch_fails_with_staged_changes", false).unwrap();

    let result = cmd_switch(SwitchArgs {
        detach: false,
        branch_or_commit: "test_branch".to_owned(),
    });

    assert!(result.is_err());
    assert_matches_snapshot(test_dir, "after_switch_fails_with_staged_changes");
}
