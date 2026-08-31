//! The render plan: the single entry point both editor prototypes call.
//!
//! Rendering is a pure function of `(line text, cursor on this line?)` — see
//! design decision D4, line-scoped marker reveal. The cursor line is drawn raw
//! so the source stays editable as plain characters; every other line is drawn
//! styled with its markers hidden.

use crate::{parse_line, Span, SpanKind, Style};
use std::ops::Range;

/// A run of text to draw, with the style to draw it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSpan {
    /// Byte range into the line the plan was built from.
    pub range: Range<usize>,
    pub style: Style,
}

/// What to draw for one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPlan {
    /// Heading level to size the line at, or `None` for body text. Always
    /// `None` while the cursor is on the line: the raw line is plain text.
    pub heading: Option<u8>,
    /// Spans to draw, in source order. Skipped markers leave gaps in the
    /// ranges; concatenating the slices gives the visible text.
    pub spans: Vec<RenderSpan>,
    /// True when the line is shown as-is, markers and all.
    pub raw: bool,
}

impl RenderPlan {
    /// The visible text, reassembled. Handy for tests and for measuring; a
    /// renderer normally slices the source per span instead.
    pub fn visible_text(&self, line: &str) -> String {
        self.spans.iter().map(|s| &line[s.range.clone()]).collect()
    }
}

/// Build the render plan for one line.
///
/// With the cursor on the line the plan is the raw line, unstyled. Otherwise
/// markers are dropped and the remaining text carries its emphasis, with
/// adjacent runs of equal style merged.
pub fn render_plan(line: &str, cursor_on_line: bool) -> RenderPlan {
    if cursor_on_line {
        let mut spans = Vec::new();
        if !line.is_empty() {
            spans.push(RenderSpan { range: 0..line.len(), style: Style::PLAIN });
        }
        return RenderPlan { heading: None, spans, raw: true };
    }

    let parsed = parse_line(line);
    let heading = parsed.heading_level();
    let mut spans: Vec<RenderSpan> = Vec::new();
    for Span { range, kind } in parsed.spans {
        let SpanKind::Text(style) = kind else { continue };
        match spans.last_mut() {
            Some(last) if last.style == style && last.range.end == range.start => {
                last.range.end = range.end;
            }
            _ => spans.push(RenderSpan { range, style }),
        }
    }

    RenderPlan { heading, spans, raw: false }
}
