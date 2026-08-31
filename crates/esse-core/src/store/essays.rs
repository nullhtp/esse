//! Essays, one file each under `essays/`.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::model::Essay;
use crate::store::{frontmatter, read_to_string, write_atomic, DataDir};

pub struct EssayStore {
    dir: PathBuf,
}

impl EssayStore {
    pub fn new(data: &DataDir) -> Self {
        EssayStore {
            dir: data.essays_dir(),
        }
    }

    /// Starts a new essay, in Draft.
    ///
    /// This is the WIP = 1 invariant: while any essay is Draft or Editing,
    /// creation is refused and nothing is written. There is deliberately no
    /// other way to create an essay, so no caller — no UI — can get around it.
    pub fn create(&self, slug: &str, spark: Option<&str>) -> Result<Essay> {
        validate_slug(slug)?;
        if let Some(open) = self.in_progress()? {
            let status = open.status();
            return Err(Error::EssayInProgress {
                slug: open.slug,
                status,
            });
        }
        if self.path(slug).exists() {
            return Err(Error::SlugTaken {
                slug: slug.to_string(),
            });
        }

        let essay = Essay::draft(slug.to_string(), spark.map(str::to_string));
        self.save(&essay)?;
        Ok(essay)
    }

    pub fn load(&self, slug: &str) -> Result<Essay> {
        let path = self.path(slug);
        let content = read_to_string(&path)?.ok_or_else(|| Error::EssayNotFound {
            slug: slug.to_string(),
        })?;
        frontmatter::parse(&path, slug, &content)
    }

    /// Every essay, most recently changed first.
    pub fn load_all(&self) -> Result<Vec<Essay>> {
        let entries = match fs::read_dir(&self.dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(Error::io(&self.dir)(error)),
        };

        let mut essays = Vec::new();
        for entry in entries {
            let path = entry.map_err(Error::io(&self.dir))?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let Some(slug) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            essays.push(self.load(slug)?);
        }
        essays.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        Ok(essays)
    }

    /// The essay currently occupying the work-in-progress slot, if any.
    pub fn in_progress(&self) -> Result<Option<Essay>> {
        Ok(self
            .load_all()?
            .into_iter()
            .find(|essay| essay.is_in_progress()))
    }

    /// Writes the essay atomically, keeping any front-matter fields this
    /// version does not understand.
    pub fn save(&self, essay: &Essay) -> Result<()> {
        let path = self.path(&essay.slug);
        let previous = read_to_string(&path)?;
        let content = frontmatter::render(&path, essay, previous.as_deref())?;
        write_atomic(&path, &content)
    }

    pub fn path(&self, slug: &str) -> PathBuf {
        self.dir.join(format!("{slug}.md"))
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

/// A slug is a file name, so it stays to characters that mean the same thing
/// on every filesystem — and can never step out of the essays directory.
fn validate_slug(slug: &str) -> Result<()> {
    let usable = !slug.is_empty()
        && slug
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        && slug.chars().any(|ch| ch.is_ascii_alphanumeric());
    if usable {
        Ok(())
    } else {
        Err(Error::InvalidSlug {
            slug: slug.to_string(),
        })
    }
}
