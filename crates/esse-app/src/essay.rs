//! How the screens speak about an essay.
//!
//! The Shelf and the published row on Today show the same essays; what to call
//! one is decided once, here, so the two cannot drift apart (design.md, D4).

use esse_core::{Essay, WIP_LIMIT};

/// What to call an essay: the spark it grew from, which is how the writer
/// remembers it. A hand-written file without one falls back to its file name.
pub fn title(essay: &Essay) -> String {
    essay.spark.clone().unwrap_or_else(|| essay.slug.clone())
}

/// The WIP limit as a word, because the copy says it aloud: "Three essays are
/// open", never "3 essays are open".
///
/// Derived from the constant rather than typed into each sentence, so that
/// lowering or raising the limit stays the one-constant edit the design
/// promises (design.md, D2).
pub fn limit_in_words() -> &'static str {
    match WIP_LIMIT {
        1 => "One",
        2 => "Two",
        3 => "Three",
        4 => "Four",
        5 => "Five",
        _ => "Several",
    }
}
