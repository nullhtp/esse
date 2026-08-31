//! Everything that can go wrong between the domain model and the disk.

use std::io;
use std::path::PathBuf;

use crate::model::EssayStatus;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no platform data directory is available; set ESSE_DATA_DIR to choose one")]
    NoDataDir,

    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    /// A JSONL line that does not parse. Reported with its line number so the
    /// file can be repaired by hand — that is the point of a text format.
    #[error("{path}: line {line} is not a valid record: {source}")]
    Record {
        path: PathBuf,
        line: usize,
        #[source]
        source: serde_json::Error,
    },

    #[error("{path}: {message}")]
    Format { path: PathBuf, message: String },

    #[error("could not encode a record: {0}")]
    Encode(#[from] serde_json::Error),

    #[error("essay '{slug}' does not exist")]
    EssayNotFound { slug: String },

    /// The spark was gone by the time the essay was started — two windows, or
    /// a hand-edited `sparks.jsonl`.
    #[error("that spark is no longer in the box")]
    SparkNotFound { id: String },

    #[error("essay '{slug}' already exists")]
    SlugTaken { slug: String },

    #[error("'{slug}' is not a usable essay name")]
    InvalidSlug { slug: String },

    /// The WIP = 1 invariant, refused at the storage layer.
    #[error("'{slug}' is still in progress ({status}); publish or shelve it before starting another")]
    EssayInProgress { slug: String, status: EssayStatus },

    #[error("an essay cannot move from {from} to {to}")]
    IllegalTransition { from: EssayStatus, to: EssayStatus },
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>) -> impl FnOnce(io::Error) -> Error {
        move |source| Error::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn format(path: impl Into<PathBuf>, message: impl Into<String>) -> Error {
        Error::Format {
            path: path.into(),
            message: message.into(),
        }
    }
}
