//! Render-plan tests: the contract both editor prototypes are built against.
//!
//! Each case maps back to a scenario in
//! `openspec/changes/editor-framework-prototype/specs/live-markdown-editing/spec.md`.

use markdown_lite::{render_plan, Style};

const PLAIN: Style = Style::PLAIN;
const BOLD: Style = Style { bold: true, italic: false };
const ITALIC: Style = Style { bold: false, italic: true };

/// `(visible text, style)` per span.
fn drawn(line: &str, cursor_on_line: bool) -> Vec<(&str, Style)> {
    render_plan(line, cursor_on_line)
        .spans
        .into_iter()
        .map(|s| (&line[s.range], s.style))
        .collect()
}

// Scenario: Bold renders as you type.
#[test]
fn bold_markers_hidden_off_the_cursor_line() {
    let line = "a **bold** word";
    assert_eq!(
        drawn(line, false),
        [("a ", PLAIN), ("bold", BOLD), (" word", PLAIN)]
    );
    assert_eq!(render_plan(line, false).visible_text(line), "a bold word");
}

// Scenario: Italic renders as you type.
#[test]
fn italic_markers_hidden_off_the_cursor_line() {
    let line = "a *slanted* word";
    assert_eq!(
        drawn(line, false),
        [("a ", PLAIN), ("slanted", ITALIC), (" word", PLAIN)]
    );
    assert_eq!(render_plan(line, false).visible_text(line), "a slanted word");
}

// Scenario: Heading renders as you type.
#[test]
fn heading_marker_hidden_and_level_reported() {
    let plan = render_plan("## Второй уровень", false);
    assert_eq!(plan.heading, Some(2));
    assert_eq!(plan.visible_text("## Второй уровень"), "Второй уровень");
}

// Scenario: Entering a styled line reveals markers.
#[test]
fn cursor_line_renders_raw() {
    let line = "# Title with **bold** and *italic*";
    let plan = render_plan(line, true);

    assert!(plan.raw);
    assert_eq!(plan.heading, None, "a raw line is plain text, not a sized heading");
    assert_eq!(drawn(line, true), [(line, PLAIN)]);
    assert_eq!(plan.visible_text(line), line, "every character stays on screen");
}

// Scenario: Leaving a line hides markers again.
#[test]
fn same_line_styles_once_the_cursor_leaves() {
    let line = "# Title with **bold** and *italic*";
    let plan = render_plan(line, false);

    assert!(!plan.raw);
    assert_eq!(plan.heading, Some(1));
    assert_eq!(plan.visible_text(line), "Title with bold and italic");
}

// Scenario: Other markdown syntax stays plain.
#[test]
fn other_markdown_draws_literally() {
    for line in ["- list", "[link](url)", "`code`", "> quote"] {
        let plan = render_plan(line, false);
        assert_eq!(plan.heading, None);
        assert_eq!(drawn(line, false), [(line, PLAIN)], "{line:?}");
        assert_eq!(plan.visible_text(line), line);
    }
}

#[test]
fn adjacent_runs_of_equal_style_are_merged() {
    // Nothing between "a" and " word" but a hidden marker pair, yet they are
    // separate source ranges — they must not merge across the gap.
    let plan = render_plan("a **b** c", false);
    assert_eq!(plan.spans.len(), 3);

    // A plain line is a single span, however long.
    assert_eq!(render_plan("no markup at all here", false).spans.len(), 1);
}

#[test]
fn empty_line_draws_nothing_either_way() {
    assert!(render_plan("", true).spans.is_empty());
    assert!(render_plan("", false).spans.is_empty());
}

#[test]
fn ranges_stay_on_char_boundaries_for_cyrillic() {
    let line = "## Заголовок и **жирный** текст";
    for cursor_on_line in [true, false] {
        for span in render_plan(line, cursor_on_line).spans {
            assert!(line.is_char_boundary(span.range.start));
            assert!(line.is_char_boundary(span.range.end));
        }
    }
    assert_eq!(
        render_plan(line, false).visible_text(line),
        "Заголовок и жирный текст"
    );
}

#[test]
fn unclosed_markers_draw_as_typed() {
    // Mid-typing state: the writer has opened bold but not closed it yet.
    let line = "half a **thought";
    assert_eq!(drawn(line, false), [(line, PLAIN)]);
    assert_eq!(render_plan(line, false).visible_text(line), line);
}
