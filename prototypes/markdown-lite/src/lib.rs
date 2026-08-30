//! Markdown-lite: the whole syntax is `#` headings, `*italic*` and `**bold**`.
//!
//! Line-based by design. Every function takes a single line and returns byte
//! ranges into that same line, so a renderer can slice the source directly and
//! map screen positions back to buffer offsets without a second index.
//!
//! Two layers:
//!
//! * [`parse_line`] — the full picture: text spans *and* marker spans, tiling
//!   the line.
//! * [`render_plan`] — what to actually draw, given whether the cursor sits on
//!   this line. This is the entry point the editor prototypes call.

use std::ops::Range;

mod parse;
mod render;

pub use parse::parse_line;
pub use render::{render_plan, RenderPlan, RenderSpan};

/// Emphasis carried by a run of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
}

impl Style {
    pub const PLAIN: Style = Style { bold: false, italic: false };
}

/// A heading marker: `#` repeated `level` times plus the space after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// 1-6.
    pub level: u8,
    /// Byte range of `#`s and the trailing space, always starting at 0.
    pub marker: Range<usize>,
}

/// One tile of a parsed line: either visible text or a syntax marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Byte range into the line that was parsed.
    pub range: Range<usize>,
    pub kind: SpanKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// Content, with the emphasis that applies to it.
    Text(Style),
    /// `#`, `*` or `**` — hidden when the line renders styled.
    Marker,
}

/// A line broken into heading + spans. The spans tile the line exactly: their
/// ranges are contiguous, in order, and cover every byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLine {
    pub heading: Option<Heading>,
    pub spans: Vec<Span>,
}

impl ParsedLine {
    /// Heading level, if this line is a heading.
    pub fn heading_level(&self) -> Option<u8> {
        self.heading.as_ref().map(|h| h.level)
    }
}
