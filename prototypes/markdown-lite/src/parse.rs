//! Line parser for markdown-lite.
//!
//! Rules, in full:
//!
//! * A heading is 1-6 `#` at the very start of the line followed by a space.
//!   The marker is `#`s plus that one space.
//! * `**bold**` and `*italic*` delimit emphasis. A delimiter is a run of
//!   exactly two (bold) or exactly one (italic) `*`; runs of three or more are
//!   literal text, and so is any opener without a matching closer on the line.
//! * A delimiter may not hug whitespace on the inside: an opener must be
//!   followed by a non-space and a closer preceded by one. This is what keeps
//!   `* bullet *` a literal bullet list rather than an emphasised space.
//! * Emphasis may nest one level the other way round (bold inside italic and
//!   vice versa). Everything else is literal text.

use crate::{Heading, ParsedLine, Span, SpanKind, Style};

/// Parse one line (without its trailing newline) into heading + spans.
///
/// The returned spans tile the whole line: concatenating their byte ranges in
/// order reproduces the input exactly, with each byte marked either as text
/// (carrying its style) or as a marker (hidden when the line renders styled).
pub fn parse_line(line: &str) -> ParsedLine {
    let (heading, body_start) = parse_heading(line);

    let mut spans = Vec::new();
    if let Some(h) = &heading {
        spans.push(Span { range: h.marker.clone(), kind: SpanKind::Marker });
    }
    parse_inline(line, body_start, line.len(), Style::default(), &mut spans);

    ParsedLine { heading, spans }
}

/// Returns the heading (if any) and the byte offset where the body text starts.
fn parse_heading(line: &str) -> (Option<Heading>, usize) {
    let bytes = line.as_bytes();
    let level = bytes.iter().take_while(|b| **b == b'#').count();
    if level == 0 || level > 6 || bytes.get(level) != Some(&b' ') {
        return (None, 0);
    }
    let marker_end = level + 1;
    (Some(Heading { level: level as u8, marker: 0..marker_end }), marker_end)
}

/// Parse `line[start..end]` for emphasis, appending spans in source order.
///
/// `outer` carries the styles already open around this slice, both to apply
/// them to the text found and to stop the same delimiter from nesting twice.
fn parse_inline(line: &str, start: usize, end: usize, outer: Style, spans: &mut Vec<Span>) {
    let bytes = line.as_bytes();
    let mut text_start = start;
    let mut i = start;

    while i < end {
        if bytes[i] != b'*' {
            i += 1;
            continue;
        }

        let run = run_len(bytes, i, end);
        let delim = match run {
            2 if !outer.bold => 2,
            1 if !outer.italic => 1,
            // Runs of 3+ are literal, and so is a delimiter whose emphasis is
            // already open around us.
            _ => {
                i += run;
                continue;
            }
        };

        let content_start = i + delim;
        if content_start >= end || is_space(bytes[content_start]) {
            i += run;
            continue;
        }
        let Some(close) = find_close(bytes, content_start, end, delim) else {
            i += run;
            continue;
        };

        push_text(line, text_start, i, outer, spans);
        spans.push(Span { range: i..content_start, kind: SpanKind::Marker });

        let mut inner = outer;
        if delim == 2 {
            inner.bold = true;
        } else {
            inner.italic = true;
        }
        parse_inline(line, content_start, close, inner, spans);

        spans.push(Span { range: close..close + delim, kind: SpanKind::Marker });
        i = close + delim;
        text_start = i;
    }

    push_text(line, text_start, end, outer, spans);
}

/// Length of the run of `*` starting at `i`, clamped to `end`.
fn run_len(bytes: &[u8], i: usize, end: usize) -> usize {
    let mut n = 0;
    while i + n < end && bytes[i + n] == b'*' {
        n += 1;
    }
    n
}

/// Find the closing delimiter for `delim` asterisks, searching in
/// `bytes[from..end]`. The content between opener and closer must be non-empty.
fn find_close(bytes: &[u8], from: usize, end: usize, delim: usize) -> Option<usize> {
    let mut i = from;
    while i < end {
        if bytes[i] != b'*' {
            i += 1;
            continue;
        }
        let run = run_len(bytes, i, end);
        // A run of the exact delimiter length closes, as long as it leaves
        // something other than whitespace between the markers.
        if run == delim && i > from && !is_space(bytes[i - 1]) {
            return Some(i);
        }
        i += run;
    }
    None
}

fn is_space(b: u8) -> bool {
    b == b' ' || b == b'\t'
}

fn push_text(line: &str, start: usize, end: usize, style: Style, spans: &mut Vec<Span>) {
    debug_assert!(line.is_char_boundary(start) && line.is_char_boundary(end));
    if start < end {
        spans.push(Span { range: start..end, kind: SpanKind::Text(style) });
    }
}
