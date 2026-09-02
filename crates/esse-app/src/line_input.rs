//! The app's one plain line of text: the capture line on Today, and the
//! publication link in the completion overlay.
//!
//! A hand-built single-line field, not a small editor: text, a caret, a
//! selection, IME composition, the clipboard, Enter, and the deletions and
//! jumps the platform expects — and deliberately nothing else. Undo belongs to
//! the real editor; a box you type one line into does not need it, and it would
//! have to be written twice. Everything that *is* here is here because a hand
//! reaches for it without thinking: shift over a word, option-backspace, a
//! pasted publication link.
//!
//! One *line* of text, but not one row: a spark longer than the field is wide
//! wraps onto as many rows as it needs and the field grows downwards, rather
//! than running off the edge of the screen.

use std::ops::Range;

use gpui::{
    actions, div, fill, point, prelude::*, px, relative, rgb, rgba, size, App, AvailableSpace,
    Bounds, ClipboardItem, Context, CursorStyle, Element, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, FontStyle, GlobalElementId,
    InspectorElementId, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PaintQuad, Pixels, Point, ShapedGlyph, SharedString, Style, TextAlign, TextRun, UTF16Selection,
    UnderlineStyle, Window, WrappedLine,
};

// One idea of what a word is for the whole app: the field and the editor stop
// an option-backspace in the same places.
use crate::editor::buffer::is_word_break;
use crate::theme;

actions!(
    line_input,
    [
        Backspace,
        Copy,
        Cut,
        Delete,
        DeleteToStart,
        DeleteWordLeft,
        DeleteWordRight,
        End,
        Home,
        Left,
        Paste,
        Right,
        SelectAll,
        SelectEnd,
        SelectHome,
        SelectLeft,
        SelectRight,
        SelectWordLeft,
        SelectWordRight,
        Submit,
        WordLeft,
        WordRight,
    ]
);

/// The field's baseline: moving, selecting, deleting, the clipboard, and Enter.
/// The same set the editor binds, minus undo. Like the editor's own these are
/// platform conventions rather than vocabulary this app invented, so they live
/// here rather than in the keymap table and help does not list them
/// (design.md, D1).
pub fn key_bindings() -> Vec<gpui::KeyBinding> {
    use gpui::KeyBinding as Key;
    const INPUT: Option<&str> = Some(crate::keymap::LINE_INPUT);
    vec![
        Key::new("enter", Submit, INPUT),
        Key::new("backspace", Backspace, INPUT),
        Key::new("delete", Delete, INPUT),
        Key::new("alt-backspace", DeleteWordLeft, INPUT),
        Key::new("alt-delete", DeleteWordRight, INPUT),
        Key::new("cmd-backspace", DeleteToStart, INPUT),
        Key::new("left", Left, INPUT),
        Key::new("right", Right, INPUT),
        Key::new("alt-left", WordLeft, INPUT),
        Key::new("alt-right", WordRight, INPUT),
        Key::new("home", Home, INPUT),
        Key::new("end", End, INPUT),
        Key::new("cmd-left", Home, INPUT),
        Key::new("cmd-right", End, INPUT),
        Key::new("shift-left", SelectLeft, INPUT),
        Key::new("shift-right", SelectRight, INPUT),
        Key::new("alt-shift-left", SelectWordLeft, INPUT),
        Key::new("alt-shift-right", SelectWordRight, INPUT),
        Key::new("shift-home", SelectHome, INPUT),
        Key::new("shift-end", SelectEnd, INPUT),
        Key::new("cmd-shift-left", SelectHome, INPUT),
        Key::new("cmd-shift-right", SelectEnd, INPUT),
        Key::new("cmd-a", SelectAll, INPUT),
        Key::new("cmd-c", Copy, INPUT),
        Key::new("cmd-x", Cut, INPUT),
        Key::new("cmd-v", Paste, INPUT),
    ]
}

/// The line was submitted. The text travels with the event: the field is
/// cleared by whoever saved it, and only once the spark is on disk.
pub struct Submitted(pub String);

/// The line being typed and where the caret sits in it.
///
/// Kept apart from the view because this is the part multi-byte text breaks —
/// every offset here is a byte offset, the platform speaks UTF-16 units, and
/// the app is written in a language where no character is one byte. Free of
/// gpui, so it is tested directly.
#[derive(Default)]
struct Line {
    text: String,
    /// The caret, as a byte offset into `text`. The moving end of a selection.
    cursor: usize,
    /// The end a selection was started from. Equal to `cursor` when nothing is
    /// selected, which is most of the time.
    anchor: usize,
}

