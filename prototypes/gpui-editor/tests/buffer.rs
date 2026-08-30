//! Buffer tests — the parts of "baseline editing correctness" that can be
//! checked without a window. Input, IME and clipboard plumbing still have to be
//! verified by hand in the running app.

use gpui_editor::buffer::{Buffer, Position, Selection};

fn typed(text: &str) -> Buffer {
    let mut buffer = Buffer::default();
    for ch in text.chars() {
        buffer.insert(&ch.to_string());
    }
    buffer
}

#[test]
fn typing_builds_text_and_advances_the_caret() {
    let buffer = typed("hello");
    assert_eq!(buffer.text(), "hello");
    assert_eq!(buffer.cursor(), 5);
    assert!(buffer.selection().is_empty());
}

#[test]
fn lines_are_indexed_without_their_newlines() {
    let buffer = Buffer::new("one\ntwo\n\nfour");
    assert_eq!(buffer.line_count(), 4);
    assert_eq!(buffer.line(0), "one");
    assert_eq!(buffer.line(1), "two");
    assert_eq!(buffer.line(2), "");
    assert_eq!(buffer.line(3), "four");
}

#[test]
fn trailing_newline_opens_an_empty_last_line() {
    let buffer = Buffer::new("one\n");
    assert_eq!(buffer.line_count(), 2);
    assert_eq!(buffer.line(1), "");
}

#[test]
fn line_at_maps_every_offset_to_its_line() {
    let buffer = Buffer::new("ab\ncd");
    let lines: Vec<_> = (0..=buffer.text().len()).map(|o| buffer.line_at(o)).collect();
    //                     a  b  \n c  d  end
    assert_eq!(lines, [0, 0, 0, 1, 1, 1]);
}

#[test]
fn cursor_line_tracks_the_caret() {
    let mut buffer = Buffer::new("first\nsecond");
    assert_eq!(buffer.cursor_line(), 0);
    buffer.set_cursor(6);
    assert_eq!(buffer.cursor_line(), 1);
    buffer.move_up(false);
    assert_eq!(buffer.cursor_line(), 0);
}

// -- movement -------------------------------------------------------------

#[test]
fn horizontal_movement_steps_whole_cyrillic_letters() {
    let mut buffer = Buffer::new("да");
    assert_eq!(buffer.text().len(), 4, "two bytes per letter");

    buffer.set_cursor(0);
    buffer.move_right(false);
    assert_eq!(buffer.cursor(), 2, "one letter, not one byte");
    buffer.move_right(false);
    assert_eq!(buffer.cursor(), 4);
    buffer.move_right(false);
    assert_eq!(buffer.cursor(), 4, "stops at the end");

    buffer.move_left(false);
    assert_eq!(buffer.cursor(), 2);
}

#[test]
fn vertical_movement_keeps_the_goal_column_past_short_lines() {
    let mut buffer = Buffer::new("long line here\nx\nanother long line");
    buffer.set_cursor(10);
    assert_eq!(buffer.position_of(buffer.cursor()).column, 10);

    buffer.move_down(false);
    assert_eq!(buffer.position_of(buffer.cursor()).column, 1, "short line clamps");

    buffer.move_down(false);
    assert_eq!(
        buffer.position_of(buffer.cursor()).column,
        10,
        "the original column comes back"
    );
}

#[test]
fn vertical_movement_off_the_ends_goes_to_the_extremes() {
    let mut buffer = Buffer::new("one\ntwo");
    buffer.set_cursor(1);
    buffer.move_up(false);
    assert_eq!(buffer.cursor(), 0);

    buffer.set_cursor(5);
    buffer.move_down(false);
    assert_eq!(buffer.cursor(), buffer.text().len());
}

#[test]
fn home_and_end_use_the_current_line() {
    let mut buffer = Buffer::new("first\nsecond line");
    buffer.set_cursor(9);

    buffer.move_to_line_start(false);
    assert_eq!(buffer.cursor(), 6);
    buffer.move_to_line_end(false);
    assert_eq!(buffer.cursor(), 17);
}

#[test]
fn click_to_position_round_trips_and_clamps() {
    let buffer = Buffer::new("абв\nde");
    assert_eq!(buffer.offset_of(Position { line: 0, column: 2 }), 4);
    assert_eq!(buffer.offset_of(Position { line: 1, column: 1 }), 8);

    // Clicking past the end of a line lands at its end, not the next line.
    assert_eq!(buffer.offset_of(Position { line: 0, column: 99 }), 6);
    // Clicking below the last line lands in it.
    assert_eq!(buffer.offset_of(Position { line: 99, column: 0 }), 7);

    for offset in [0, 2, 4, 6, 7, 8, 9] {
        assert_eq!(buffer.offset_of(buffer.position_of(offset)), offset);
    }
}

