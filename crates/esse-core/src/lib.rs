//! Domain model and file-backed storage for esse.
//!
//! This crate knows nothing about the GUI: everything load-bearing — the essay
//! lifecycle, the WIP = 1 invariant, the on-disk formats — lives here so it can
//! be tested without building gpui.

pub mod error;
pub mod model;
pub mod store;

pub use error::{Error, Result};
pub use model::{Essay, EssayStatus, Session, Spark};
pub use store::{DataDir, EssayStore, SessionStore, SparkStore};
