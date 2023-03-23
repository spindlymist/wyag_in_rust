mod common;
use common::*;

use wyag::commands::{cmd_hash_object, HashObjectArgs, ClapObjectFormat};

#[test]
fn hash_blob() {
    let test_dir = setup("before_hash_blob", false).unwrap();

    cmd_hash_object(HashObjectArgs {
        write: true,
        format: ClapObjectFormat::Blob,
        path: "a.txt".into(),
    }).unwrap();

    assert_matches_snapshot(test_dir, "after_hash_blob");
}
