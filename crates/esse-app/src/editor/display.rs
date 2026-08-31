//! Bridging source text and what is on screen.
//!
//! On a styled line the markers are not drawn, so the string handed to the text
//! shaper is shorter than the source line. Every offset that crosses that gap —
//! the caret, the selection edges, a mouse click — has to be translated. This
//! module owns that translation, framework-free so it can be tested directly.
//!
//! Offsets here are relative to the start of the source line, in bytes. Word
//! wrap is a second translation on top of this one and lives in `wrap`.

use std::ops::Range;

use markdown_lite::{RenderPlan, Style};

/// One drawn run: where it came from, where it landed, how it looks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub source: Range<usize>,
    pub display: Range<usize>,
    pub style: Style,
}

/// A source line as it appears on screen, plus the map back to the source.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DisplayLine {
    /// The text actually handed to the shaper.
    pub text: String,
    pub segments: Vec<Segment>,
    /// Heading level to size the line at, if any.
    pub heading: Option<u8>,
    source_len: usize,
}

impl DisplayLine {
    /// Build the display form of `line` from its render plan.
    pub fn new(line: &str, plan: &RenderPlan) -> Self {
        let mut text = String::with_capacity(line.len());
        let mut segments = Vec::with_capacity(plan.spans.len());

        for span in &plan.spans {
            let start = text.len();
            text.push_str(&line[span.range.clone()]);
            segments.push(Segment {
                source: span.range.clone(),
                display: start..text.len(),
                style: span.style,
            });
        }

        DisplayLine {
            text,
            segments,
            heading: plan.heading,
            source_len: line.len(),
        }
    }

    /// Where a source offset lands on screen. An offset inside a hidden marker
    /// collapses onto the near edge of the marker, so a selection that covers
    /// `**bold**` highlights exactly `bold`.
    pub fn display_offset(&self, source: usize) -> usize {
        let mut last_end = 0;
        for segment in &self.segments {
            if source < segment.source.start {
                // Inside the hidden marker before this segment.
                return last_end;
            }
            if source <= segment.source.end {
                return segment.display.start + (source - segment.source.start);
            }
            last_end = segment.display.end;
        }
        last_end
    }

    /// Where a screen offset came from. Used for click-to-position.
    ///
    /// A screen position at a hidden marker is ambiguous — the offsets on both
    /// sides of the marker draw at the same x. It resolves to the earlier one,
    /// which puts the caret *outside* the emphasis: clicking at the left edge
    /// of bold text and typing produces unemphasised text, as expected.
    pub fn source_offset(&self, display: usize) -> usize {
        let Some(last) = self.segments.last() else {
            // Nothing visible on this line at all (`# ` on its own, say) — put
            // the caret at the end, where typing continues.
            return self.source_len;
        };
        for segment in &self.segments {
            if display <= segment.display.end {
                let clamped = display.max(segment.display.start);
                return segment.source.start + (clamped - segment.display.start);
            }
        }
        last.source.end
    }
}

#[cfg(test)]
mod tests {
    use markdown_lite::render_plan;

    use super::*;

    fn styled(line: &str) -> DisplayLine {
        DisplayLine::new(line, &render_plan(line, false))
    }

    fn raw(line: &str) -> DisplayLine {
        DisplayLine::new(line, &render_plan(line, true))
    }

    #[test]
    fn styled_line_drops_the_markers() {
        let line = "a **bold** word";
        let display = styled(line);
        assert_eq!(display.text, "a bold word");
        assert_eq!(display.heading, None);
    }

    #[test]
    fn raw_line_is_the_source_line() {
        let line = "a **bold** word";
        let display = raw(line);
        assert_eq!(display.text, line);
        // On the cursor line the map is the identity — which is why caret
        // positioning while typing never needs it.
        for offset in 0..=line.len() {
            assert_eq!(display.display_offset(offset), offset);
            assert_eq!(display.source_offset(offset), offset);
        }
    }

