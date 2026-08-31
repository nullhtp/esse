//! The domain: sparks, essays and the states an essay moves through.

use std::fmt;

use chrono::{DateTime, FixedOffset, Local, Timelike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};

/// Every timestamp esse stores: RFC 3339, at second precision, carrying the
/// offset it was written at.
///
/// The offset is kept rather than normalised to the current machine's zone, so
/// a file always reads back exactly as it was written — a record of when the
/// writing happened, in the writer's own time. Sub-second digits are dropped:
/// they are noise in a file a person edits by hand, and dropping them lets a
/// value survive a write and read back unchanged.
pub type Timestamp = DateTime<FixedOffset>;

/// The current time as a [`Timestamp`].
pub fn now() -> Timestamp {
    let now = Local::now().fixed_offset();
    now.with_nanosecond(0).unwrap_or(now)
}

/// A captured idea: one line, kept until it becomes an essay.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spark {
    pub id: String,
    pub text: String,
    pub created_at: Timestamp,
}

impl Spark {
    /// Captures `text` now, trimmed. Blank input is not a spark and is refused
    /// earlier, by `SparkStore::capture`.
    pub fn new(text: impl AsRef<str>) -> Self {
        Spark {
            id: Uuid::new_v4().to_string(),
            text: text.as_ref().trim().to_string(),
            created_at: now(),
        }
    }
}

/// One writing session. The format exists now; nothing records sessions until
/// the editor does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub essay_slug: String,
    pub started_at: Timestamp,
    pub duration_min: u32,
}

/// Where an essay is in the pipeline. Draft and Editing are the two halves of
/// being in progress; Published and Shelved are both endings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EssayStatus {
    Draft,
    Editing,
    Published,
    Shelved,
}

impl EssayStatus {
    /// Whether the essay occupies the single work-in-progress slot.
    pub fn is_in_progress(self) -> bool {
        matches!(self, EssayStatus::Draft | EssayStatus::Editing)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            EssayStatus::Draft => "draft",
            EssayStatus::Editing => "editing",
            EssayStatus::Published => "published",
            EssayStatus::Shelved => "shelved",
        }
    }

    /// Parses a status from front matter. Case-insensitive: the files are
    /// hand-editable.
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "draft" => Some(EssayStatus::Draft),
            "editing" => Some(EssayStatus::Editing),
            "published" => Some(EssayStatus::Published),
            "shelved" => Some(EssayStatus::Shelved),
            _ => None,
        }
    }

    /// Writing and editing alternate freely; either can end the essay. Nothing
    /// leaves Published or Shelved — an essay ends once.
    pub fn can_transition_to(self, to: EssayStatus) -> bool {
        use EssayStatus::*;
        matches!(
            (self, to),
            (Draft, Editing) | (Editing, Draft) | (Draft | Editing, Published | Shelved)
        )
    }
}

impl fmt::Display for EssayStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An essay: front matter plus a markdown-lite body, stored as one file.
#[derive(Debug, Clone)]
pub struct Essay {
    /// File name stem under `essays/`, and the essay's identity.
    pub slug: String,
    status: EssayStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub published_at: Option<Timestamp>,
    pub publication_url: Option<String>,
    /// The spark this essay grew from.
    pub spark: Option<String>,
    pub body: String,
}

impl Essay {
    /// A new essay, in Draft. Only `EssayStore::create` calls this, so the
    /// WIP = 1 check cannot be walked around.
    pub(crate) fn draft(slug: String, spark: Option<String>) -> Self {
        let created_at = now();
        Essay {
            slug,
            status: EssayStatus::Draft,
            created_at,
            updated_at: created_at,
            published_at: None,
            publication_url: None,
            spark,
            body: String::new(),
        }
    }

    /// Rebuilds an essay read from disk. The optional fields are filled in by
    /// the caller.
    pub(crate) fn from_parts(
        slug: String,
        status: EssayStatus,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Essay {
            slug,
            status,
            created_at,
            updated_at,
            published_at: None,
            publication_url: None,
            spark: None,
            body: String::new(),
        }
    }

    pub fn status(&self) -> EssayStatus {
        self.status
    }

    /// The essay as a blog receives it: the markdown body alone, with none of
    /// esse's front matter — what "Copy as markdown" and "Export to file…"
    /// hand out (essay-completion spec).
    ///
    /// The stored body already ends where the front matter does; what this adds
    /// is the tidying text handed to somebody else wants — no blank lines left
    /// over from the fence above it, and one newline at the end.
    pub fn body_markdown(&self) -> String {
        let body = self.body.trim();
        if body.is_empty() {
            String::new()
        } else {
            format!("{body}\n")
        }
    }

    pub fn is_in_progress(&self) -> bool {
        self.status.is_in_progress()
    }

