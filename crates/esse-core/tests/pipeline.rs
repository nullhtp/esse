//! The whole conveyor, end to end, on a scratch `ESSE_DATA_DIR`: a spark
//! becomes a draft, the draft is written and edited, the essay ends — published
//! or shelved — and a lane is free for the next one.
//!
//! Everything the two editor modes do to an essay happens here through the same
//! core the app calls; what is left over for a person to check is the pointing
//! and clicking. `ESSE_DATA_DIR` is process-wide state, so this gets a test
//! binary to itself.

use std::env;
use std::fs;

use esse_core::{publish, shelve, start_essay_from_spark, DataDir, EssayStatus, EssayStore, SparkStore};
use tempfile::TempDir;

#[test]
fn the_pipeline_runs_from_a_spark_to_an_ending_and_makes_room() {
    let temp = TempDir::new().unwrap();
    env::set_var("ESSE_DATA_DIR", temp.path().join("esse"));
    let dir = DataDir::open().unwrap();
    let sparks = SparkStore::new(&dir);
    let essays = EssayStore::new(&dir);

    // Two ideas in the box; the second is the one to start from.
    sparks.capture("подождёт своей очереди").unwrap().unwrap();
    let spark = sparks.capture("почему эссе, а не посты").unwrap().unwrap();

    // Today → Write: the spark becomes a draft and leaves the box.
    let mut essay = start_essay_from_spark(&sparks, &essays, &spark.id).unwrap();
    assert_eq!(essay.status(), EssayStatus::Draft);
    assert_eq!(sparks.load_all().unwrap().len(), 1);

    // Write mode: the text, saved as it is typed.
    essay.body = "# Почему эссе\n\nПотому что мысль нужно додумать.\n".to_string();
    essays.save(&essay).unwrap();

    // Write → Edit.
    essay.transition_to(EssayStatus::Editing).unwrap();
    essays.save(&essay).unwrap();
    assert_eq!(essays.in_progress().unwrap()[0].slug, essay.slug);

    // The publish panel: the body goes out to the clipboard and to a file,
    // and neither carries the front matter or changes the essay.
    let markdown = essay.body_markdown();
    let exported = temp.path().join("pochemu-esse.md");
    fs::write(&exported, &markdown).unwrap();
    assert_eq!(fs::read_to_string(&exported).unwrap(), markdown);
    assert!(!markdown.contains("+++"), "{markdown}");
    assert!(!markdown.contains("status"), "{markdown}");
    assert_eq!(
        essays.load(&essay.slug).unwrap().status(),
        EssayStatus::Editing
    );

    // Confirmed, with the link the writer pasted back.
    let published = publish(&essays, essay, Some("https://example.com/pochemu-esse")).unwrap();
    let file = fs::read_to_string(essays.path(&published.slug)).unwrap();
    assert!(file.contains("status = \"published\""), "{file}");
    assert!(file.contains("published_at"), "{file}");
    assert!(
        file.contains("publication_url = \"https://example.com/pochemu-esse\""),
        "{file}"
    );
    assert!(file.contains("# Почему эссе"), "{file}");

    // The lane freed itself: the next spark can start.
    assert!(essays.in_progress().unwrap().is_empty());
    let next = sparks.load_all().unwrap().pop().unwrap();
    let second = start_essay_from_spark(&sparks, &essays, &next.id).unwrap();
    assert_eq!(second.status(), EssayStatus::Draft);
    assert!(sparks.load_all().unwrap().is_empty());

    // The other ending, and the lane frees the same way.
    let shelved = shelve(&essays, second).unwrap();
    assert_eq!(shelved.status(), EssayStatus::Shelved);
    assert!(essays.in_progress().unwrap().is_empty());

    // What the Shelf shows at the end of it all: one published essay with its
    // link, one in the drawer, nothing in progress, no sparks left.
    let published = essays.published().unwrap();
    assert_eq!(published.len(), 1);
    assert_eq!(
        published[0].publication_url.as_deref(),
        Some("https://example.com/pochemu-esse")
    );
    let shelved = essays.shelved().unwrap();
    assert_eq!(shelved.len(), 1);
    assert_eq!(shelved[0].spark.as_deref(), Some("подождёт своей очереди"));
}

