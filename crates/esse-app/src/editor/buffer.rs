//! Text buffer with cursor, selection and undo — everything about editing that
//! does not depend on the GUI framework.
//!
//! Carried over from the stage-0 prototype, which is the half of it that was
//! declared worth keeping: nothing about word wrap changes byte-offset text
//! storage (design.md, D1). Wrapping lives a layer up, in `wrap`.
//!
//! Positions are byte offsets into the buffer text, always on a `char`
//! boundary. Horizontal movement steps by `char`, so Cyrillic (two bytes each)
//! moves one letter at a time. Grapheme clusters are not handled.

use std::ops::Range;

/// Where the caret sits, and what is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Fixed end of the selection, set when the selection started.
    pub anchor: usize,
    /// Moving end — where the caret is drawn.
    pub head: usize,
}

impl Selection {
    pub fn cursor(at: usize) -> Self {
        Selection {
            anchor: at,
            head: at,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    pub fn range(&self) -> Range<usize> {
        if self.anchor <= self.head {
            self.anchor..self.head
        } else {
            self.head..self.anchor
        }
    }
}

/// A line/column position, both zero-based. Columns count `char`s, not bytes,
/// so they can be compared across lines with different encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// What kind of edit was last applied, for undo coalescing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditKind {
    /// Typing ordinary characters — these merge into one undo step.
    Typing,
    /// Backspace/delete — merge with each other, not with typing.
    Deleting,
    /// Paste, newline, or anything else that deserves its own undo step.
    Discrete,
}

#[derive(Debug, Clone)]
struct Snapshot {
    text: String,
    selection: Selection,
}

/// The editable document.
#[derive(Debug, Clone)]
pub struct Buffer {
    text: String,
    /// Byte offset of the start of each line. Always non-empty, always begins
    /// with 0; recomputed after every edit.
    line_starts: Vec<usize>,
    selection: Selection,
    /// Column to aim for during vertical movement, so passing through a short
    /// line does not lose the original column.
    goal_column: Option<usize>,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    last_edit: Option<EditKind>,
    /// Bumped by every edit. The autosave debounce watches this rather than
    /// comparing whole documents.
    revision: u64,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new("")
    }
}

impl Buffer {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let mut buffer = Buffer {
            line_starts: Vec::new(),
            text,
            selection: Selection::cursor(0),
            goal_column: None,
            undo: Vec::new(),
            redo: Vec::new(),
            last_edit: None,
            revision: 0,
        };
        buffer.reindex();
        buffer
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Changes since the buffer was built. Only ever goes up.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn cursor(&self) -> usize {
        self.selection.head
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// The line's text, without its trailing newline.
    pub fn line(&self, index: usize) -> &str {
        let range = self.line_range(index);
        &self.text[range]
    }

    /// Byte range of the line's text, excluding the trailing newline.
    pub fn line_range(&self, index: usize) -> Range<usize> {
        let start = self.line_starts[index];
        let end = match self.line_starts.get(index + 1) {
            // Step back over the '\n' that starts the next line.
            Some(next) => next - 1,
            None => self.text.len(),
        };
        start..end
    }

    /// Index of the line containing `offset`.
    pub fn line_at(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(next) => next - 1,
        }
    }

    /// The line the caret is on — the one that renders raw.
    pub fn cursor_line(&self) -> usize {
        self.line_at(self.selection.head)
    }

    pub fn position_of(&self, offset: usize) -> Position {
        let line = self.line_at(offset);
        let column = self.text[self.line_starts[line]..offset].chars().count();
        Position { line, column }
    }

    /// Byte offset for a line/column, clamped into the buffer.
    pub fn offset_of(&self, position: Position) -> usize {
        let line = position.line.min(self.line_count() - 1);
        let range = self.line_range(line);
        let text = &self.text[range.clone()];
        match text.char_indices().nth(position.column) {
            Some((byte, _)) => range.start + byte,
            None => range.end,
        }
    }

    pub fn selected_text(&self) -> &str {
        &self.text[self.selection.range()]
    }

    // -- editing ----------------------------------------------------------

    /// Insert text at the caret, replacing the selection.
    pub fn insert(&mut self, text: &str) {
        // A newline or a multi-character insert (paste) starts its own undo
        // step; ordinary typing merges into the one before it. Counted in
        // chars, not bytes — a Cyrillic letter is two bytes and is still one
        // keystroke.
        let kind = if text.chars().count() > 1 || text.contains('\n') {
            EditKind::Discrete
        } else {
            EditKind::Typing
        };
        self.checkpoint(kind);

        let range = self.selection.range();
        self.text.replace_range(range.clone(), text);
        self.reindex();
        // `place_cursor`, not `set_cursor`: moving the caret as part of the
        // edit must not break the run this edit just joined.
        self.place_cursor(range.start + text.len());
    }