impl Line {
    fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.anchor = 0;
    }

    /// What is selected, low end first. Empty when the selection is a caret.
    fn selection(&self) -> Range<usize> {
        if self.cursor <= self.anchor {
            self.cursor..self.anchor
        } else {
            self.anchor..self.cursor
        }
    }

    fn selected(&self) -> bool {
        self.cursor != self.anchor
    }

    /// Puts the caret somewhere, either dragging the selection along behind it
    /// or dropping it — which is the whole of what shift does in this field.
    fn move_head(&mut self, offset: usize, extend: bool) {
        self.cursor = offset;
        if !extend {
            self.anchor = offset;
        }
    }

    /// Deletes the selection. False when there is none — which is what makes
    /// this the first line of every deletion below.
    fn delete_selection(&mut self) -> bool {
        if !self.selected() {
            return false;
        }
        let range = self.selection();
        self.text.replace_range(range.clone(), "");
        self.move_head(range.start, false);
        true
    }

    /// Deletes the selection, or the character before the caret. False when
    /// there is neither.
    fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let Some(previous) = self.previous_boundary() else {
            return false;
        };
        self.text.replace_range(previous..self.cursor, "");
        self.move_head(previous, false);
        true
    }

    /// Deletes the selection, or the character after the caret. False when
    /// there is neither.
    fn delete(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let Some(next) = self.next_boundary() else {
            return false;
        };
        self.text.replace_range(self.cursor..next, "");
        true
    }

    /// Deletes the word before the caret — option-backspace. Whatever stands
    /// between the caret and that word, a space or a comma, goes with it: this
    /// is what every other field on the platform does.
    fn delete_word_left(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let start = self.word_start();
        if start == self.cursor {
            return false;
        }
        self.text.replace_range(start..self.cursor, "");
        self.move_head(start, false);
        true
    }

    /// The same forwards — option-delete.
    fn delete_word_right(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let end = self.word_end();
        if end == self.cursor {
            return false;
        }
        self.text.replace_range(self.cursor..end, "");
        true
    }

    /// Deletes everything before the caret — command-backspace. The field holds
    /// one line, so that is the whole of what has been typed up to here.
    fn delete_to_start(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor == 0 {
            return false;
        }
        self.text.replace_range(0..self.cursor, "");
        self.move_head(0, false);
        true
    }

    /// The start of the word before the caret: back over whatever separates
    /// them, then back over the word itself — the two steps the editor's own
    /// word jump takes.
    fn word_start(&self) -> usize {
        let mut offset = self.cursor;
        while let Some(previous) = self.boundary_before(offset) {
            if !is_word_break(self.char_at(previous)) {
                break;
            }
            offset = previous;
        }
        while let Some(previous) = self.boundary_before(offset) {
            if is_word_break(self.char_at(previous)) {
                break;
            }
            offset = previous;
        }
        offset
    }

    /// The end of the word after the caret, by the same two steps forwards.
    fn word_end(&self) -> usize {
        let mut offset = self.cursor;
        while let Some(next) = self.boundary_after(offset) {
            if !is_word_break(self.char_at(offset)) {
                break;
            }
            offset = next;
        }
        while let Some(next) = self.boundary_after(offset) {
            if is_word_break(self.char_at(offset)) {
                break;
            }
            offset = next;
        }
        offset
    }

    fn move_left(&mut self, extend: bool) -> bool {
        // A plain left out of a selection lands on its left edge, rather than
        // one character back from wherever the moving end happened to be.
        if !extend && self.selected() {
            self.move_head(self.selection().start, false);
            return true;
        }
        match self.previous_boundary() {
            Some(previous) => {
                self.move_head(previous, extend);
                true
            }
            None => false,
        }
    }

    fn move_right(&mut self, extend: bool) -> bool {
        if !extend && self.selected() {
            self.move_head(self.selection().end, false);
            return true;
        }
        match self.next_boundary() {
            Some(next) => {
                self.move_head(next, extend);
                true
            }
            None => false,
        }
    }

    /// Jump to the start of the word before the caret — option-left.
    fn move_word_left(&mut self, extend: bool) -> bool {
        let target = self.word_start();
        if target == self.cursor && !self.selected() {
            return false;
        }
        self.move_head(target, extend);
        true
    }

    /// Jump past the end of the word after the caret — option-right.
    fn move_word_right(&mut self, extend: bool) -> bool {
        let target = self.word_end();
        if target == self.cursor && !self.selected() {
            return false;
        }
        self.move_head(target, extend);
        true
    }

    fn move_to_start(&mut self, extend: bool) {
        self.move_head(0, extend);
    }

    fn move_to_end(&mut self, extend: bool) {
        self.move_head(self.text.len(), extend);
    }

    /// The whole line, with the caret at its end — command-a.
    fn select_all(&mut self) {
        self.anchor = 0;
        self.cursor = self.text.len();
    }

    fn previous_boundary(&self) -> Option<usize> {
        self.boundary_before(self.cursor)
    }

    fn next_boundary(&self) -> Option<usize> {
        self.boundary_after(self.cursor)
    }

    fn boundary_before(&self, offset: usize) -> Option<usize> {
        self.text[..offset]
            .chars()
            .next_back()
            .map(|ch| offset - ch.len_utf8())
    }

    fn boundary_after(&self, offset: usize) -> Option<usize> {
        self.text[offset..]
            .chars()
            .next()
            .map(|ch| offset + ch.len_utf8())
    }

    /// The character at a byte offset. Past the end of the line there is none,
    /// and a space answers for it: the end of the line breaks a word.
    fn char_at(&self, offset: usize) -> char {
        self.text[offset..].chars().next().unwrap_or(' ')
    }

    /// Replaces a byte range with new text and leaves the caret after it, with
    /// nothing selected — typing over a selection replaces it. This is one
    /// line, so anything the platform hands over is flattened.
    fn splice(&mut self, range: Range<usize>, new_text: &str) {
        let flattened = if new_text.contains(['\n', '\r']) {
            new_text.replace(['\n', '\r'], " ")
        } else {
            new_text.to_string()
        };
        self.text.replace_range(range.clone(), &flattened);
        self.move_head(range.start + flattened.len(), false);
    }

    // -- UTF-16 bridge ---------------------------------------------------
    //
    // The platform's input protocol counts UTF-16 units; the line stores
    // UTF-8. Cyrillic and IME composition both cross this boundary.

    fn offset_from_utf16(&self, target: usize) -> usize {
        let mut utf8 = 0;
        let mut utf16 = 0;
        for ch in self.text.chars() {
            if utf16 >= target {
                break;
            }
            utf16 += ch.len_utf16();
            utf8 += ch.len_utf8();
        }
        utf8
    }

    fn offset_to_utf16(&self, target: usize) -> usize {
        let mut utf8 = 0;
        let mut utf16 = 0;
        for ch in self.text.chars() {
            if utf8 >= target {
                break;
            }
            utf8 += ch.len_utf8();
            utf16 += ch.len_utf16();
        }
        utf16
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }
}

