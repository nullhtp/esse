//! The on-disk layer: the data directory, the two JSONL files, and the essay
//! file format with its WIP = 1 invariant.

use std::fs;

use esse_core::model::now;
use esse_core::{
    start_essay_from_spark, DataDir, Error, EssayStatus, EssayStore, Session, SessionStore,
    SparkStore,
};
use tempfile::TempDir;

/// A data directory that does not exist yet, so every test also exercises
/// create-on-first-use.
fn data_dir() -> (TempDir, DataDir) {
    let temp = TempDir::new().unwrap();
    let dir = DataDir::at(temp.path().join("esse")).unwrap();
    (temp, dir)
}

fn lines(path: &std::path::Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

// -- data directory ------------------------------------------------------

#[test]
fn the_data_directory_is_created_on_first_use() {
    let (_temp, dir) = data_dir();

    assert!(dir.root().is_dir());
    assert_eq!(dir.sparks_path(), dir.root().join("sparks.jsonl"));
    assert_eq!(dir.sessions_path(), dir.root().join("sessions.jsonl"));
}

// -- sparks --------------------------------------------------------------

#[test]
fn capturing_appends_one_line_and_leaves_the_others_alone() {
    let (_temp, dir) = data_dir();
    let store = SparkStore::new(&dir);

    store.capture("первая искра").unwrap().unwrap();
    let after_first = lines(&dir.sparks_path());
    store.capture("вторая искра").unwrap().unwrap();
    let after_second = lines(&dir.sparks_path());

    assert_eq!(after_first.len(), 1);
    assert_eq!(after_second.len(), 2);
    assert_eq!(after_second[0], after_first[0]);
    assert!(after_second[1].contains("вторая искра"));
}

#[test]
fn blank_input_is_not_a_spark() {
    let (_temp, dir) = data_dir();
    let store = SparkStore::new(&dir);

    assert!(store.capture("   \t ").unwrap().is_none());
    assert!(store.capture("").unwrap().is_none());
    assert!(!dir.sparks_path().exists());
    assert!(store.load_all().unwrap().is_empty());
}

#[test]
fn sparks_come_back_newest_first() {
    let (_temp, dir) = data_dir();
    let store = SparkStore::new(&dir);

    for text in ["первая", "вторая", "третья"] {
        store.capture(text).unwrap().unwrap();
    }

    let texts: Vec<_> = store
        .load_all()
        .unwrap()
        .into_iter()
        .map(|spark| spark.text)
        .collect();
    assert_eq!(texts, ["третья", "вторая", "первая"]);
}

#[test]
fn sparks_survive_reopening_the_directory() {
    let (_temp, dir) = data_dir();
    SparkStore::new(&dir).capture("пережить перезапуск").unwrap();

    let reopened = DataDir::at(dir.root()).unwrap();
    let sparks = SparkStore::new(&reopened).load_all().unwrap();

    assert_eq!(sparks.len(), 1);
    assert_eq!(sparks[0].text, "пережить перезапуск");
}

#[test]
fn removing_a_spark_leaves_the_others_untouched() {
    let (_temp, dir) = data_dir();
    let store = SparkStore::new(&dir);

    for text in ["первая", "вторая", "третья"] {
        store.capture(text).unwrap().unwrap();
    }
    let middle = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|spark| spark.text == "вторая")
        .unwrap();

    assert!(store.remove(&middle.id).unwrap());

    let texts: Vec<_> = store
        .load_all()
        .unwrap()
        .into_iter()
        .map(|spark| spark.text)
        .collect();
    assert_eq!(texts, ["третья", "первая"]);
    // The rewrite is a whole-file replacement: no temp file survives it.
    let names: Vec<_> = fs::read_dir(dir.root())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(names.is_empty(), "{names:?}");
}

#[test]
fn removing_a_spark_that_is_not_there_changes_nothing() {
    let (_temp, dir) = data_dir();
    let store = SparkStore::new(&dir);
    store.capture("единственная").unwrap().unwrap();
    let before = lines(&dir.sparks_path());

    assert!(!store.remove("no-such-id").unwrap());

    assert_eq!(lines(&dir.sparks_path()), before);
    assert_eq!(store.load_all().unwrap().len(), 1);
}

#[test]
fn a_removed_spark_stays_gone_after_reopening() {
    let (_temp, dir) = data_dir();
    let store = SparkStore::new(&dir);
    let gone = store.capture("исчезнет").unwrap().unwrap();
    store.capture("останется").unwrap().unwrap();

    store.remove(&gone.id).unwrap();

    let reopened = DataDir::at(dir.root()).unwrap();
    let sparks = SparkStore::new(&reopened).load_all().unwrap();
    assert_eq!(sparks.len(), 1);
    assert_eq!(sparks[0].text, "останется");
}

