//! The on-disk layer: the data directory, the two JSONL files, and the essay
//! file format with its WIP limit.

use std::fs;

use esse_core::model::now;
use esse_core::{
    publish, shelve, start_essay_from_spark, DataDir, Error, EssayStatus, EssayStore, Session,
    SessionStore, SparkStore, WIP_LIMIT,
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

/// The slugs of a column, in the order the store handed them over.
fn slugs(essays: Vec<esse_core::Essay>) -> Vec<String> {
    essays.into_iter().map(|essay| essay.slug).collect()
}

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
fn three_essays_can_be_in_progress_at_once() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);

    for slug in ["first", "second", "third"] {
        assert_eq!(
            store.create(slug, None).unwrap().status(),
            EssayStatus::Draft
        );
    }

    assert_eq!(slugs(store.in_progress().unwrap()).len(), WIP_LIMIT);
    assert_eq!(store.load_all().unwrap().len(), WIP_LIMIT);
}

#[test]
fn a_fourth_essay_is_refused() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    for slug in ["first", "second", "third"] {
        store.create(slug, None).unwrap();
    }

    let error = store.create("fourth", None).unwrap_err();

    match error {
        Error::TooManyInProgress { limit } => assert_eq!(limit, WIP_LIMIT),
        other => panic!("expected the limit to be named: {other}"),
    }
    // Nothing was created.
    assert!(!store.path("fourth").exists());
    assert_eq!(store.load_all().unwrap().len(), WIP_LIMIT);
}

#[test]
fn editing_also_counts_against_the_limit() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    for slug in ["first", "second", "third"] {
        let mut essay = store.create(slug, None).unwrap();
        essay.transition_to(EssayStatus::Editing).unwrap();
        store.save(&essay).unwrap();
    }

    assert!(store.create("fourth", None).is_err());
    assert_eq!(store.in_progress().unwrap().len(), WIP_LIMIT);
}

#[test]
fn room_returns_once_an_essay_is_finished() {
    for ending in [EssayStatus::Published, EssayStatus::Shelved] {
        let (_temp, dir) = data_dir();
        let store = EssayStore::new(&dir);
        for slug in ["first", "second", "third"] {
            store.create(slug, None).unwrap();
        }

        let mut first = store.load("first").unwrap();
        first.transition_to(ending).unwrap();
        store.save(&first).unwrap();

        // One lane free, and the other two essays untouched.
        assert_eq!(store.in_progress().unwrap().len(), WIP_LIMIT - 1, "{ending}");
        let fourth = store.create("fourth", None).unwrap();
        assert_eq!(fourth.status(), EssayStatus::Draft, "{ending}");
        assert!(store.create("fifth", None).is_err(), "{ending}");
    }
}

#[test]
fn finished_essays_never_count_against_the_limit() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);

    // A pile of endings, made one at a time so the limit is never in the way.
    for slug in ["one", "two", "three", "four", "five"] {
        let essay = store.create(slug, None).unwrap();
        if slug.len() % 2 == 0 {
            publish(&store, essay, None).unwrap();
        } else {
            shelve(&store, essay).unwrap();
        }
    }

    assert!(store.in_progress().unwrap().is_empty());
    for slug in ["first", "second", "third"] {
        store.create(slug, None).unwrap();
    }
    assert_eq!(store.in_progress().unwrap().len(), WIP_LIMIT);
}

#[test]
fn the_essays_in_progress_come_back_most_recently_worked_first() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    for slug in ["first", "second", "third"] {
        store.create(slug, None).unwrap();
    }

    // Timestamps are kept at second precision, so three essays created in one
    // test share theirs. Typing into one is what moves it up the list, and
    // that is what this says out loud.
    let mut second = store.load("second").unwrap();
    second.body = "Текст.\n".to_string();
    second.updated_at = now() + chrono::Duration::seconds(60);
    store.save(&second).unwrap();

    assert_eq!(slugs(store.in_progress().unwrap())[0], "second");
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

// -- ending an essay ------------------------------------------------------