pub struct LineInput {
    focus_handle: FocusHandle,
    /// What the empty field says it is for.
    placeholder: SharedString,
    line: Line,
    /// What the IME is composing, in bytes.
    marked_range: Option<Range<usize>>,
    /// The mouse is down and dragging a selection out.
    is_selecting: bool,
    /// The last painted frame: where the field was and how the line was broken
    /// into rows. A click and the IME candidate window both read it.
    last_bounds: Option<Bounds<Pixels>>,
    last_field: Option<Field>,
}

impl EventEmitter<Submitted> for LineInput {}

impl LineInput {
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        LineInput {
            focus_handle: cx.focus_handle(),
            placeholder: placeholder.into(),
            line: Line::default(),
            marked_range: None,
            is_selecting: false,
            last_bounds: None,
            last_field: None,
        }
    }

    /// The line as it stands, for a caller that reads it on its own terms
    /// rather than on Enter — the publication link, read when Publish is
    /// confirmed.
    pub fn text(&self) -> &str {
        &self.line.text
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.line.clear();
        self.marked_range = None;
        cx.notify();
    }

    // -- editing ---------------------------------------------------------

    fn submit(&mut self, _: &Submit, _: &mut Window, cx: &mut Context<Self>) {
        // Mid-composition the Enter key belongs to the IME, not to us.
        if self.marked_range.is_some() {
            return;
        }
        cx.emit(Submitted(self.line.text.clone()));
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.backspace() {
            cx.notify();
        }
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.delete() {
            cx.notify();
        }
    }

    fn delete_word_left(&mut self, _: &DeleteWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.delete_word_left() {
            cx.notify();
        }
    }

    fn delete_word_right(&mut self, _: &DeleteWordRight, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.delete_word_right() {
            cx.notify();
        }
    }

    fn delete_to_start(&mut self, _: &DeleteToStart, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.delete_to_start() {
            cx.notify();
        }
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_left(false) {
            cx.notify();
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_right(false) {
            cx.notify();
        }
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_left(true) {
            cx.notify();
        }
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_right(true) {
            cx.notify();
        }
    }

    fn word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_word_left(false) {
            cx.notify();
        }
    }

    fn word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_word_right(false) {
            cx.notify();
        }
    }

    fn select_word_left(&mut self, _: &SelectWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_word_left(true) {
            cx.notify();
        }
    }

    fn select_word_right(&mut self, _: &SelectWordRight, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_word_right(true) {
            cx.notify();
        }
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.line.move_to_start(false);
        cx.notify();
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.line.move_to_end(false);
        cx.notify();
    }

    fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.line.move_to_start(true);
        cx.notify();
    }

    fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.line.move_to_end(true);
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.line.select_all();
        cx.notify();
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        let selected = self.line.text[self.line.selection()].to_string();
        if !selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected));
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        let selected = self.line.text[self.line.selection()].to_string();
        if selected.is_empty() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(selected));
        self.line.delete_selection();
        cx.notify();
    }

    /// Whatever the clipboard carries arrives as one line: a link pasted from
    /// a browser brings a trailing newline often enough to matter.
    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if self.marked_range.is_some() {
            return;
        }
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        if text.is_empty() {
            return;
        }
        self.line.splice(self.line.selection(), &text);
        cx.notify();
    }

    // -- mouse -----------------------------------------------------------

    /// Where a window position lands in the line. None before the first frame,
    /// and while the field is empty — there is nothing there to land in.
    fn offset_for_position(&self, position: Point<Pixels>) -> Option<usize> {
        let bounds = self.last_bounds?;
        let field = self.last_field.as_ref()?;
        Some(field.offset_at(position - bounds.origin))
    }

    /// A click moves the caret; it does not take the focus. Where the field
    /// lives, the screen around it decides who holds the keys — the spark
    /// choice borrows them on Today, the steps do in the completion panel —
    /// and a field that grabbed them back would leave that screen thinking
    /// otherwise.
    fn on_mouse_down(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(offset) = self.offset_for_position(event.position) else {
            return;
        };
        self.is_selecting = true;
        // Shift-click extends what is already selected, like everywhere else.
        self.line.move_head(offset, event.modifiers.shift);
        cx.notify();
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.is_selecting {
            return;
        }
        if let Some(offset) = self.offset_for_position(event.position) {
            self.line.move_head(offset, true);
            cx.notify();
        }
    }
}