// -- selection ------------------------------------------------------------

#[test]
fn shift_arrows_extend_from_a_fixed_anchor() {
    let mut buffer = Buffer::new("hello world");
    buffer.set_cursor(0);
    for _ in 0..5 {
        buffer.move_right(true);
    }

    assert_eq!(buffer.selection(), Selection { anchor: 0, head: 5 });
    assert_eq!(buffer.selected_text(), "hello");

    // Shrinking back keeps the same anchor.
    buffer.move_left(true);
    assert_eq!(buffer.selected_text(), "hell");
}

#[test]
fn selection_extends_backwards_too() {
    let mut buffer = Buffer::new("hello");
    buffer.set_cursor(5);
    buffer.move_left(true);
    buffer.move_left(true);
    assert_eq!(buffer.selected_text(), "lo");
    assert_eq!(buffer.selection().range(), 3..5);
}

#[test]
fn plain_arrow_collapses_a_selection_to_its_edge() {
    let mut buffer = Buffer::new("hello");
    buffer.set_cursor(1);
    for _ in 0..3 {
        buffer.move_right(true);
    }
    assert_eq!(buffer.selection().range(), 1..4);

    buffer.move_right(false);
    assert_eq!(buffer.cursor(), 4);
    assert!(buffer.selection().is_empty());

    buffer.set_cursor(1);
    for _ in 0..3 {
        buffer.move_right(true);
    }
    buffer.move_left(false);
    assert_eq!(buffer.cursor(), 1);
}

#[test]
fn typing_over_a_selection_replaces_it() {
    let mut buffer = Buffer::new("hello world");
    buffer.set_cursor(0);
    for _ in 0..5 {
        buffer.move_right(true);
    }
    buffer.insert("goodbye");
    assert_eq!(buffer.text(), "goodbye world");
    assert_eq!(buffer.cursor(), 7);
}

#[test]
fn select_all_covers_the_document() {
    let mut buffer = Buffer::new("one\ntwo");
    buffer.select_all();
    assert_eq!(buffer.selected_text(), "one\ntwo");
}

// -- clipboard round-trip -------------------------------------------------

// Scenario: Selection and clipboard round-trip.
#[test]
fn cut_and_paste_moves_text_with_its_markers_intact() {
    let mut buffer = Buffer::new("keep **bold bit** and the rest");
    buffer.set_cursor(5);
    for _ in 0.."**bold bit**".chars().count() {
        buffer.move_right(true);
    }

    let clipboard = buffer.cut();
    assert_eq!(clipboard, "**bold bit**", "markers travel with the text");
    assert_eq!(buffer.text(), "keep  and the rest");

    buffer.move_to_line_end(false);
    buffer.insert(&clipboard);
    assert_eq!(buffer.text(), "keep  and the rest**bold bit**");
}

#[test]
fn cut_with_no_selection_changes_nothing() {
    let mut buffer = Buffer::new("text");
    buffer.set_cursor(2);
    assert_eq!(buffer.cut(), "");
    assert_eq!(buffer.text(), "text");
}

// -- deletion -------------------------------------------------------------

#[test]
fn backspace_removes_one_whole_character() {
    let mut buffer = Buffer::new("абв");
    buffer.set_cursor(6);
    buffer.backspace();
    assert_eq!(buffer.text(), "аб");
    buffer.backspace();
    assert_eq!(buffer.text(), "а");
}

#[test]
fn backspace_at_the_start_is_a_no_op() {
    let mut buffer = Buffer::new("text");
    buffer.set_cursor(0);
    buffer.backspace();
    assert_eq!(buffer.text(), "text");
}

#[test]
fn backspace_joins_lines() {
    let mut buffer = Buffer::new("one\ntwo");
    buffer.set_cursor(4);
    buffer.backspace();
    assert_eq!(buffer.text(), "onetwo");
    assert_eq!(buffer.line_count(), 1);
}

#[test]
fn delete_forward_removes_the_next_character() {
    let mut buffer = Buffer::new("abc");
    buffer.set_cursor(1);
    buffer.delete_forward();
    assert_eq!(buffer.text(), "ac");

    buffer.set_cursor(2);
    buffer.delete_forward();
    assert_eq!(buffer.text(), "ac", "no-op at the end");
}