#[test]
fn publishing_records_when_and_where_and_frees_a_lane() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    let mut essay = store.create("first", Some("почему эссе")).unwrap();
    essay.transition_to(EssayStatus::Editing).unwrap();

    let published = publish(&store, essay, Some(" https://example.com/esse ")).unwrap();

    assert_eq!(published.status(), EssayStatus::Published);
    let loaded = store.load("first").unwrap();
    assert_eq!(loaded.status(), EssayStatus::Published);
    assert!(loaded.published_at.is_some());
    // The link is stored trimmed, as it was typed and not as it was pasted.
    assert_eq!(
        loaded.publication_url.as_deref(),
        Some("https://example.com/esse")
    );
    assert!(store.in_progress().unwrap().is_empty());
}

#[test]
fn publishing_without_a_link_leaves_the_field_out() {
    // No link at all, and a field the writer left blank: both are "not
    // published anywhere yet", and neither writes an empty key.
    for link in [None, Some("   ")] {
        let (_temp, dir) = data_dir();
        let store = EssayStore::new(&dir);
        let essay = store.create("first", None).unwrap();

        publish(&store, essay, link).unwrap();

        let written = fs::read_to_string(store.path("first")).unwrap();
        assert!(!written.contains("publication_url"), "{written}");
        let loaded = store.load("first").unwrap();
        assert!(loaded.publication_url.is_none(), "{link:?}");
        assert!(loaded.published_at.is_some(), "{link:?}");
    }
}

#[test]
fn shelving_ends_the_essay_and_frees_a_lane() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    let essay = store.create("first", None).unwrap();

    let shelved = shelve(&store, essay).unwrap();

    assert_eq!(shelved.status(), EssayStatus::Shelved);
    assert_eq!(store.load("first").unwrap().status(), EssayStatus::Shelved);
    assert!(store.published().unwrap().is_empty());
    assert!(store.in_progress().unwrap().is_empty());
}

#[test]
fn an_essay_can_only_end_once() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    let essay = store.create("first", None).unwrap();
    let published = publish(&store, essay, Some("https://example.com/esse")).unwrap();
    let before = fs::read_to_string(store.path("first")).unwrap();

    // Both endings refuse an essay that has already ended, and neither of the
    // refusals touches the file.
    let error = publish(&store, published.clone(), None).unwrap_err();
    assert!(
        matches!(
            error,
            Error::IllegalTransition {
                from: EssayStatus::Published,
                to: EssayStatus::Published
            }
        ),
        "{error}"
    );
    let error = shelve(&store, published).unwrap_err();
    assert!(matches!(error, Error::IllegalTransition { .. }), "{error}");

    assert_eq!(fs::read_to_string(store.path("first")).unwrap(), before);
}

#[test]
fn the_body_travels_without_the_front_matter() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    write_by_hand(&store, "hand-written", HAND_WRITTEN);

    let essay = store.load("hand-written").unwrap();

    assert_eq!(essay.body_markdown(), "# Заголовок\n\nПервый абзац.\n");
    assert!(!essay.body_markdown().contains("+++"));
    assert!(!essay.body_markdown().contains("status"));

    // An essay nobody has written into yet has nothing to hand out.
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    assert!(store
        .create("empty", None)
        .unwrap()
        .body_markdown()
        .is_empty());
}

// -- the shelf's queries ---------------------------------------------------

/// A published essay file, written by hand so its dates can be told apart —
/// `published_at` and `updated_at` deliberately disagree about the order.
fn published_by_hand(store: &EssayStore, slug: &str, published_at: &str, updated_at: &str) {
    write_by_hand(
        store,
        slug,
        &format!(
            "+++\nstatus = \"published\"\ncreated_at = \"2026-01-01T10:00:00+03:00\"\nupdated_at = \"{updated_at}\"\npublished_at = \"{published_at}\"\n+++\nТекст.\n"
        ),
    );
}

#[test]
fn published_essays_come_back_newest_published_first() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);

    // The column is ordered by when each essay ended, so the one touched most
    // recently — a typo fixed by hand months later — keeps its own place.
    published_by_hand(
        &store,
        "first",
        "2026-02-01T10:00:00+03:00",
        "2026-08-01T10:00:00+03:00",
    );
    published_by_hand(
        &store,
        "second",
        "2026-03-01T10:00:00+03:00",
        "2026-03-01T10:00:00+03:00",
    );
    published_by_hand(
        &store,
        "third",
        "2026-04-01T10:00:00+03:00",
        "2026-04-01T10:00:00+03:00",
    );

    let slugs: Vec<_> = store
        .published()
        .unwrap()
        .into_iter()
        .map(|essay| essay.slug)
        .collect();
    assert_eq!(slugs, ["third", "second", "first"]);
}