impl Focusable for LineInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// Typed text and IME composition both arrive through this trait.
impl EntityInputHandler for LineInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.line.range_from_utf16(&range_utf16);
        actual_range.replace(self.line.range_to_utf16(&range));
        Some(self.line.text[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.line.range_to_utf16(&self.line.selection()),
            reversed: self.line.cursor < self.line.anchor,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.line.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.line.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            // Nothing said otherwise: typing goes over the selection, and the
            // caret is an empty one.
            .unwrap_or_else(|| self.line.selection());

        self.line.splice(range, new_text);
        self.marked_range = None;
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range| self.line.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.line.selection());

        let start = range.start;
        self.line.splice(range, new_text);
        self.marked_range = (!new_text.is_empty()).then_some(start..self.line.cursor);

        // The IME reports where the caret sits inside the text it is composing.
        if let Some(selected) = new_selected_range_utf16 {
            let head = start + self.line.range_from_utf16(&selected).start;
            self.line.move_head(head, false);
        }
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        // Where the IME candidate window should appear. A composition that ran
        // on to the next row is reported on the row it started on, ending where
        // that row does; the panel only needs somewhere to sit.
        let field = self.last_field.as_ref()?;
        let range = self.line.range_from_utf16(&range_utf16);
        let row = field.rows.at(range.start);
        let within = field.rows.range(row);
        let top = element_bounds.top() + field.line_height * row as f32;
        Some(Bounds::from_corners(
            point(
                element_bounds.left()
                    + field.x_in_row(range.start.clamp(within.start, within.end), row),
                top,
            ),
            point(
                element_bounds.left()
                    + field.x_in_row(range.end.clamp(within.start, within.end), row),
                top + field.line_height,
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let field = self.last_field.as_ref()?;
        let index = field.offset_at(point - bounds.origin);
        Some(self.line.offset_to_utf16(index))
    }
}

impl Render for LineInput {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context(crate::keymap::LINE_INPUT)
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .w_full()
            .pb(px(theme::SPACE_S))
            .border_b_1()
            .border_color(rgb(theme::RULE))
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_word_left))
            .on_action(cx.listener(Self::delete_word_right))
            .on_action(cx.listener(Self::delete_to_start))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::word_left))
            .on_action(cx.listener(Self::word_right))
            .on_action(cx.listener(Self::select_word_left))
            .on_action(cx.listener(Self::select_word_right))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .child(LineInputElement { input: cx.entity() })
    }
}

