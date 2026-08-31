//! File-backed storage. The files are the single source of truth: nothing is
//! cached, every read scans the disk, so hand-edited or synced files need no
//! import step (design.md, D2).

mod data_dir;
mod essays;
mod frontmatter;
mod sessions;
mod sparks;

pub use data_dir::DataDir;
pub use essays::EssayStore;
pub use sessions::SessionStore;
pub use sparks::SparkStore;

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};

/// Reads a file that is allowed not to exist yet.
pub(crate) fn read_to_string(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::io(path)(error)),
    }
}

/// Replaces a file by writing a sibling temp file and renaming over the
/// target, so a crash can never leave half a file behind (design.md, D5).
pub(crate) fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir).map_err(Error::io(dir))?;

    // Same directory as the target: a rename within one filesystem is atomic,
    // across filesystems it would be a copy.
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temp = dir.join(format!(".{name}.{}.tmp", unique()));

    let write = || -> std::io::Result<()> {
        let mut file = File::create(&temp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
    };
    if let Err(error) = write() {
        let _ = fs::remove_file(&temp);
        return Err(Error::io(&temp)(error));
    }

    fs::rename(&temp, path).map_err(Error::io(path))
}

/// Appends one line, flushed before returning.
pub(crate) fn append_line(path: &Path, line: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(Error::io(dir))?;
    }
    let append = || -> std::io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_data()
    };
    append().map_err(Error::io(path))
}

/// Reads a JSONL file in file order. A missing file is an empty file.
pub(crate) fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let Some(text) = read_to_string(path)? else {
        return Ok(Vec::new());
    };
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line).map_err(|source| Error::Record {
                path: path.to_path_buf(),
                line: index + 1,
                source,
            })
        })
        .collect()
}

fn unique() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}
