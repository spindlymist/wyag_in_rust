mod common;
use common::*;

use wyag::commands::{cmd_tag, TagArgs};

#[test]
fn create_lightweight_tag() {
    let test_dir = setup("before_create_lightweight_tag", false).unwrap();

    cmd_tag(TagArgs {
        annotate: false,
        delete: false,
        name: Some("test_tag".to_owned()),
        object: "HEAD".to_owned(),
        message: "".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_create_lightweight_tag");
}


#[test]
fn create_annotated_tag() {
    let test_dir = setup("before_create_annotated_tag", false).unwrap();

    cmd_tag(TagArgs {
        annotate: true,
        delete: false,
        name: Some("test_tag".to_owned()),
        object: "HEAD".to_owned(),
        message: "this is the message".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_create_annotated_tag");
}

#[test]
fn delete_tag() {
    let test_dir = setup("before_delete_tag", false).unwrap();

    cmd_tag(TagArgs {
        annotate: false,
        delete: true,
        name: Some("test_tag".to_owned()),
        object: "HEAD".to_owned(),
        message: "".to_owned(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_delete_tag");
}