#[test]
fn removing_the_last_spark_leaves_an_empty_box() {
    let (_temp, dir) = data_dir();
    let store = SparkStore::new(&dir);
    let only = store.capture("единственная").unwrap().unwrap();

    assert!(store.remove(&only.id).unwrap());

    assert!(store.load_all().unwrap().is_empty());
    assert_eq!(fs::read_to_string(dir.sparks_path()).unwrap(), "");
}

// -- essays --------------------------------------------------------------

const HAND_WRITTEN: &str = "\
+++
status = \"editing\"
created_at = \"2026-08-30T10:15:00+03:00\"
updated_at = \"2026-08-31T09:00:00+03:00\"
spark = \"почему эссе, а не посты\"

# a field some later version of esse added
mood = \"stubborn\"
+++
# Заголовок

Первый абзац.
";

fn write_by_hand(store: &EssayStore, slug: &str, content: &str) {
    fs::create_dir_all(store.dir()).unwrap();
    fs::write(store.path(slug), content).unwrap();
}

#[test]
fn an_essay_file_round_trips_unchanged() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    write_by_hand(&store, "hand-written", HAND_WRITTEN);

    let essay = store.load("hand-written").unwrap();
    assert_eq!(essay.status(), EssayStatus::Editing);
    assert_eq!(essay.spark.as_deref(), Some("почему эссе, а не посты"));
    assert_eq!(essay.body, "# Заголовок\n\nПервый абзац.\n");

    store.save(&essay).unwrap();

    // Byte for byte, comment and unknown field included.
    let written = fs::read_to_string(store.path("hand-written")).unwrap();
    assert_eq!(written, HAND_WRITTEN);
}

#[test]
fn changing_an_essay_keeps_the_fields_esse_does_not_know() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    write_by_hand(&store, "hand-written", HAND_WRITTEN);

    let mut essay = store.load("hand-written").unwrap();
    essay.transition_to(EssayStatus::Published).unwrap();
    store.save(&essay).unwrap();

    let written = fs::read_to_string(store.path("hand-written")).unwrap();
    assert!(written.contains("status = \"published\""), "{written}");
    assert!(written.contains("mood = \"stubborn\""), "{written}");
    assert!(written.contains("# Заголовок"), "{written}");
    assert_eq!(store.load("hand-written").unwrap().status(), EssayStatus::Published);
}

#[test]
fn saving_leaves_no_temporary_file_behind() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    let essay = store.create("first", Some("искра")).unwrap();
    store.save(&essay).unwrap();

    let names: Vec<_> = fs::read_dir(store.dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["first.md"]);
}

#[test]
fn a_created_essay_starts_as_a_draft_with_its_spark() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);

    let essay = store.create("first", Some("почему эссе")).unwrap();
    assert_eq!(essay.status(), EssayStatus::Draft);

    let loaded = store.load("first").unwrap();
    assert_eq!(loaded.status(), EssayStatus::Draft);
    assert_eq!(loaded.spark.as_deref(), Some("почему эссе"));
    assert_eq!(loaded.created_at, essay.created_at);
    assert!(loaded.body.is_empty());
}

#[test]
fn a_second_essay_is_refused_while_one_is_in_progress() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    store.create("first", None).unwrap();

    let error = store.create("second", None).unwrap_err();

    match error {
        Error::EssayInProgress { slug, status } => {
            assert_eq!(slug, "first");
            assert_eq!(status, EssayStatus::Draft);
        }
        other => panic!("expected the in-progress essay to be named: {other}"),
    }
    // Nothing was created.
    assert!(!store.path("second").exists());
    assert_eq!(store.load_all().unwrap().len(), 1);
}

#[test]
fn editing_also_occupies_the_slot() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    let mut essay = store.create("first", None).unwrap();
    essay.transition_to(EssayStatus::Editing).unwrap();
    store.save(&essay).unwrap();

    assert!(store.create("second", None).is_err());
    assert_eq!(store.in_progress().unwrap().unwrap().slug, "first");
}

#[test]
fn the_slot_frees_once_the_essay_is_finished() {
    for ending in [EssayStatus::Published, EssayStatus::Shelved] {
        let (_temp, dir) = data_dir();
        let store = EssayStore::new(&dir);

        let mut first = store.create("first", None).unwrap();
        first.transition_to(ending).unwrap();
        store.save(&first).unwrap();

        assert!(store.in_progress().unwrap().is_none(), "{ending}");
        let second = store.create("second", None).unwrap();
        assert_eq!(second.status(), EssayStatus::Draft, "{ending}");
        assert_eq!(store.load_all().unwrap().len(), 2, "{ending}");
    }
}

