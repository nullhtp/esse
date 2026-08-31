//! Turning a spark into an essay — the one operation that touches both stores.
//!
//! Order is the whole point (design.md, D4): the essay is created first and the
//! spark removed only once it exists. A failure in between leaves a duplicate
//! seed, which a person can delete; the other order would lose the idea, which
//! nobody can undo.

use crate::error::{Error, Result};
use crate::model::Essay;
use crate::slug::derive_slug;
use crate::store::{EssayStore, SparkStore};

/// How many suffixed slugs to try before giving up. Reaching the end would
/// mean a hundred essays about the same thing; the error is more useful than
/// the hundred-and-first file.
const MAX_ATTEMPTS: u32 = 99;

/// Starts an essay from the spark with this id, consuming the spark.
///
/// The WIP = 1 refusal comes straight from [`EssayStore::create`] and is passed
/// through untouched — this is not a second way to create an essay, it is the
/// same one with a spark attached.
pub fn start_essay_from_spark(
    sparks: &SparkStore,
    essays: &EssayStore,
    spark_id: &str,
) -> Result<Essay> {
    let spark = sparks
        .load_all()?
        .into_iter()
        .find(|spark| spark.id == spark_id)
        .ok_or_else(|| Error::SparkNotFound {
            id: spark_id.to_string(),
        })?;

    let essay = create_with_free_slug(essays, &spark.text)?;
    // The essay is on disk; the spark has done its job.
    sparks.remove(&spark.id)?;
    Ok(essay)
}

/// Creates the essay under the derived slug, or the first free `-2`, `-3`, …
/// variant of it. Every other refusal — a taken slot above all — comes back
/// as it is.
fn create_with_free_slug(essays: &EssayStore, spark_text: &str) -> Result<Essay> {
    let base = derive_slug(spark_text);
    for attempt in 1..=MAX_ATTEMPTS {
        let slug = if attempt == 1 {
            base.clone()
        } else {
            format!("{base}-{attempt}")
        };
        match essays.create(&slug, Some(spark_text)) {
            Err(Error::SlugTaken { .. }) => continue,
            other => return other,
        }
    }
    Err(Error::SlugTaken { slug: base })
}