    /// Delete the selection, or the character before the caret.
    pub fn backspace(&mut self) {
        self.checkpoint(EditKind::Deleting);
        let range = if self.selection.is_empty() {
            let head = self.selection.head;
            match self.prev_boundary(head) {
                Some(prev) => prev..head,
                None => return,
            }
        } else {
            self.selection.range()
        };
        self.delete_range(range);
    }

    /// Delete the selection, or the character after the caret.
    pub fn delete_forward(&mut self) {
        self.checkpoint(EditKind::Deleting);
        let range = if self.selection.is_empty() {
            let head = self.selection.head;
            match self.next_boundary(head) {
                Some(next) => head..next,
                None => return,
            }
        } else {
            self.selection.range()
        };
        self.delete_range(range);
    }

    /// Delete the selection and return it, for cut.
    pub fn cut(&mut self) -> String {
        let taken = self.selected_text().to_string();
        if !taken.is_empty() {
            self.checkpoint(EditKind::Discrete);
            self.delete_range(self.selection.range());
        }
        taken
    }

    fn delete_range(&mut self, range: Range<usize>) {
        self.text.replace_range(range.clone(), "");
        self.reindex();
        self.place_cursor(range.start);
    }

    // -- undo -------------------------------------------------------------

    /// Save the pre-edit state, unless this edit merges with the previous one.
    fn checkpoint(&mut self, kind: EditKind) {
        self.redo.clear();
        let merges = self.last_edit == Some(kind) && kind != EditKind::Discrete;
        if !merges {
            self.undo.push(Snapshot {
                text: self.text.clone(),
                selection: self.selection,
            });
        }
        self.last_edit = Some(kind);
        self.revision += 1;
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else {
            return false;
        };
        self.redo.push(Snapshot {
            text: self.text.clone(),
            selection: self.selection,
        });
        self.restore(previous);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(Snapshot {
            text: self.text.clone(),
            selection: self.selection,
        });
        self.restore(next);
        true
    }

    fn restore(&mut self, snapshot: Snapshot) {
        self.text = snapshot.text;
        self.reindex();
        self.selection = snapshot.selection;
        self.goal_column = None;
        // The restored state is a fresh starting point: the next keystroke
        // must not merge into the step we just undid.
        self.last_edit = None;
        self.revision += 1;
    }

    // -- movement ---------------------------------------------------------

    /// Place the caret, collapsing any selection. Ends the current undo run:
    /// text typed after a deliberate move is a separate step.
    pub fn set_cursor(&mut self, offset: usize) {
        self.place_cursor(offset);
        self.last_edit = None;
    }

    /// Place the caret without ending the undo run — for edits, which move the
    /// caret as a side effect.
    fn place_cursor(&mut self, offset: usize) {
        self.selection = Selection::cursor(self.clamp(offset));
        self.goal_column = None;
    }

    /// Move the caret's head, keeping the anchor when `extend` is set — the
    /// shift-arrow / mouse-drag path.
    pub fn move_head(&mut self, offset: usize, extend: bool) {
        let head = self.clamp(offset);
        if extend {
            self.selection.head = head;
            self.last_edit = None;
        } else {
            self.set_cursor(head);
        }
        self.goal_column = None;
    }

    pub fn move_left(&mut self, extend: bool) {
        // A plain left-arrow out of a selection lands on its left edge.
        let from = if extend || self.selection.is_empty() {
            self.selection.head
        } else {
            self.move_head(self.selection.range().start, false);
            return;
        };
        let to = self.prev_boundary(from).unwrap_or(from);
        self.move_head(to, extend);
    }

    pub fn move_right(&mut self, extend: bool) {
        let from = if extend || self.selection.is_empty() {
            self.selection.head
        } else {
            self.move_head(self.selection.range().end, false);
            return;
        };
        let to = self.next_boundary(from).unwrap_or(from);
        self.move_head(to, extend);
    }

    /// Jump to the start of the word before the caret — option-left on macOS.
    pub fn move_word_left(&mut self, extend: bool) {
        let mut offset = self.selection.head;
        while let Some(previous) = self.prev_boundary(offset) {
            if !is_word_break(self.char_at(previous)) {
                break;
            }
            offset = previous;
        }
        while let Some(previous) = self.prev_boundary(offset) {
            if is_word_break(self.char_at(previous)) {
                break;
            }
            offset = previous;
        }
        self.move_head(offset, extend);
    }

    /// Jump past the end of the word after the caret — option-right on macOS.
    pub fn move_word_right(&mut self, extend: bool) {
        let mut offset = self.selection.head;
        while let Some(next) = self.next_boundary(offset) {
            if !is_word_break(self.char_at(offset)) {
                break;
            }
            offset = next;
        }
        while let Some(next) = self.next_boundary(offset) {
            if is_word_break(self.char_at(offset)) {
                break;
            }
            offset = next;
        }
        self.move_head(offset, extend);
    }

