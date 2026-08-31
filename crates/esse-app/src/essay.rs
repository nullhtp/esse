//! How the screens speak about an essay.
//!
//! The Shelf and the published row on Today show the same essays; what to call
//! one is decided once, here, so the two cannot drift apart (design.md, D4).

use esse_core::Essay;

/// What to call an essay: the spark it grew from, which is how the writer
/// remembers it. A hand-written file without one falls back to its file name.
pub fn title(essay: &Essay) -> String {
    essay.spark.clone().unwrap_or_else(|| essay.slug.clone())
}
