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

    /// The one-time move into the visible folder went wrong. Both paths are in
    /// the message: the old directory is still standing, and a person with a
    /// terminal can finish the move by hand.
    #[error("could not move the data from {from} to {to}: {source}")]
    Migration {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },

    /// A folder named at setup that is not there to be opened — the volume it
    /// lives on is not mounted, most likely. Creating it would hand the writer
    /// a convincing empty lookalike while their essays sat on an unplugged
    /// disk (guided-install design.md, D9).
    #[error(
        "{path} is not reachable: the folder above it does not exist. \
         If it is on another volume, mount it and open esse again; \
         to choose a different folder, run esse-setup"
    )]
    MissingLocation { path: PathBuf },

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

    /// The WIP invariant, refused at the storage layer. It names the limit
    /// rather than one of the essays: any of them could be the one to finish,
    /// so naming one would be misdirection (design.md, D6).
    #[error("{limit} essays are already in progress; publish or shelve one before starting another")]
    TooManyInProgress { limit: usize },

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

    pub(crate) fn migration(
        from: impl Into<PathBuf>,
        to: impl Into<PathBuf>,
    ) -> impl FnOnce(io::Error) -> Error {
        move |source| Error::Migration {
            from: from.into(),
            to: to.into(),
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