#[test]
fn the_columns_hold_only_what_belongs_in_them() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);

    let published = store.create("published", None).unwrap();
    publish(&store, published, None).unwrap();
    let shelved = store.create("shelved", None).unwrap();
    shelve(&store, shelved).unwrap();
    let mut editing = store.create("editing", None).unwrap();
    editing.transition_to(EssayStatus::Editing).unwrap();
    store.save(&editing).unwrap();

    assert_eq!(slugs(store.published().unwrap()), ["published"]);
    assert_eq!(slugs(store.shelved().unwrap()), ["shelved"]);
    assert_eq!(slugs(store.in_progress().unwrap()), ["editing"]);
}

#[test]
fn a_hand_published_essay_without_a_date_still_has_a_place() {
    let (_temp, dir) = data_dir();
    let store = EssayStore::new(&dir);
    write_by_hand(
        &store,
        "by-hand",
        "+++\nstatus = \"published\"\ncreated_at = \"2026-01-01T10:00:00+03:00\"\nupdated_at = \"2026-01-02T10:00:00+03:00\"\n+++\nТекст.\n",
    );
    let dated = store.create("dated", None).unwrap();
    publish(&store, dated, None).unwrap();

    let slugs: Vec<_> = store
        .published()
        .unwrap()
        .into_iter()
        .map(|essay| essay.slug)
        .collect();
    // Published today, so the one with no `published_at` falls back to its
    // `updated_at` and sorts below rather than dropping out of the column.
    assert_eq!(slugs, ["dated", "by-hand"]);
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
    for slug in ["odno", "dva", "tri"] {
        essays.create(slug, None).unwrap();
    }
    let spark = sparks.capture("новая мысль").unwrap().unwrap();

    let error = start_essay_from_spark(&sparks, &essays, &spark.id).unwrap_err();

    match error {
        Error::TooManyInProgress { limit } => assert_eq!(limit, WIP_LIMIT),
        other => panic!("expected the limit to be named: {other}"),
    }
    // Creation failed, so the spark is still where it was — an idea is never
    // the price of hitting the limit.
    assert_eq!(sparks.load_all().unwrap(), [spark]);
    assert_eq!(essays.load_all().unwrap().len(), WIP_LIMIT);
}

/// Three lanes at once: what ends, ends alone. Publishing one essay must leave
/// the other two exactly where the writer left them — text, state and all.
#[test]
fn ending_one_essay_leaves_the_others_in_progress() {
    let (_temp, dir) = data_dir();
    let sparks = SparkStore::new(&dir);
    let essays = EssayStore::new(&dir);

    let mut started = Vec::new();
    for text in ["первая мысль", "вторая мысль", "третья мысль"] {
        let spark = sparks.capture(text).unwrap().unwrap();
        started.push(start_essay_from_spark(&sparks, &essays, &spark.id).unwrap());
    }
    assert_eq!(essays.in_progress().unwrap().len(), WIP_LIMIT);

    // One of them is written into and moved on to editing; another is left as
    // the bare draft it was.
    let mut written = essays.load(&started[1].slug).unwrap();
    written.body = "Текст, который нужно сохранить.\n".to_string();
    written.transition_to(EssayStatus::Editing).unwrap();
    essays.save(&written).unwrap();

    publish(&essays, essays.load(&started[0].slug).unwrap(), None).unwrap();

    assert_eq!(essays.in_progress().unwrap().len(), WIP_LIMIT - 1);
    let still_editing = essays.load(&started[1].slug).unwrap();
    assert_eq!(still_editing.status(), EssayStatus::Editing);
    assert_eq!(still_editing.body, "Текст, который нужно сохранить.\n");
    assert_eq!(
        essays.load(&started[2].slug).unwrap().status(),
        EssayStatus::Draft
    );

    // And the freed lane takes one more.
    let spark = sparks.capture("четвёртая мысль").unwrap().unwrap();
    start_essay_from_spark(&sparks, &essays, &spark.id).unwrap();
    assert_eq!(essays.in_progress().unwrap().len(), WIP_LIMIT);
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