// -- element -------------------------------------------------------------
//
// The line is painted by hand rather than with a text element: the caret, the
// IME underline and the wrap all need the shaped line's own geometry.

struct LineInputElement {
    input: Entity<LineInput>,
}

impl IntoElement for LineInputElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

struct PrepaintState {
    field: Option<Field>,
    /// The selection, one quad per row it covers.
    selection: Vec<PaintQuad>,
    caret: PaintQuad,
    /// The field is showing the placeholder, so its geometry must not be kept
    /// for hit-testing.
    placeholder: bool,
}

impl Element for LineInputElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        _: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();

        // The line wraps, so how tall the field is depends on how wide it is —
        // and the width is only known here, when the layout offers one. Hence a
        // measured layout rather than the one fixed row this field used to be.
        let input = self.input.clone();
        let layout_id =
            window.request_measured_layout(style, move |known, available, window, cx| {
                let wrap_width = known.width.or(match available.width {
                    AvailableSpace::Definite(width) => Some(width),
                    _ => None,
                });
                let field = shape(&content(input.read(cx), window), wrap_width, window);
                size(
                    wrap_width.unwrap_or(field.wrapped.unwrapped_layout.width),
                    field.height(),
                )
            });
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = content(input, window);
        let field = shape(&content, Some(bounds.size.width), window);

        // The placeholder is not the line, so the caret stays at its start.
        let caret_at = if content.placeholder {
            0
        } else {
            input.line.cursor
        };
        let caret = fill(
            Bounds::new(
                bounds.origin + field.position(caret_at),
                size(px(2.), field.line_height),
            ),
            rgb(theme::CARET),
        );
        let selection = if content.placeholder {
            Vec::new()
        } else {
            selection_quads(input.line.selection(), &field, bounds.origin)
        };

        PrepaintState {
            field: Some(field),
            selection,
            caret,
            placeholder: content.placeholder,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );

        let Some(field) = prepaint.field.take() else {
            return;
        };
        // The selection goes down first, under the text it is behind.
        for quad in std::mem::take(&mut prepaint.selection) {
            window.paint_quad(quad);
        }
        // One call paints every row: the shaper already knows where it broke.
        if let Err(error) = field.wrapped.paint(
            bounds.origin,
            field.line_height,
            TextAlign::Left,
            None,
            window,
            cx,
        ) {
            log::error!("could not paint the line: {error:?}");
        }
        if focus_handle.is_focused(window) {
            window.paint_quad(prepaint.caret.clone());
        }

        let placeholder = prepaint.placeholder;
        self.input.update(cx, |input, _| {
            input.last_bounds = Some(bounds);
            input.last_field = (!placeholder).then_some(field);
        });
    }
}

/// What the field draws, and the runs to draw it in: the line itself, or the
/// placeholder when it is empty.
struct Content {
    text: SharedString,
    runs: Vec<TextRun>,
    placeholder: bool,
}

/// Where the shaper broke the line, as byte offsets into it — one entry per
/// row. Free of gpui, so the row arithmetic is tested directly.
#[derive(Debug, PartialEq, Eq)]
struct Rows {
    /// Byte offset each row starts at. Always opens with 0, so there is always
    /// at least one row — an empty line included.
    starts: Vec<usize>,
    len: usize,
}

impl Rows {
    /// `boundaries` are the byte offsets the shaper wrapped at, in order.
    fn new(len: usize, boundaries: impl IntoIterator<Item = usize>) -> Self {
        let mut starts = vec![0];
        starts.extend(
            boundaries
                .into_iter()
                // A boundary at either end would make a row no offset can ever
                // be in; the shaper should not produce one, but a wrong row
                // count would put the caret on the wrong row.
                .filter(|start| *start > 0 && *start < len),
        );
        starts.dedup();
        Rows { starts, len }
    }

    fn count(&self) -> usize {
        self.starts.len()
    }

    /// Byte range of one row. Rows tile the line.
    fn range(&self, row: usize) -> Range<usize> {
        let row = row.min(self.count() - 1);
        let start = self.starts[row];
        let end = self.starts.get(row + 1).copied().unwrap_or(self.len);
        start..end
    }

    /// The row an offset is drawn on — the last one that starts at or before
    /// it. Exactly at a wrap the offset opens the later row, so the caret sits
    /// before the word it is about to continue rather than out in the margin.
    fn at(&self, offset: usize) -> usize {
        self.starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1)
    }
}