    /// The only way to change status: illegal moves are refused rather than
    /// silently applied, and the essay's `updated_at` follows the change.
    pub fn transition_to(&mut self, to: EssayStatus) -> Result<()> {
        if !self.status.can_transition_to(to) {
            return Err(Error::IllegalTransition {
                from: self.status,
                to,
            });
        }
        self.status = to;
        self.updated_at = now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    const ALL: [EssayStatus; 4] = [
        EssayStatus::Draft,
        EssayStatus::Editing,
        EssayStatus::Published,
        EssayStatus::Shelved,
    ];

    /// The transitions the spec permits, written out rather than derived, so
    /// the test disagrees with the implementation when either drifts.
    const LEGAL: [(EssayStatus, EssayStatus); 6] = [
        (EssayStatus::Draft, EssayStatus::Editing),
        (EssayStatus::Editing, EssayStatus::Draft),
        (EssayStatus::Draft, EssayStatus::Published),
        (EssayStatus::Editing, EssayStatus::Published),
        (EssayStatus::Draft, EssayStatus::Shelved),
        (EssayStatus::Editing, EssayStatus::Shelved),
    ];

    fn essay_in(status: EssayStatus) -> Essay {
        let yesterday = now() - Duration::days(1);
        Essay::from_parts("an-essay".to_string(), status, yesterday, yesterday)
    }

    #[test]
    fn new_essay_starts_as_draft() {
        let essay = Essay::draft("first".to_string(), Some("почему эссе".to_string()));

        assert_eq!(essay.status(), EssayStatus::Draft);
        assert!(essay.is_in_progress());
        assert_eq!(essay.spark.as_deref(), Some("почему эссе"));
        assert_eq!(essay.created_at, essay.updated_at);
        assert!(essay.body.is_empty());
        assert!(essay.published_at.is_none());
    }

    #[test]
    fn draft_and_editing_are_the_in_progress_states() {
        assert!(EssayStatus::Draft.is_in_progress());
        assert!(EssayStatus::Editing.is_in_progress());
        assert!(!EssayStatus::Published.is_in_progress());
        assert!(!EssayStatus::Shelved.is_in_progress());
    }

    #[test]
    fn status_round_trips_through_its_text_form() {
        for status in ALL {
            assert_eq!(EssayStatus::parse(status.as_str()), Some(status));
            assert_eq!(status.to_string(), status.as_str());
        }
        // Front matter is hand-editable, so parsing forgives case and spacing.
        assert_eq!(EssayStatus::parse(" DRAFT "), Some(EssayStatus::Draft));
        assert_eq!(EssayStatus::parse("finished"), None);
    }

    #[test]
    fn spark_is_trimmed_timestamped_and_uniquely_identified() {
        let before = now();
        let spark = Spark::new("  поймать мысль  ");

        assert_eq!(spark.text, "поймать мысль");
        assert!(spark.created_at >= before);
        assert_ne!(Spark::new("одна и та же").id, Spark::new("одна и та же").id);
    }

    #[test]
    fn every_status_pair_is_permitted_or_rejected_as_specified() {
        for from in ALL {
            for to in ALL {
                let legal = LEGAL.contains(&(from, to));
                assert_eq!(from.can_transition_to(to), legal, "{from} -> {to}");

                let mut essay = essay_in(from);
                assert_eq!(essay.transition_to(to).is_ok(), legal, "{from} -> {to}");
                // A rejected transition changes nothing.
                assert_eq!(essay.status(), if legal { to } else { from });
            }
        }
    }

    #[test]
    fn the_markdown_handed_out_is_the_body_and_nothing_around_it() {
        let mut essay = essay_in(EssayStatus::Editing);
        essay.body = "\n\n# Заголовок\n\nПервый абзац.\n\n\n".to_string();

        assert_eq!(essay.body_markdown(), "# Заголовок\n\nПервый абзац.\n");

        essay.body = "  \n\n ".to_string();
        assert!(essay.body_markdown().is_empty());
    }

    #[test]
    fn a_transition_moves_updated_at_forward() {
        let mut essay = essay_in(EssayStatus::Draft);
        essay.transition_to(EssayStatus::Editing).unwrap();

        assert_eq!(essay.status(), EssayStatus::Editing);
        assert!(essay.updated_at > essay.created_at);
    }

    #[test]
    fn a_rejected_transition_reports_both_ends() {
        let mut essay = essay_in(EssayStatus::Published);
        let error = essay.transition_to(EssayStatus::Draft).unwrap_err();

        assert!(matches!(
            error,
            Error::IllegalTransition {
                from: EssayStatus::Published,
                to: EssayStatus::Draft
            }
        ));
        assert_eq!(essay.updated_at, essay.created_at);
    }
}
