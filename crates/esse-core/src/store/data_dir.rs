//! Where the files live.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// The directory holding every file esse owns. Resolving the path and creating
/// the directory happen once, here; the stores are handed paths.
#[derive(Debug, Clone)]
pub struct DataDir {
    root: PathBuf,
}

impl DataDir {
    /// Opens the platform data directory (`~/Library/Application Support/esse`
    /// on macOS), creating it on first launch.
    ///
    /// `ESSE_DATA_DIR` overrides the location. That is a development and test
    /// affordance, not a user setting.
    pub fn open() -> Result<Self> {
        let root = match env::var_os("ESSE_DATA_DIR") {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            _ => dirs::data_dir().ok_or(Error::NoDataDir)?.join("esse"),
        };
        DataDir::at(root)
    }

    /// Opens a specific directory, creating it if it does not exist.
    pub fn at(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(Error::io(&root))?;
        Ok(DataDir { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn essays_dir(&self) -> PathBuf {
        self.root.join("essays")
    }

    pub fn sparks_path(&self) -> PathBuf {
        self.root.join("sparks.jsonl")
    }

    pub fn sessions_path(&self) -> PathBuf {
        self.root.join("sessions.jsonl")
    }
}
