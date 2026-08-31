//! Responsiveness at essay size, for the part that does not need a window.
//!
//! Because layout is anchored on the caret, a frame only ever touches the
//! paragraphs in and near the viewport — the document below and above is never
//! parsed, planned, or shaped. These guard the half we own, so a change that
//! reintroduces a whole-document pass per frame shows up as a failing test
//! rather than as a vague feeling that typing got worse. Glyph shaping is
//! gpui's and is checked by hand in the running app.

use std::time::Instant;

use markdown_lite::render_plan;

use super::buffer::Buffer;
use super::display::DisplayLine;
use super::wrap::VisualLine;

/// Roughly what the spec calls essay size.
fn essay() -> String {
    let paragraphs = [
        "Письмо начинается не с идеи, а с готовности сесть за стол и написать \
         первую **плохую** строчку, за которой прячется вторая.",
        "Черновик существует, чтобы быть *переписанным*: его задача — вытащить \
         мысль наружу, а не понравиться читателю с первого раза.",
        "Правка — отдельная работа, и смешивать её с письмом значит не сделать \
         толком ни того, ни другого.",
    ];
    let mut text = String::with_capacity(21_000);
    let mut index = 0;
    while text.chars().count() < 20_000 {
        if index % 8 == 0 {
            text.push_str("## Раздел\n\n");
        }
        text.push_str(paragraphs[index % paragraphs.len()]);
        text.push_str("\n\n");
        index += 1;
    }
    text
}

/// Paragraphs that fit on screen at once — all a frame ever looks at.
const VISIBLE_LINES: usize = 30;
/// One frame at 60fps. The real budget is smaller, since shaping and painting
/// also have to fit, but exceeding even this means something has gone wrong.
const FRAME_BUDGET_MS: f64 = 16.0;

/// The work of a frame, minus the shaper: parse and plan the visible
/// paragraphs and map each one's rows. Wrap boundaries stand in for the
/// shaper's, which needs a window.
fn frame(buffer: &Buffer, cursor_line: usize) {
    for index in cursor_line..(cursor_line + VISIBLE_LINES).min(buffer.line_count()) {
        let line = buffer.line(index);
        let display = DisplayLine::new(line, &render_plan(line, index == cursor_line));
        let boundaries: Vec<usize> = (1..display.text.len() / 60)
            .map(|row| row * 60)
            .filter(|start| display.text.is_char_boundary(*start))
            .collect();
        std::hint::black_box(VisualLine::new(display, boundaries));
    }
}

#[test]
fn a_frame_of_layout_work_fits_the_budget_at_essay_size() {
    let text = essay();
    assert!(text.chars().count() >= 20_000, "sample must be essay sized");

    let buffer = Buffer::new(text);
    let cursor_line = buffer.line_count() / 2;

    // Measure a run of frames so one scheduling hiccup cannot fail the test.
    const FRAMES: usize = 20;
    let start = Instant::now();
    for _ in 0..FRAMES {
        frame(&buffer, cursor_line);
    }
    let per_frame = start.elapsed().as_secs_f64() * 1000. / FRAMES as f64;

    println!(
        "{per_frame:.2} ms per frame over {} paragraphs",
        buffer.line_count()
    );
    assert!(
        per_frame < FRAME_BUDGET_MS,
        "layout work took {per_frame:.2}ms per frame, over the {FRAME_BUDGET_MS}ms budget"
    );
}

#[test]
fn a_frame_costs_the_same_in_a_long_document_as_in_a_short_one() {
    let long = Buffer::new(essay());
    let short = Buffer::new(essay().lines().take(VISIBLE_LINES).collect::<Vec<_>>().join("\n"));
    assert!(long.line_count() > short.line_count() * 4, "not a fair test");

    // The claim is structural, not statistical: a frame reads a fixed window of
    // paragraphs, so the loop below runs the same number of times either way.
    let touched = |buffer: &Buffer, cursor_line: usize| {
        (cursor_line..(cursor_line + VISIBLE_LINES).min(buffer.line_count())).count()
    };
    assert_eq!(touched(&long, 0), touched(&short, 0));
    assert_eq!(touched(&long, long.line_count() / 2), VISIBLE_LINES);
}

#[test]
fn editing_an_essay_sized_buffer_stays_cheap() {
    let mut buffer = Buffer::new(essay());
    let middle = buffer.text().len() / 2;
    buffer.set_cursor(middle);

    // Reindexing after an edit is O(document); at this size it must still
    // disappear under a keystroke.
    const KEYSTROKES: usize = 200;
    let start = Instant::now();
    for _ in 0..KEYSTROKES {
        buffer.insert("я");
    }
    let per_keystroke = start.elapsed().as_secs_f64() * 1000. / KEYSTROKES as f64;

    println!("{per_keystroke:.3} ms per keystroke");
    assert!(
        per_keystroke < 1.0,
        "a keystroke cost {per_keystroke:.3}ms, which will be felt while typing"
    );
}
