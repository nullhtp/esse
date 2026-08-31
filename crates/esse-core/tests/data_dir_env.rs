//! `ESSE_DATA_DIR` is process-wide state, so it gets a test binary to itself.

use std::env;

use esse_core::{DataDir, SparkStore};
use tempfile::TempDir;

#[test]
fn esse_data_dir_replaces_the_platform_location() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("elsewhere");
    env::set_var("ESSE_DATA_DIR", &root);

    let dir = DataDir::open().unwrap();
    assert_eq!(dir.root(), root);
    assert!(root.is_dir());

    SparkStore::new(&dir).capture("сюда").unwrap().unwrap();
    assert!(root.join("sparks.jsonl").is_file());
}
