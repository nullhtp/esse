//! Parser tests. Every case checks byte ranges, not just shape: the editor
//! maps cursor offsets through these, so an off-by-one here is a bug on screen.

use markdown_lite::{parse_line, SpanKind, Style};

const PLAIN: Style = Style::PLAIN;
const BOLD: Style = Style { bold: true, italic: false };
const ITALIC: Style = Style { bold: false, italic: true };
const BOTH: Style = Style { bold: true, italic: true };

/// Compact rendering of a parse: `("text", Some(style))` or `("**", None)`
/// for a marker, so a whole expectation fits on one line.
fn spans(line: &str) -> Vec<(&str, Option<Style>)> {
    parse_line(line)
        .spans
        .into_iter()
        .map(|s| {
            let text = &line[s.range];
            match s.kind {
                SpanKind::Text(style) => (text, Some(style)),
                SpanKind::Marker => (text, None),
            }
        })
        .collect()
}

/// The spans must tile the line exactly — no gaps, no overlaps, no lost bytes.
fn assert_tiles(line: &str) {
    let parsed = parse_line(line);
    let mut at = 0;
    for span in &parsed.spans {
        assert_eq!(span.range.start, at, "gap or overlap in {line:?}: {:?}", parsed.spans);
        at = span.range.end;
    }
    assert_eq!(at, line.len(), "spans stop short of the line end in {line:?}");
}

#[test]
fn plain_line_is_one_text_span() {
    assert_eq!(spans("just some prose"), [("just some prose", Some(PLAIN))]);
    assert_tiles("just some prose");
}

#[test]
fn empty_line_has_no_spans() {
    assert!(parse_line("").spans.is_empty());
}

#[test]
fn headings_carry_level_and_marker() {
    for level in 1..=6usize {
        let line = format!("{} Title", "#".repeat(level));
        let parsed = parse_line(&line);
        assert_eq!(parsed.heading_level(), Some(level as u8), "level {level}");
        assert_eq!(parsed.heading.unwrap().marker, 0..level + 1);
        assert_eq!(spans(&line).last().copied(), Some(("Title", Some(PLAIN))));
        assert_tiles(&line);
    }
}

#[test]
fn heading_needs_a_space_and_at_most_six_hashes() {
    for line in ["#no-space", "####### seven", " # indented", "text # mid-line", "#"] {
        assert_eq!(parse_line(line).heading_level(), None, "{line:?} is not a heading");
        assert_tiles(line);
    }
}

#[test]
fn bold_and_italic_split_into_markers_and_text() {
    assert_eq!(
        spans("**bold**"),
        [("**", None), ("bold", Some(BOLD)), ("**", None)]
    );
    assert_eq!(
        spans("*italic*"),
        [("*", None), ("italic", Some(ITALIC)), ("*", None)]
    );
    assert_eq!(
        spans("a *b* c **d** e"),
        [
            ("a ", Some(PLAIN)),
            ("*", None),
            ("b", Some(ITALIC)),
            ("*", None),
            (" c ", Some(PLAIN)),
            ("**", None),
            ("d", Some(BOLD)),
            ("**", None),
            (" e", Some(PLAIN)),
        ]
    );
    assert_tiles("a *b* c **d** e");
}

#[test]
fn emphasis_nests_one_level_each_way() {
    assert_eq!(
        spans("**bold *both* bold**"),
        [
            ("**", None),
            ("bold ", Some(BOLD)),
            ("*", None),
            ("both", Some(BOTH)),
            ("*", None),
            (" bold", Some(BOLD)),
            ("**", None),
        ]
    );
    assert_eq!(
        spans("*it **both** it*"),
        [
            ("*", None),
            ("it ", Some(ITALIC)),
            ("**", None),
            ("both", Some(BOTH)),
            ("**", None),
            (" it", Some(ITALIC)),
            ("*", None),
        ]
    );
}

#[test]
fn unclosed_markers_stay_literal() {
    for line in ["**unclosed", "*unclosed", "trailing*", "a ** b", "one * two"] {
        let parsed = spans(line);
        assert!(
            parsed.iter().all(|(_, style)| *style == Some(PLAIN)),
            "{line:?} should be plain text, got {parsed:?}"
        );
        assert_tiles(line);
    }
}

#[test]
fn empty_emphasis_is_literal() {
    // Nothing between the markers, so nothing to emphasise.
    for line in ["**", "****", "* *"] {
        let parsed = spans(line);
        assert!(
            parsed.iter().all(|(_, style)| matches!(style, Some(PLAIN) | None)),
            "{line:?} should not produce emphasis, got {parsed:?}"
        );
        assert_tiles(line);
    }
    assert_eq!(spans("****"), [("****", Some(PLAIN))]);
}

#[test]
fn runs_of_three_or_more_asterisks_are_literal() {
    assert_eq!(spans("***stars***"), [("***stars***", Some(PLAIN))]);
    assert_eq!(spans("a **** b"), [("a **** b", Some(PLAIN))]);
}

#[test]
fn other_markdown_stays_plain() {
    for line in [
        "- a list item",
        "* a bullet with an asterisk list marker",
        "* one * two",
        "1. numbered",
        "[link](https://example.com)",
        "`code`",
        "> quote",
        "| table | row |",
        "~~strike~~",
        "![img](a.png)",
        "---",
    ] {
        assert_eq!(spans(line), [(line, Some(PLAIN))], "{line:?} must stay plain");
        assert_eq!(parse_line(line).heading_level(), None);
    }
}

#[test]
fn cyrillic_byte_offsets_are_correct() {
    let line = "## Заголовок с **жирным** текстом";
    let parsed = parse_line(line);

    assert_eq!(parsed.heading_level(), Some(2));
    assert_eq!(
        spans(line),
        [
            ("## ", None),
            ("Заголовок с ", Some(PLAIN)),
            ("**", None),
            ("жирным", Some(BOLD)),
            ("**", None),
            (" текстом", Some(PLAIN)),
        ]
    );
    assert_tiles(line);

    // Every boundary must land on a char boundary, or slicing would panic.
    for span in &parsed.spans {
        assert!(line.is_char_boundary(span.range.start));
        assert!(line.is_char_boundary(span.range.end));
    }

    // Two bytes per Cyrillic letter: "## " is 3 bytes, then "Заголовок с "
    // is 9*2 + 1 + 2 + 1 = 22.
    assert_eq!(parsed.spans[1].range, 3..25);
}

#[test]
fn emphasis_around_non_ascii_and_emoji() {
    assert_eq!(
        spans("*кириллица* и *ещё*"),
        [
            ("*", None),
            ("кириллица", Some(ITALIC)),
            ("*", None),
            (" и ", Some(PLAIN)),
            ("*", None),
            ("ещё", Some(ITALIC)),
            ("*", None),
        ]
    );
    assert_tiles("**жирный 🎉 текст**");
    assert_eq!(
        spans("**жирный 🎉 текст**"),
        [("**", None), ("жирный 🎉 текст", Some(BOLD)), ("**", None)]
    );
}