#[test]
fn a_slug_that_is_not_a_safe_file_name_is_refused() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);

    for slug in ["", "../escape", "with/slash", "-"] {
        assert!(store.create(slug, None).is_err(), "{slug}");
    }
}

#[test]
fn a_broken_essay_file_is_reported_with_its_path() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    write_by_hand(&store, "broken", "no front matter here\n");

    let error = store.load("broken").unwrap_err();
    assert!(matches!(error, Error::Format { .. }), "{error}");
    assert!(error.to_string().contains("broken.md"), "{error}");
}

// -- starting an essay from a spark ---------------------------------------

#[test]
fn a_spark_becomes_a_draft_and_leaves_the_box() {
    let (_temp, dir) = data_dir();
    let sparks = SparkStore::new(&dir);
    let essays = EssayStore::new(&dir);
    sparks.capture("другая мысль").unwrap().unwrap();
    let spark = sparks.capture("почему эссе").unwrap().unwrap();

    let essay = start_essay_from_spark(&sparks, &essays, &spark.id).unwrap();

    assert_eq!(essay.slug, "pochemu-esse");
    assert_eq!(essay.status(), EssayStatus::Draft);
    assert_eq!(essay.spark.as_deref(), Some("почему эссе"));
    assert!(essays.path("pochemu-esse").exists());

    let left: Vec<_> = sparks
        .load_all()
        .unwrap()
        .into_iter()
        .map(|spark| spark.text)
        .collect();
    assert_eq!(left, ["другая мысль"]);
}

#[test]
fn a_taken_slug_gets_the_first_free_suffix() {
    let (_temp, dir) = data_dir();
    let sparks = SparkStore::new(&dir);
    let essays = EssayStore::new(&dir);
    // Two finished essays already own the obvious names.
    for slug in ["pochemu-esse", "pochemu-esse-2"] {
        let mut essay = essays.create(slug, None).unwrap();
        essay.transition_to(EssayStatus::Published).unwrap();
        essays.save(&essay).unwrap();
    }
    let spark = sparks.capture("Почему эссе!").unwrap().unwrap();

    let essay = start_essay_from_spark(&sparks, &essays, &spark.id).unwrap();

    assert_eq!(essay.slug, "pochemu-esse-3");
    assert!(sparks.load_all().unwrap().is_empty());
}

#[test]
fn the_wip_refusal_passes_through_and_keeps_the_spark() {
    let (_temp, dir) = data_dir();
    let sparks = SparkStore::new(&dir);
    let essays = EssayStore::new(&dir);
    essays.create("uzhe-pishetsya", None).unwrap();
    let spark = sparks.capture("новая мысль").unwrap().unwrap();

    let error = start_essay_from_spark(&sparks, &essays, &spark.id).unwrap_err();

    match error {
        Error::EssayInProgress { slug, status } => {
            assert_eq!(slug, "uzhe-pishetsya");
            assert_eq!(status, EssayStatus::Draft);
        }
        other => panic!("expected the in-progress essay to be named: {other}"),
    }
    // Creation failed, so the spark is still where it was.
    assert_eq!(sparks.load_all().unwrap(), [spark]);
    assert_eq!(essays.load_all().unwrap().len(), 1);
}

#[test]
fn a_spark_that_is_not_in_the_box_starts_nothing() {
    let (_temp, dir) = data_dir();
    let sparks = SparkStore::new(&dir);
    let essays = EssayStore::new(&dir);

    let error = start_essay_from_spark(&sparks, &essays, "no-such-id").unwrap_err();

    assert!(matches!(error, Error::SparkNotFound { .. }), "{error}");
    assert!(essays.load_all().unwrap().is_empty());
}

#[test]
fn a_spark_with_no_spellable_letters_still_starts_an_essay() {
    let (_temp, dir) = data_dir();
    let sparks = SparkStore::new(&dir);
    let essays = EssayStore::new(&dir);
    let spark = sparks.capture("🔥🔥🔥").unwrap().unwrap();

    let essay = start_essay_from_spark(&sparks, &essays, &spark.id).unwrap();

    assert!(essay.slug.starts_with("essay-"), "{}", essay.slug);
    assert_eq!(essay.spark.as_deref(), Some("🔥🔥🔥"));
    assert!(sparks.load_all().unwrap().is_empty());
}

// -- sessions ------------------------------------------------------------

#[test]
fn a_session_record_round_trips() {
    let (_temp, dir) = data_dir();
    let store = SessionStore::new(&dir);

    let session = Session {
        essay_slug: "first".to_string(),
        started_at: now(),
        duration_min: 22,
    };
    store.append(&session).unwrap();

    assert_eq!(store.load_all().unwrap(), [session]);
}
