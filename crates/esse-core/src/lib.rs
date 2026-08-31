//! Domain model and file-backed storage for esse.
//!
//! This crate knows nothing about the GUI: everything load-bearing — the essay
//! lifecycle, the WIP = 1 invariant, the on-disk formats — lives here so it can
//! be tested without building gpui.

pub mod complete;
pub mod error;
pub mod model;
pub mod setup;
pub mod slug;
pub mod start;
pub mod store;

pub use complete::{publish, shelve};
pub use error::{Error, Result};
pub use model::{Essay, EssayStatus, Session, Spark};
pub use setup::Setup;
pub use slug::derive_slug;
pub use start::start_essay_from_spark;
pub use store::{DataDir, EssayStore, SessionStore, SparkStore};
