//! The essay file format: TOML front matter between `+++` fences, then the
//! markdown-lite body.
//!
//! `+++` is the Hugo convention and says "TOML" unambiguously, where `---`
//! conventionally means YAML. The front matter is kept as a real TOML document
//! rather than a struct, and a key is rewritten only when its value actually
//! changed — so fields this version knows nothing about survive a read and
//! write back untouched, formatting and all.

use std::path::Path;

use chrono::{DateTime, SecondsFormat};
use toml_edit::{value, DocumentMut};

use crate::error::{Error, Result};
use crate::model::{Essay, EssayStatus, Timestamp};

const FENCE: &str = "+++";

/// Splits a file into its front matter and its body.
fn split(content: &str) -> Option<(&str, &str)> {
    let rest = content.strip_prefix(FENCE)?.strip_prefix('\n')?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == FENCE {
            return Some((&rest[..offset], &rest[offset + line.len()..]));
        }
        offset += line.len();
    }
    None
}

pub(crate) fn parse(path: &Path, slug: &str, content: &str) -> Result<Essay> {
    let (front, body) = split(content).ok_or_else(|| {
        Error::format(path, "expected TOML front matter between `+++` fences")
    })?;

    let doc: DocumentMut = front
        .parse()
        .map_err(|error| Error::format(path, format!("front matter is not valid TOML: {error}")))?;

    let status_text = required_str(path, &doc, "status")?;
    let status = EssayStatus::parse(status_text)
        .ok_or_else(|| Error::format(path, format!("unknown status `{status_text}`")))?;

    let mut essay = Essay::from_parts(
        slug.to_string(),
        status,
        time(path, &doc, "created_at")?.ok_or_else(|| missing(path, "created_at"))?,
        time(path, &doc, "updated_at")?.ok_or_else(|| missing(path, "updated_at"))?,
    );
    essay.published_at = time(path, &doc, "published_at")?;
    essay.publication_url = optional_str(&doc, "publication_url").map(str::to_string);
    essay.spark = optional_str(&doc, "spark").map(str::to_string);
    essay.body = body.to_string();
    Ok(essay)
}

/// Renders the essay back to file content. `previous` is the file as it is on
/// disk, if it exists; its front matter carries anything this version does not
/// model.
pub(crate) fn render(path: &Path, essay: &Essay, previous: Option<&str>) -> Result<String> {
    let mut doc = match previous.and_then(split) {
        Some((front, _)) => front.parse::<DocumentMut>().map_err(|error| {
            Error::format(path, format!("front matter is not valid TOML: {error}"))
        })?,
        None => DocumentMut::new(),
    };

    set(&mut doc, "status", Some(essay.status().as_str()));
    set(&mut doc, "created_at", Some(&format_time(essay.created_at)));
    set(&mut doc, "updated_at", Some(&format_time(essay.updated_at)));
    set(
        &mut doc,
        "published_at",
        essay.published_at.map(format_time).as_deref(),
    );
    set(&mut doc, "publication_url", essay.publication_url.as_deref());
    set(&mut doc, "spark", essay.spark.as_deref());

    Ok(format!("{FENCE}\n{doc}{FENCE}\n{}", essay.body))
}

/// Writes a key only when it differs from what the document already holds, so
/// an unchanged essay renders byte-for-byte as it was read.
fn set(doc: &mut DocumentMut, key: &str, new: Option<&str>) {
    match new {
        Some(new) if optional_str(doc, key) != Some(new) => doc[key] = value(new),
        Some(_) => {}
        None => {
            doc.remove(key);
        }
    }
}

fn optional_str<'a>(doc: &'a DocumentMut, key: &str) -> Option<&'a str> {
    doc.get(key).and_then(|item| item.as_str())
}

fn required_str<'a>(path: &Path, doc: &'a DocumentMut, key: &str) -> Result<&'a str> {
    optional_str(doc, key).ok_or_else(|| missing(path, key))
}

fn time(path: &Path, doc: &DocumentMut, key: &str) -> Result<Option<Timestamp>> {
    let Some(text) = optional_str(doc, key) else {
        return Ok(None);
    };
    DateTime::parse_from_rfc3339(text)
        .map(Some)
        .map_err(|error| Error::format(path, format!("`{key}` is not an RFC 3339 time: {error}")))
}

fn missing(path: &Path, key: &str) -> Error {
    Error::format(path, format!("front matter has no `{key}`"))
}

fn format_time(time: Timestamp) -> String {
    time.to_rfc3339_opts(SecondsFormat::Secs, false)
}
