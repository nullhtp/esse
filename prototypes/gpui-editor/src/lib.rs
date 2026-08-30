//! gpui prototype of the live markdown-lite editor.
//!
//! Split lib/bin so the framework-free half — the text buffer — can be tested
//! without opening a window. Everything that touches gpui lives in the binary.

pub mod buffer;
pub mod display;