    #[test]
    fn offsets_translate_across_hidden_markers() {
        //             0123456789...
        let line = "a **bold** word";
        let display = styled(line);

        assert_eq!(display.display_offset(0), 0, "start of line");
        assert_eq!(display.display_offset(2), 2, "start of the opening marker");
        assert_eq!(display.display_offset(4), 2, "first letter of 'bold'");
        assert_eq!(display.display_offset(8), 6, "end of 'bold'");
        assert_eq!(display.display_offset(10), 6, "end of the closing marker");
        assert_eq!(display.display_offset(line.len()), display.text.len());
    }

    #[test]
    fn offsets_inside_a_hidden_marker_collapse_to_its_edge() {
        let line = "**bold**";
        let display = styled(line);
        assert_eq!(display.text, "bold");

        // Anywhere inside the opening `**` maps to the start of "bold".
        assert_eq!(display.display_offset(0), 0);
        assert_eq!(display.display_offset(1), 0);
        // Anywhere inside the closing `**` maps to the end.
        assert_eq!(display.display_offset(7), 4);
        assert_eq!(display.display_offset(8), 4);
    }

    #[test]
    fn clicks_map_back_into_the_source() {
        let line = "a **bold** word";
        let display = styled(line);

        assert_eq!(display.source_offset(0), 0);
        assert_eq!(display.source_offset(3), 5, "clicking inside 'bold'");
        assert_eq!(display.source_offset(display.text.len()), line.len());

        // At a marker the two sides draw at the same x; the caret takes the
        // earlier one, landing outside the emphasis.
        assert_eq!(
            display.source_offset(2),
            2,
            "left edge of 'bold' is before `**`"
        );
        assert_eq!(
            display.source_offset(6),
            8,
            "right edge is before the closing `**`"
        );
    }

    #[test]
    fn heading_marker_is_hidden_but_the_level_survives() {
        let line = "## Заголовок";
        let display = styled(line);

        assert_eq!(display.text, "Заголовок");
        assert_eq!(display.heading, Some(2));
        assert_eq!(display.display_offset(3), 0, "first letter");
        assert_eq!(display.source_offset(0), 3, "clicking the first letter");
    }

    #[test]
    fn cyrillic_offsets_survive_the_round_trip() {
        let line = "Текст с **жирным** словом";
        let display = styled(line);
        assert_eq!(display.text, "Текст с жирным словом");

        // Strictly inside a segment the map is a bijection. (At a segment edge
        // it cannot be: two source offsets share one screen position.)
        for segment in &display.segments {
            for offset in segment.source.start + 1..segment.source.end {
                if !line.is_char_boundary(offset) {
                    continue;
                }
                assert_eq!(
                    display.source_offset(display.display_offset(offset)),
                    offset,
                    "offset {offset} did not survive the round trip"
                );
            }
        }

        // Every mapped offset must stay on a character boundary, or shaping
        // the line would panic on a Cyrillic letter.
        for offset in 0..=line.len() {
            if line.is_char_boundary(offset) {
                let display_offset = display.display_offset(offset);
                assert!(display.text.is_char_boundary(display_offset));
                assert!(line.is_char_boundary(display.source_offset(display_offset)));
            }
        }
    }

    #[test]
    fn a_line_with_nothing_visible_puts_the_caret_at_its_end() {
        let line = "# ";
        let display = styled(line);
        assert!(display.text.is_empty());
        assert_eq!(display.source_offset(0), line.len());
    }

    #[test]
    fn segments_carry_the_emphasis_they_are_drawn_with() {
        let line = "a **b** *i*";
        let display = styled(line);
        assert_eq!(display.text, "a b i");

        let styles: Vec<_> = display.segments.iter().map(|s| s.style).collect();
        assert!(!styles[0].bold, "the plain run");
        assert!(styles[1].bold);
        assert!(styles[3].italic);
    }
}