#[test]
fn deleting_a_selection_removes_all_of_it() {
    let mut buffer = Buffer::new("hello world");
    buffer.set_cursor(5);
    for _ in 0..6 {
        buffer.move_right(true);
    }
    buffer.backspace();
    assert_eq!(buffer.text(), "hello");
}

// -- undo / redo ----------------------------------------------------------

// Scenario: Undo restores prior text.
#[test]
fn undo_restores_and_redo_reapplies() {
    let mut buffer = Buffer::new("start");
    buffer.move_to_line_end(false);
    buffer.insert(" more");
    assert_eq!(buffer.text(), "start more");

    assert!(buffer.undo());
    assert_eq!(buffer.text(), "start");

    assert!(buffer.redo());
    assert_eq!(buffer.text(), "start more");
}

#[test]
fn a_typing_run_undoes_as_one_step() {
    let mut buffer = typed("word");
    assert_eq!(buffer.text(), "word");

    assert!(buffer.undo());
    assert_eq!(buffer.text(), "", "the whole run, not one letter");
    assert!(!buffer.undo());
}

#[test]
fn undo_steps_break_between_typing_and_deleting() {
    let mut buffer = typed("word");
    buffer.backspace();
    buffer.backspace();
    assert_eq!(buffer.text(), "wo");

    buffer.undo();
    assert_eq!(buffer.text(), "word", "the deletions undo together");
    buffer.undo();
    assert_eq!(buffer.text(), "");
}

#[test]
fn a_cyrillic_typing_run_undoes_as_one_step() {
    // Each of these letters is two bytes; undo granularity must follow
    // keystrokes, not bytes.
    let mut buffer = typed("слово");
    assert_eq!(buffer.text(), "слово");

    assert!(buffer.undo());
    assert_eq!(buffer.text(), "", "the whole word, not one letter");
    assert!(!buffer.undo());
}

#[test]
fn a_paste_is_its_own_undo_step() {
    let mut buffer = typed("abc");
    buffer.insert("PASTED");
    buffer.undo();
    assert_eq!(buffer.text(), "abc", "paste undoes alone");
    buffer.undo();
    assert_eq!(buffer.text(), "");
}

#[test]
fn moving_the_caret_ends_the_typing_run() {
    let mut buffer = typed("ab");
    buffer.set_cursor(0);
    buffer.insert("X");
    assert_eq!(buffer.text(), "Xab");

    buffer.undo();
    assert_eq!(buffer.text(), "ab", "only the post-move typing is undone");
}

#[test]
fn a_new_edit_after_undo_drops_the_redo_stack() {
    let mut buffer = typed("one");
    buffer.undo();
    buffer.insert("two");
    assert!(!buffer.redo(), "the old redo branch is gone");
    assert_eq!(buffer.text(), "two");
}

#[test]
fn undo_restores_the_selection_too() {
    let mut buffer = Buffer::new("hello world");
    buffer.set_cursor(0);
    for _ in 0..5 {
        buffer.move_right(true);
    }
    buffer.insert("bye");

    buffer.undo();
    assert_eq!(buffer.text(), "hello world");
    assert_eq!(buffer.selected_text(), "hello", "the replaced text is selected again");
}

#[test]
fn undo_on_an_untouched_buffer_reports_nothing_to_do() {
    let mut buffer = Buffer::new("text");
    assert!(!buffer.undo());
    assert!(!buffer.redo());
}

// -- robustness -----------------------------------------------------------

#[test]
fn offsets_never_land_inside_a_character() {
    let mut buffer = Buffer::new("абв\nгде");
    for offset in 0..=buffer.text().len() {
        buffer.set_cursor(offset);
        assert!(
            buffer.text().is_char_boundary(buffer.cursor()),
            "offset {offset} snapped to a bad boundary"
        );
    }
}

#[test]
fn an_essay_sized_document_indexes_correctly() {
    let paragraph = "Пишу длинный абзац с **жирным** и *курсивом*, чтобы набрать объём.\n";
    let text = paragraph.repeat(400);
    let buffer = Buffer::new(text.clone());

    assert!(text.chars().count() > 20_000, "sample must be essay sized");
    assert_eq!(buffer.line_count(), 401);
    assert_eq!(buffer.line(0), paragraph.trim_end());
    assert_eq!(buffer.line(400), "");
    assert_eq!(buffer.line_at(text.len()), 400);
}
