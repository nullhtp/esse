//! `ESSE_DATA_DIR` and `HOME` are process-wide state, so they get a test binary
//! to themselves — and one lock, so the two tests cannot overwrite each other's
//! environment mid-run.

use std::env;
use std::fs;
use std::sync::Mutex;

use esse_core::{DataDir, SparkStore};
use tempfile::TempDir;

static ENVIRONMENT: Mutex<()> = Mutex::new(());

#[test]
fn esse_data_dir_replaces_the_platform_location() {
    let _guard = ENVIRONMENT.lock().unwrap_or_else(|error| error.into_inner());
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("elsewhere");
    env::set_var("ESSE_DATA_DIR", &root);

    let dir = DataDir::open().unwrap();
    assert_eq!(dir.root(), root);
    assert!(root.is_dir());

    SparkStore::new(&dir).capture("сюда").unwrap().unwrap();
    assert!(root.join("sparks.jsonl").is_file());
}

/// The folder named at setup, through the whole of `DataDir::open`: nothing in
/// the environment, an answer recorded in the preferences file, and an older
/// installation left exactly where it is (guided-install design.md, D4–D5).
#[test]
fn a_folder_recorded_at_setup_is_opened() {
    let _guard = ENVIRONMENT.lock().unwrap_or_else(|error| error.into_inner());
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    let old = home.join("Library/Application Support/esse");
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("sparks.jsonl"), "{\"text\":\"older\"}\n").unwrap();

    let chosen = temp.path().join("Writing/Essays");
    fs::create_dir_all(chosen.parent().unwrap()).unwrap();
    let preferences = home.join("Library/Preferences");
    fs::create_dir_all(&preferences).unwrap();
    fs::write(
        preferences.join("com.nullhtp.esse.conf"),
        format!("data_dir={}\n", chosen.display()),
    )
    .unwrap();

    env::set_var("HOME", &home);
    env::remove_var("ESSE_DATA_DIR");

    let dir = DataDir::open().unwrap();

    assert_eq!(dir.root(), chosen);
    assert_eq!(fs::read_dir(&chosen).unwrap().count(), 0);
    assert!(old.join("sparks.jsonl").is_file());
}

/// A named path is a deliberate one — a test's directory, a second copy for
/// development. Real essays must never walk into it (design.md, D4).
#[test]
fn a_named_directory_is_never_migrated_into() {
    let _guard = ENVIRONMENT.lock().unwrap_or_else(|error| error.into_inner());
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    let old = home.join("Library/Application Support/esse");
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("sparks.jsonl"), "{\"text\":\"older\"}\n").unwrap();

    let named = temp.path().join("named");
    env::set_var("HOME", &home);
    env::set_var("ESSE_DATA_DIR", &named);

    let dir = DataDir::open().unwrap();

    assert_eq!(dir.root(), named);
    assert_eq!(fs::read_dir(&named).unwrap().count(), 0);
    assert!(old.join("sparks.jsonl").is_file());
}
