//! Sessions, one JSON object per line in `sessions.jsonl`.
//!
//! The format and its round-trip exist now; nothing records a session until
//! the editor has a session timer.

use std::path::PathBuf;

use crate::error::Result;
use crate::model::Session;
use crate::store::{append_line, read_jsonl, DataDir};

pub struct SessionStore {
    path: PathBuf,
}

impl SessionStore {
    pub fn new(data: &DataDir) -> Self {
        SessionStore {
            path: data.sessions_path(),
        }
    }

    pub fn append(&self, session: &Session) -> Result<()> {
        append_line(&self.path, &serde_json::to_string(session)?)
    }

    /// Every session, oldest first — the order the dotted calendar reads in.
    pub fn load_all(&self) -> Result<Vec<Session>> {
        read_jsonl(&self.path)
    }
}