    pub fn move_up(&mut self, extend: bool) {
        self.move_vertically(-1, extend);
    }

    pub fn move_down(&mut self, extend: bool) {
        self.move_vertically(1, extend);
    }

    fn move_vertically(&mut self, delta: isize, extend: bool) {
        let position = self.position_of(self.selection.head);
        let goal = self.goal_column.unwrap_or(position.column);

        let target = position.line as isize + delta;
        if target < 0 || target >= self.line_count() as isize {
            // Off the top or bottom: go to the very start or end, like every
            // other editor does.
            let edge = if delta < 0 { 0 } else { self.text.len() };
            self.move_head(edge, extend);
            self.goal_column = Some(goal);
            return;
        }

        let offset = self.offset_of(Position {
            line: target as usize,
            column: goal,
        });
        self.move_head(offset, extend);
        self.goal_column = Some(goal);
    }

    pub fn move_to_line_start(&mut self, extend: bool) {
        let line = self.cursor_line();
        self.move_head(self.line_range(line).start, extend);
    }

    pub fn move_to_line_end(&mut self, extend: bool) {
        let line = self.cursor_line();
        self.move_head(self.line_range(line).end, extend);
    }

    pub fn select_all(&mut self) {
        self.selection = Selection {
            anchor: 0,
            head: self.text.len(),
        };
        self.goal_column = None;
        self.last_edit = None;
    }

    // -- internals --------------------------------------------------------

    fn reindex(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0);
        self.line_starts
            .extend(self.text.match_indices('\n').map(|(i, _)| i + 1));
    }

    /// Clamp to the buffer and snap onto a `char` boundary, so a caret can
    /// never land inside a multi-byte character.
    fn clamp(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.text.len());
        while !self.text.is_char_boundary(offset) {
            offset -= 1;
        }
        offset
    }

    fn char_at(&self, offset: usize) -> char {
        self.text[offset..].chars().next().unwrap_or('\n')
    }

    fn prev_boundary(&self, offset: usize) -> Option<usize> {
        self.text[..offset]
            .chars()
            .next_back()
            .map(|c| offset - c.len_utf8())
    }

    fn next_boundary(&self, offset: usize) -> Option<usize> {
        self.text[offset..]
            .chars()
            .next()
            .map(|c| offset + c.len_utf8())
    }
}

/// Where a word jump stops. Letters, digits and the apostrophes inside words
/// are word; everything else — spaces, punctuation, markdown markers — is not.
fn is_word_break(ch: char) -> bool {
    !ch.is_alphanumeric() && ch != '\'' && ch != '’'
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let lines: Vec<_> = (0..=buffer.text().len())
            .map(|o| buffer.line_at(o))
            .collect();
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

    // -- movement ---------------------------------------------------------

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
        assert_eq!(
            buffer.position_of(buffer.cursor()).column,
            1,
            "short line clamps"
        );

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
    fn word_jumps_step_over_whole_words_including_cyrillic() {
        let mut buffer = Buffer::new("одно слово, второе");
        buffer.set_cursor(0);

        buffer.move_word_right(false);
        assert_eq!(&buffer.text()[..buffer.cursor()], "одно");
        buffer.move_word_right(false);
        assert_eq!(&buffer.text()[..buffer.cursor()], "одно слово");
        buffer.move_word_right(false);
        assert_eq!(buffer.cursor(), buffer.text().len());
        buffer.move_word_right(false);
        assert_eq!(buffer.cursor(), buffer.text().len(), "stops at the end");

        buffer.move_word_left(false);
        assert_eq!(&buffer.text()[..buffer.cursor()], "одно слово, ");
        buffer.move_word_left(false);
        assert_eq!(&buffer.text()[..buffer.cursor()], "одно ");
        buffer.move_word_left(false);
        assert_eq!(buffer.cursor(), 0);
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

    // -- selection --------------------------------------------------------

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

    // -- clipboard round-trip ---------------------------------------------

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

    // -- deletion ---------------------------------------------------------

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

    // -- undo / redo ------------------------------------------------------

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
        assert_eq!(
            buffer.selected_text(),
            "hello",
            "the replaced text is selected again"
        );
    }

    #[test]
    fn undo_on_an_untouched_buffer_reports_nothing_to_do() {
        let mut buffer = Buffer::new("text");
        assert!(!buffer.undo());
        assert!(!buffer.redo());
    }

    #[test]
    fn the_revision_moves_with_every_change_and_never_back() {
        let mut buffer = Buffer::new("text");
        let fresh = buffer.revision();

        buffer.move_to_line_end(false);
        assert_eq!(buffer.revision(), fresh, "moving the caret is not a change");

        buffer.insert("!");
        let typed = buffer.revision();
        assert!(typed > fresh);

        buffer.undo();
        assert!(buffer.revision() > typed, "undo is a change of its own");
    }

    // -- robustness -------------------------------------------------------

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
}