/// The shaped field: the wrapped text, the rows it fell into, and how tall a
/// row is. Everything that needs a position on screen measures against it.
struct Field {
    wrapped: WrappedLine,
    rows: Rows,
    line_height: Pixels,
}

impl Field {
    fn height(&self) -> Pixels {
        self.line_height * self.rows.count() as f32
    }

    /// Where an offset is drawn, from the top-left corner of the field.
    fn position(&self, offset: usize) -> Point<Pixels> {
        let row = self.rows.at(offset);
        point(self.x_in_row(offset, row), self.line_height * row as f32)
    }

    /// The x of an offset on a given row, measured from the left edge. The end
    /// of a wrapped row *is* the start of the next one, so an offset there has
    /// to be measured against the row being drawn rather than the one it opens.
    fn x_in_row(&self, offset: usize, row: usize) -> Pixels {
        let layout = &self.wrapped.unwrapped_layout;
        layout.x_for_index(offset) - layout.x_for_index(self.rows.starts[row])
    }

    /// The offset closest to a point measured from that same corner — how a
    /// click lands. Clamped to the row it falls on, so overshooting to the
    /// right stops at the end of that row rather than running on to the next.
    fn offset_at(&self, position: Point<Pixels>) -> usize {
        let row =
            ((position.y / self.line_height).floor().max(0.) as usize).min(self.rows.count() - 1);
        let within = self.rows.range(row);
        let layout = &self.wrapped.unwrapped_layout;
        let row_x = layout.x_for_index(within.start);
        layout
            .closest_index_for_x(position.x + row_x)
            .clamp(within.start, within.end)
    }
}

/// The selection as quads, one per row it runs across. A selection that ends
/// exactly at a wrap covers nothing on the row it opens, and that row is
/// skipped rather than drawn as a hairline.
fn selection_quads(range: Range<usize>, field: &Field, origin: Point<Pixels>) -> Vec<PaintQuad> {
    if range.is_empty() {
        return Vec::new();
    }

    let mut quads = Vec::new();
    for row in field.rows.at(range.start)..=field.rows.at(range.end) {
        let within = field.rows.range(row);
        let left = field.x_in_row(range.start.max(within.start), row);
        let right = field.x_in_row(range.end.min(within.end), row);
        if right <= left {
            continue;
        }
        let top = field.line_height * row as f32;
        quads.push(fill(
            Bounds::from_corners(
                origin + point(left, top),
                origin + point(right, top + field.line_height),
            ),
            rgba(theme::SELECTION),
        ));
    }
    quads
}

/// The text to draw and the runs to draw it in.
fn content(input: &LineInput, window: &Window) -> Content {
    let mut font = window.text_style().font();
    let placeholder = input.line.text.is_empty();
    // The prompt is set in italic, so an empty line reads as an invitation
    // rather than as a spark someone already captured.
    if placeholder {
        font.style = FontStyle::Italic;
    }

    let text: SharedString = if placeholder {
        input.placeholder.clone()
    } else {
        input.line.text.clone().into()
    };
    let color = if placeholder {
        rgb(theme::MUTED).into()
    } else {
        rgb(theme::INK).into()
    };

    // Text the IME is still composing is underlined, so it reads as
    // provisional.
    let marked = input.marked_range.clone().filter(|_| !placeholder);
    let runs = split_at(text.len(), marked)
        .into_iter()
        .map(|(range, underlined)| TextRun {
            len: range.end - range.start,
            font: font.clone(),
            color,
            background_color: None,
            underline: underlined.then(|| UnderlineStyle {
                color: Some(color),
                thickness: px(1.),
                wavy: false,
            }),
            strikethrough: None,
        })
        .collect();

    Content {
        text,
        runs,
        placeholder,
    }
}

/// Shape the content, wrapping it to a width when there is one to wrap to. The
/// text holds no newline, so the shaper returns exactly one line; the rows
/// inside it are what makes the field grow downwards.
fn shape(content: &Content, wrap_width: Option<Pixels>, window: &Window) -> Field {
    let mut shaped = window
        .text_system()
        .shape_text(
            content.text.clone(),
            px(theme::INPUT_SIZE),
            &content.runs,
            wrap_width,
            None,
        )
        .unwrap_or_default();
    let wrapped = if shaped.is_empty() {
        WrappedLine::default()
    } else {
        shaped.remove(0)
    };

    let rows = Rows::new(content.text.len(), boundary_offsets(&wrapped));
    Field {
        wrapped,
        rows,
        line_height: px(theme::INPUT_SIZE * theme::LINE_SPACING),
    }
}

