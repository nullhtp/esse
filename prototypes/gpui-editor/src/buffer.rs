//! Text buffer with cursor, selection and undo — everything about editing that
//! does not depend on the GUI framework.
//!
//! Kept framework-free on purpose: it is the part of the prototype that can be
//! tested without a window, which is most of the behaviour the spec calls for.
//!
//! Positions are byte offsets into the buffer text, always on a `char`
//! boundary. Horizontal movement steps by `char`, so Cyrillic (two bytes each)
//! moves one letter at a time. Grapheme clusters are not handled — a
//! prototype-level shortcut, noted for the production editor.

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
        Selection { anchor: at, head: at }
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
        };
        buffer.reindex();
        buffer
    }

    pub fn text(&self) -> &str {
        &self.text
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

    /// The line the caret is on — the one that renders raw (design D4).
    pub fn cursor_line(&self) -> usize {
        self.line_at(self.selection.head)
    }

    pub fn position_of(&self, offset: usize) -> Position {
        let line = self.line_at(offset);
        let column = self.text[self.line_starts[line]..offset].chars().count();
        Position { line, column }
    }

    /// Byte offset for a line/column, clamped into the buffer. This is the
    /// click-to-position entry point: the view converts a point to line/column
    /// and asks for the offset.
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
    }

    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else { return false };
        self.redo.push(Snapshot {
            text: self.text.clone(),
            selection: self.selection,
        });
        self.restore(previous);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else { return false };
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

        let offset = self.offset_of(Position { line: target as usize, column: goal });
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
        self.selection = Selection { anchor: 0, head: self.text.len() };
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

    fn prev_boundary(&self, offset: usize) -> Option<usize> {
        self.text[..offset].chars().next_back().map(|c| offset - c.len_utf8())
    }

    fn next_boundary(&self, offset: usize) -> Option<usize> {
        self.text[offset..].chars().next().map(|c| offset + c.len_utf8())
    }
}
