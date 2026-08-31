//! Ending an essay: published, or deliberately in the drawer.
//!
//! Both endings are the same two steps in the same order — move the essay
//! through the lifecycle, then write it — and neither frees the
//! work-in-progress slot as a step of its own. The slot is free afterwards
//! because Published and Shelved simply do not count as in progress
//! (essay-lifecycle spec); there is nothing else to undo.
//!
//! An essay that cannot make the move is refused before anything is written,
//! so a failed ending leaves the file exactly as it was.

use crate::error::Result;
use crate::model::{now, Essay, EssayStatus};
use crate::store::EssayStore;

/// Publishes the essay, recording when it ended and — when the writer has a
/// link to the published piece — where it went.
///
/// The link is optional by design: publishing finishes outside esse, after the
/// text has been copied out, and a URL the writer does not have yet must never
/// block the ending (design.md, D4). Blank input is not a link: the field stays
/// out of the front matter rather than sitting there empty.
pub fn publish(
    essays: &EssayStore,
    mut essay: Essay,
    publication_url: Option<&str>,
) -> Result<Essay> {
    essay.transition_to(EssayStatus::Published)?;
    essay.published_at = Some(now());
    essay.publication_url = publication_url
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(str::to_string);
    essays.save(&essay)?;
    Ok(essay)
}

/// Puts the essay in the drawer, deliberately.
///
/// Nothing leaves Shelved (essay-lifecycle spec), so this is final — which is
/// why the confirmation step belongs above it, in the room where the writer can
/// still see the text.
pub fn shelve(essays: &EssayStore, mut essay: Essay) -> Result<Essay> {
    essay.transition_to(EssayStatus::Shelved)?;
    essays.save(&essay)?;
    Ok(essay)
}