/// Byte offsets the shaper wrapped at: the first glyph of each row after the
/// first.
fn boundary_offsets(wrapped: &WrappedLine) -> Vec<usize> {
    let layout = &wrapped.unwrapped_layout;
    wrapped
        .wrap_boundaries
        .iter()
        .filter_map(|boundary| {
            layout
                .runs
                .get(boundary.run_ix)?
                .glyphs
                .get(boundary.glyph_ix)
                .map(|glyph: &ShapedGlyph| glyph.index)
        })
        .collect()
}

/// Splits `len` bytes around an optional marked range, as (range, underlined)
/// pairs — at most three runs, and exactly one when nothing is composing.
fn split_at(len: usize, marked: Option<Range<usize>>) -> Vec<(Range<usize>, bool)> {
    let Some(marked) = marked.filter(|range| range.start < range.end && range.end <= len) else {
        return vec![(0..len, false)];
    };
    [
        (0..marked.start, false),
        (marked.start..marked.end, true),
        (marked.end..len, false),
    ]
    .into_iter()
    .filter(|(range, _)| range.start < range.end)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two bytes per letter, so every offset below would be wrong if the field
    /// counted characters as bytes.
    fn line(text: &str) -> Line {
        let mut line = Line::default();
        line.splice(0..0, text);
        line
    }

    /// The caret at a byte offset, with nothing selected.
    fn at(text: &str, cursor: usize) -> Line {
        let mut line = line(text);
        line.move_head(cursor, false);
        line
    }

    /// A line with `range` selected, the caret at its end.
    fn selecting(text: &str, range: Range<usize>) -> Line {
        let mut line = at(text, range.start);
        line.move_head(range.end, true);
        line
    }

    #[test]
    fn backspace_removes_a_whole_character() {
        let mut line = line("искра");
        assert!(line.backspace());

        assert_eq!(line.text, "искр");
        assert_eq!(line.cursor, line.text.len());
    }

    #[test]
    fn backspace_at_the_start_does_nothing() {
        let mut line = Line::default();
        assert!(!line.backspace());
        assert!(line.text.is_empty());
    }

    #[test]
    fn the_caret_moves_by_characters_not_bytes() {
        let mut line = line("да");
        assert!(line.move_left(false));
        assert_eq!(line.cursor, 2);
        assert!(line.move_left(false));
        assert_eq!(line.cursor, 0);
        assert!(!line.move_left(false));

        assert!(line.move_right(false));
        assert_eq!(line.cursor, 2);
        line.move_to_end(false);
        assert!(!line.move_right(false));
    }

    #[test]
    fn delete_removes_the_character_after_the_caret() {
        let mut line = at("мысль", 0);
        assert!(line.delete());

        assert_eq!(line.text, "ысль");
        assert_eq!(line.cursor, 0);
    }

    #[test]
    fn option_backspace_takes_the_word_and_what_separates_it() {
        let mut line = line("первая мысль");

        assert!(line.delete_word_left());
        assert_eq!(line.text, "первая ");
        assert_eq!(line.cursor, line.text.len());

        // The space before the word goes with the word after it.
        assert!(line.delete_word_left());
        assert_eq!(line.text, "");
        assert!(!line.delete_word_left(), "nothing left to take");
    }

    #[test]
    fn option_backspace_leaves_what_is_after_the_caret() {
        let mut line = at("одна и другая", "одна и ".len());

        assert!(line.delete_word_left());
        assert_eq!(line.text, "одна другая");
        assert_eq!(line.cursor, "одна ".len());
    }

    #[test]
    fn option_delete_takes_the_word_after_the_caret() {
        let mut line = at("две мысли", 0);

        assert!(line.delete_word_right());
        assert_eq!(line.text, " мысли");
        assert_eq!(line.cursor, 0, "the caret does not move");
    }

    #[test]
    fn command_backspace_clears_the_line_up_to_the_caret() {
        let mut line = at("одна и другая", "одна и ".len());

        assert!(line.delete_to_start());
        assert_eq!(line.text, "другая");
        assert_eq!(line.cursor, 0);
        assert!(!line.delete_to_start(), "nothing before the caret");
    }

    #[test]
    fn shift_extends_the_selection_and_a_bare_arrow_drops_it() {
        let mut line = line("искра");

        assert!(line.move_left(true));
        assert!(line.move_left(true));
        assert_eq!(line.selection(), 6..10, "two letters, four bytes");
        assert_eq!(&line.text[line.selection()], "ра");

        // A bare arrow out of a selection lands on its edge, not one step in
        // from the moving end.
        assert!(line.move_left(false));
        assert_eq!(line.cursor, 6);
        assert!(!line.selected());
    }

    #[test]
    fn shift_selects_by_words_and_to_the_ends() {
        let mut line = line("первая мысль");

        assert!(line.move_word_left(true));
        assert_eq!(&line.text[line.selection()], "мысль");

        line.move_to_start(true);
        assert_eq!(&line.text[line.selection()], "первая мысль");
        assert_eq!(line.cursor, 0, "the caret is the moving end");

        line.select_all();
        assert_eq!(line.selection(), 0..line.text.len());
    }

    #[test]
    fn typing_over_a_selection_replaces_it() {
        let mut line = selecting("одна мысль", 0..8);
        assert_eq!(&line.text[line.selection()], "одна");

        line.splice(line.selection(), "другая");
        assert_eq!(line.text, "другая мысль");
        assert_eq!(line.cursor, 12, "the caret follows the new word");
        assert!(!line.selected());
    }

    #[test]
    fn every_deletion_takes_the_selection_whole() {
        for take in [
            Line::backspace,
            Line::delete,
            Line::delete_word_left,
            Line::delete_word_right,
            Line::delete_to_start,
        ] {
            let mut line = selecting("одна мысль", 0..8);
            assert!(take(&mut line));

            assert_eq!(line.text, " мысль", "the selection, and nothing beside it");
            assert_eq!(line.cursor, 0);
            assert!(!line.selected());
        }
    }

    #[test]
    fn an_unwrapped_line_is_one_row() {
        let rows = Rows::new(11, []);
        assert_eq!(rows.count(), 1);
        assert_eq!(rows.range(0), 0..11);
        assert_eq!(rows.at(0), 0);
        assert_eq!(rows.at(11), 0);
    }

    #[test]
    fn an_empty_line_still_has_a_row_to_put_the_caret_on() {
        let rows = Rows::new(0, []);
        assert_eq!(rows.count(), 1);
        assert_eq!(rows.range(0), 0..0);
        assert_eq!(rows.at(0), 0);
    }

    #[test]
    fn rows_tile_the_line() {
        //                    0123456789012
        let rows = Rows::new(11, [4, 8]);
        assert_eq!(rows.count(), 3);
        assert_eq!(rows.range(0), 0..4);
        assert_eq!(rows.range(1), 4..8);
        assert_eq!(rows.range(2), 8..11);

        assert_eq!(rows.at(3), 0);
        // The break itself opens the row after it.
        assert_eq!(rows.at(4), 1);
        assert_eq!(rows.at(11), 2);
    }

    #[test]
    fn boundaries_the_shaper_should_never_send_are_ignored() {
        let rows = Rows::new(7, [0, 4, 4, 99]);
        assert_eq!(rows.count(), 2, "only the real break counts");
        assert_eq!(rows.range(1), 4..7);
    }

    #[test]
    fn splicing_leaves_the_caret_after_the_new_text() {
        // Four bytes in: after "ис", the third character.
        let mut line = at("иса", 4);
        line.splice(4..4, "кр");

        assert_eq!(line.text, "искра");
        assert_eq!(line.cursor, 8);
    }

    #[test]
    fn a_spliced_line_break_becomes_a_space() {
        let mut line = Line::default();
        line.splice(0..0, "две\nстроки");

        assert_eq!(line.text, "две строки");
        assert_eq!(line.cursor, line.text.len());
    }

    #[test]
    fn offsets_survive_the_trip_through_utf16() {
        // "и" is two UTF-8 bytes and one UTF-16 unit; the emoji is four and
        // two — the two ways the counts can disagree.
        let line = line("и🔥а");

        for (utf8, utf16) in [(0, 0), (2, 1), (6, 3), (8, 4)] {
            assert_eq!(line.offset_to_utf16(utf8), utf16, "byte {utf8}");
            assert_eq!(line.offset_from_utf16(utf16), utf8, "unit {utf16}");
        }
        assert_eq!(line.range_from_utf16(&line.range_to_utf16(&(2..6))), 2..6);
    }

    #[test]
    fn composing_text_is_split_into_an_underlined_run() {
        assert_eq!(split_at(10, None), [(0..10, false)]);
        assert_eq!(
            split_at(10, Some(2..6)),
            [(0..2, false), (2..6, true), (6..10, false)]
        );
        // Composition at the very start has no run before it.
        assert_eq!(split_at(10, Some(0..4)), [(0..4, true), (4..10, false)]);
        // A stale range from a previous line is ignored rather than panicking.
        assert_eq!(split_at(4, Some(2..99)), [(0..4, false)]);
    }
}
