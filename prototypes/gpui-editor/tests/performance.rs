//! Responsiveness at essay size, for the part that does not need a window.
//!
//! Every frame the editor parses *all* lines (to get their heights) and builds
//! render plans for the *visible* ones. Glyph shaping is gpui's and is measured
//! by hand in task 3.8; this guards the half we own, so a later change that
//! makes parsing quadratic shows up as a failing test rather than as a vague
//! feeling that typing got worse.

use gpui_editor::buffer::Buffer;
use gpui_editor::display::DisplayLine;
use markdown_lite::{parse_line, render_plan};
use std::time::Instant;

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

/// Lines that fit on screen at once — the ones that actually get shaped.
const VISIBLE_LINES: usize = 30;
/// One frame at 60fps. The real budget is smaller, since shaping and painting
/// also have to fit, but exceeding even this means something has gone wrong.
const FRAME_BUDGET_MS: f64 = 16.0;

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
        // Heights for every line.
        for index in 0..buffer.line_count() {
            std::hint::black_box(parse_line(buffer.line(index)).heading_level());
        }
        // Render plans for the visible window.
        for index in cursor_line..(cursor_line + VISIBLE_LINES).min(buffer.line_count()) {
            let line = buffer.line(index);
            let plan = render_plan(line, index == cursor_line);
            std::hint::black_box(DisplayLine::new(line, &plan));
        }
    }
    let per_frame = start.elapsed().as_secs_f64() * 1000. / FRAMES as f64;

    println!("{:.2} ms per frame over {} lines", per_frame, buffer.line_count());
    assert!(
        per_frame < FRAME_BUDGET_MS,
        "layout work took {per_frame:.2}ms per frame, over the {FRAME_BUDGET_MS}ms budget"
    );
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
