//! The editor: live markdown-lite, soft word wrap, typewriter centring.
//!
//! Three layers, each testable one below the next:
//!
//! * [`buffer`] — the document. Byte offsets, selection, undo. No framework.
//! * [`display`] — one source line as it is drawn, markers resolved.
//! * [`wrap`] — that line broken into the rows it occupies on screen.
//!
//! The view here binds them to gpui: it owns the buffer, handles keys, mouse
//! and IME, and hands the element what to paint. The element (in `element`)
//! does the shaping and the geometry, and hands the geometry back through
//! `last_paint` — which is what the mouse and vertical movement read.
//!
//! One editor, two modes. [`EditorStyle`] says how it looks and [`Viewport`]
//! says how the view moves — the caret carrying it in Write mode, the scroll
//! position in Edit mode (design.md, D5, D6). Everything below that line is
//! shared, so the two modes cannot drift apart.

pub mod buffer;
pub mod display;
mod element;
#[cfg(test)]
mod performance;
pub mod viewport;
pub mod wrap;

use std::ops::Range;

use gpui::{
    actions, div, prelude::*, px, App, Bounds, ClipboardItem, Context, CursorStyle,
    EntityInputHandler, EventEmitter, FocusHandle, Focusable, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, ScrollWheelEvent, UTF16Selection, Window,
};

use crate::theme;
use buffer::{Buffer, Emphasis};
use element::{EditorElement, PaintedFrame};
use viewport::Scroll;
pub use viewport::Viewport;

actions!(
    editor,
    [
        Backspace,
        Copy,
        Cut,
        Delete,
        DeleteToRowStart,
        DeleteWordLeft,
        DeleteWordRight,
        Down,
        End,
        Heading1,
        Heading2,
        Heading3,
        Home,
        Left,
        Newline,
        Paste,
        Redo,
        Right,
        SelectAll,
        SelectDown,
        SelectEnd,
        SelectHome,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectWordLeft,
        SelectWordRight,
        ToggleBold,
        ToggleItalic,
        Undo,
        Up,
        WordLeft,
        WordRight,
    ]
);

/// The editor's baseline: moving, selecting, deleting, the clipboard, undo.
/// Bound once, at startup, and deliberately kept out of the keymap table —
/// these are what every text field on the platform does, not vocabulary this
/// app invented, so help does not list them (design.md, D1). The markup keys
/// are in the table.
pub fn key_bindings() -> Vec<gpui::KeyBinding> {
    use gpui::KeyBinding as Key;
    const EDITOR: Option<&str> = Some("Editor");
    vec![
        Key::new("left", Left, EDITOR),
        Key::new("right", Right, EDITOR),
        Key::new("up", Up, EDITOR),
        Key::new("down", Down, EDITOR),
        Key::new("shift-left", SelectLeft, EDITOR),
        Key::new("shift-right", SelectRight, EDITOR),
        Key::new("shift-up", SelectUp, EDITOR),
        Key::new("shift-down", SelectDown, EDITOR),
        Key::new("alt-left", WordLeft, EDITOR),
        Key::new("alt-right", WordRight, EDITOR),
        Key::new("alt-shift-left", SelectWordLeft, EDITOR),
        Key::new("alt-shift-right", SelectWordRight, EDITOR),
        Key::new("home", Home, EDITOR),
        Key::new("end", End, EDITOR),
        Key::new("cmd-left", Home, EDITOR),
        Key::new("cmd-right", End, EDITOR),
        Key::new("shift-home", SelectHome, EDITOR),
        Key::new("shift-end", SelectEnd, EDITOR),
        Key::new("cmd-shift-left", SelectHome, EDITOR),
        Key::new("cmd-shift-right", SelectEnd, EDITOR),
        Key::new("backspace", Backspace, EDITOR),
        Key::new("delete", Delete, EDITOR),
        Key::new("alt-backspace", DeleteWordLeft, EDITOR),
        Key::new("alt-delete", DeleteWordRight, EDITOR),
        Key::new("cmd-backspace", DeleteToRowStart, EDITOR),
        Key::new("enter", Newline, EDITOR),
        Key::new("cmd-a", SelectAll, EDITOR),
        Key::new("cmd-c", Copy, EDITOR),
        Key::new("cmd-x", Cut, EDITOR),
        Key::new("cmd-v", Paste, EDITOR),
        Key::new("cmd-z", Undo, EDITOR),
        Key::new("cmd-shift-z", Redo, EDITOR),
    ]
}

/// The editor's palette. One per mode, and deliberately far apart: the change
/// of mode has to be felt at a glance (design.md, D6).
#[derive(Debug, Clone, Copy)]
pub struct EditorStyle {
    pub background: u32,
    pub ink: u32,
    /// Paragraphs above the cursor's, when `dim_above` is set.
    pub dim_ink: u32,
    /// Raw markers, on the cursor's line.
    pub marker_ink: u32,
    pub selection: u32,
    pub caret: u32,
    /// Fade what has already been written, so only the current fragment is at
    /// full strength (write-mode spec).
    pub dim_above: bool,
}

impl EditorStyle {
    pub fn write() -> Self {
        EditorStyle {
            background: theme::write::BACKGROUND,
            ink: theme::write::INK,
            dim_ink: theme::write::DIM_INK,
            marker_ink: theme::write::MARKER_INK,
            selection: theme::write::SELECTION,
            caret: theme::write::CARET,
            dim_above: true,
        }
    }

    /// Edit mode: daylight, and the whole text at full strength. Nothing fades,
    /// because everything on the page is now equally the subject (edit-mode
    /// spec).
    pub fn edit() -> Self {
        EditorStyle {
            background: theme::edit::BACKGROUND,
            ink: theme::edit::INK,
            dim_ink: theme::edit::INK,
            marker_ink: theme::edit::MARKER_INK,
            selection: theme::edit::SELECTION,
            caret: theme::edit::CARET,
            dim_above: false,
        }
    }
}

/// The editor changed the document. The screen above decides what that costs —
/// in Write mode, a save a second later.
pub struct Edited;

pub struct EditorView {
    focus_handle: FocusHandle,
    buffer: Buffer,
    style: EditorStyle,
    viewport: Viewport,
    /// Where a scrolled view sits in the document. Unused — and unmoved — in
    /// typewriter mode, where the caret decides the view every frame.
    scroll: Scroll,
    /// The caret has just moved and owes the view a look. Set by every edit and
    /// every cursor move, cleared once the frame has chased it; the wheel does
    /// not set it, which is what keeps scrolling from snapping back.
    follow_caret: bool,
    /// Range of text the IME is composing, in buffer bytes.
    marked_range: Option<Range<usize>>,
    /// Horizontal position vertical movement aims for, so passing a short row
    /// does not lose it. In pixels from the start of the row, because rows are
    /// proportional text, not columns.
    goal_x: Option<Pixels>,
    is_selecting: bool,
    /// What the last frame put on screen — the only way a key or a click can
    /// know where the wrapped rows ended up.
    last_paint: Option<PaintedFrame>,
}

impl EventEmitter<Edited> for EditorView {}

impl EditorView {
    pub fn new(
        text: impl Into<String>,
        style: EditorStyle,
        viewport: Viewport,
        cx: &mut Context<Self>,
    ) -> Self {
        EditorView {
            focus_handle: cx.focus_handle(),
            buffer: Buffer::new(text),
            style,
            viewport,
            scroll: Scroll::default(),
            follow_caret: true,
            marked_range: None,
            goal_x: None,
            is_selecting: false,
            last_paint: None,
        }
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    /// Bumped by every edit — what an autosave debounce watches.
    pub fn revision(&self) -> u64 {
        self.buffer.revision()
    }

    /// True while the IME is composing, when a key belongs to it and not to us.
    pub fn is_composing(&self) -> bool {
        self.marked_range.is_some()
    }

    fn edited(&mut self, cx: &mut Context<Self>) {
        self.goal_x = None;
        self.follow_caret = true;
        cx.emit(Edited);
        cx.notify();
    }

    fn moved(&mut self, cx: &mut Context<Self>) {
        self.goal_x = None;
        self.follow_caret = true;
        cx.notify();
    }

    // -- movement --------------------------------------------------------

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_left(false);
        self.moved(cx);
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_right(false);
        self.moved(cx);
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_left(true);
        self.moved(cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_right(true);
        self.moved(cx);
    }

    fn word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_word_left(false);
        self.moved(cx);
    }

    fn word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_word_right(false);
        self.moved(cx);
    }

    fn select_word_left(&mut self, _: &SelectWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_word_left(true);
        self.moved(cx);
    }

    fn select_word_right(&mut self, _: &SelectWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_word_right(true);
        self.moved(cx);
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.move_vertically(-1, false, cx);
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.move_vertically(1, false, cx);
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        self.move_vertically(-1, true, cx);
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        self.move_vertically(1, true, cx);
    }

    /// Up and down move by *visual* rows, keeping the caret's horizontal
    /// position. Both need the last frame's geometry; before the first frame
    /// there is none, and source lines are the honest approximation.
    fn move_vertically(&mut self, delta: isize, extend: bool, cx: &mut Context<Self>) {
        // Unlike the other moves, this one keeps `goal_x` when it has one, so
        // it does its own book-keeping rather than going through `moved`.
        self.follow_caret = true;

        let Some(step) = self.vertical_step(delta) else {
            if delta < 0 {
                self.buffer.move_up(extend);
            } else {
                self.buffer.move_down(extend);
            }
            self.goal_x = None;
            cx.notify();
            return;
        };

        match step {
            Step::To { offset, goal_x } => {
                self.buffer.move_head(offset, extend);
                self.goal_x = Some(goal_x);
            }
            Step::PastTheEdge => {
                let edge = if delta < 0 { 0 } else { self.buffer.text().len() };
                self.buffer.move_head(edge, extend);
            }
        }
        cx.notify();
    }

    fn vertical_step(&self, delta: isize) -> Option<Step> {
        let frame = self.last_paint.as_ref()?;
        let line_index = self.buffer.cursor_line();
        let line = frame.line(line_index)?;
        let line_start = self.buffer.line_range(line_index).start;
        let display = line
            .visual
            .display
            .display_offset(self.buffer.cursor() - line_start);
        let (row, _) = line.visual.position_of(self.buffer.cursor() - line_start);
        let goal_x = self.goal_x.unwrap_or_else(|| line.x_for(display));

        // One row up or down, crossing into the neighbouring paragraph when
        // this one runs out of rows. A neighbour the buffer has but the frame
        // does not is not a step — `None` sends the caller to the source-line
        // fallback rather than somewhere wrong.
        let (target_line, target_row) = if delta < 0 {
            if row > 0 {
                (line_index, row - 1)
            } else if line_index == 0 {
                return Some(Step::PastTheEdge);
            } else {
                let above = frame.line(line_index - 1)?;
                (line_index - 1, above.visual.row_count() - 1)
            }
        } else if row + 1 < line.visual.row_count() {
            (line_index, row + 1)
        } else if line_index + 1 >= self.buffer.line_count() {
            return Some(Step::PastTheEdge);
        } else {
            frame.line(line_index + 1)?;
            (line_index + 1, 0)
        };

        let target = frame.line(target_line)?;
        let offset =
            target.offset_at(goal_x, target_row) + self.buffer.line_range(target_line).start;
        Some(Step::To { offset, goal_x })
    }

    /// Home and End work on the visual row, not the paragraph: on a wrapped
    /// line they go to the ends of the row the caret is on.
    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.to_row_edge(false, false, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.to_row_edge(true, false, cx);
    }

    fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.to_row_edge(false, true, cx);
    }

    fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.to_row_edge(true, true, cx);
    }

    fn to_row_edge(&mut self, to_end: bool, extend: bool, cx: &mut Context<Self>) {
        match self.row_edge(to_end) {
            Some(offset) => self.buffer.move_head(offset, extend),
            None if to_end => self.buffer.move_to_line_end(extend),
            None => self.buffer.move_to_line_start(extend),
        }
        self.moved(cx);
    }

    fn row_edge(&self, to_end: bool) -> Option<usize> {
        let frame = self.last_paint.as_ref()?;
        let line_index = self.buffer.cursor_line();
        let line = frame.line(line_index)?;
        let line_start = self.buffer.line_range(line_index).start;
        let (row, _) = line.visual.position_of(self.buffer.cursor() - line_start);
        let column = if to_end { usize::MAX } else { 0 };
        Some(line.visual.offset_of(row, column) + line_start)
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.select_all();
        self.moved(cx);
    }

    // -- editing ---------------------------------------------------------

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.backspace();
        self.edited(cx);
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.delete_forward();
        self.edited(cx);
    }

    fn delete_word_left(&mut self, _: &DeleteWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.delete_word_left();
        self.edited(cx);
    }

    fn delete_word_right(&mut self, _: &DeleteWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.delete_word_right();
        self.edited(cx);
    }

    /// Command-backspace takes everything from the start of the visual row to
    /// the caret — the row Home goes to the start of, so the two keys agree
    /// about where a row begins.
    fn delete_to_row_start(
        &mut self,
        _: &DeleteToRowStart,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.buffer.selection().is_empty() {
            match self.row_edge(false) {
                Some(offset) => self.buffer.move_head(offset, true),
                None => self.buffer.move_to_line_start(true),
            }
            // The caret was already at the start of the row: what is above it
            // belongs to the row above, not to this one.
            if self.buffer.selection().is_empty() {
                return;
            }
        }
        self.buffer.backspace();
        self.edited(cx);
    }

    fn newline(&mut self, _: &Newline, _: &mut Window, cx: &mut Context<Self>) {
        // Mid-composition, Enter belongs to the IME.
        if self.is_composing() {
            return;
        }
        self.buffer.insert("\n");
        self.edited(cx);
    }

    fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.undo() {
            self.edited(cx);
        }
    }

    fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.redo() {
            self.edited(cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        let selected = self.buffer.selected_text();
        if !selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(selected.to_string()));
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        let taken = self.buffer.cut();
        if !taken.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(taken));
            self.edited(cx);
        }
    }

    // -- markup ----------------------------------------------------------
    //
    // The same five keys in both rooms: markup is part of writing, not a mode
    // of its own (design.md, D4). Each is an ordinary edit, so it saves, undoes
    // and renders through the paths everything else already uses.

    fn toggle_bold(&mut self, _: &ToggleBold, _: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.toggle_inline(Emphasis::Bold) {
            self.edited(cx);
        }
    }

    fn toggle_italic(&mut self, _: &ToggleItalic, _: &mut Window, cx: &mut Context<Self>) {
        if self.buffer.toggle_inline(Emphasis::Italic) {
            self.edited(cx);
        }
    }

    fn heading_1(&mut self, _: &Heading1, _: &mut Window, cx: &mut Context<Self>) {
        self.set_heading(1, cx);
    }

    fn heading_2(&mut self, _: &Heading2, _: &mut Window, cx: &mut Context<Self>) {
        self.set_heading(2, cx);
    }

    fn heading_3(&mut self, _: &Heading3, _: &mut Window, cx: &mut Context<Self>) {
        self.set_heading(3, cx);
    }

    fn set_heading(&mut self, level: u8, cx: &mut Context<Self>) {
        self.buffer.set_heading(level);
        self.edited(cx);
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            // Whatever the clipboard carries, the document only ever gains the
            // newlines that are in it — wrap adds none.
            self.buffer.insert(&text.replace("\r\n", "\n").replace('\r', "\n"));
            self.edited(cx);
        }
    }

    // -- mouse -----------------------------------------------------------

    fn offset_for_position(&self, position: gpui::Point<Pixels>) -> Option<usize> {
        let frame = self.last_paint.as_ref()?;
        let (line_index, display) = frame.hit_test(position)?;
        let line = frame.line(line_index)?;
        Some(line.visual.display.source_offset(display) + self.buffer.line_range(line_index).start)
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(offset) = self.offset_for_position(event.position) else {
            return;
        };
        self.is_selecting = true;
        self.buffer.move_head(offset, event.modifiers.shift);
        self.moved(cx);
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.is_selecting {
            return;
        }
        if let Some(offset) = self.offset_for_position(event.position) {
            self.buffer.move_head(offset, true);
            self.moved(cx);
        }
    }

    /// The wheel moves the view and nothing else: the cursor stays where it was
    /// in the text, and the caret is not chased back (edit-mode spec). Only
    /// reachable in scrolled mode — typewriter mode binds no handler.
    fn on_scroll_wheel(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let row = px(theme::EDITOR_SIZE * theme::EDITOR_LINE_SPACING);
        // A wheel that pushes the page up is the document going past the top
        // edge, which is what the offset counts.
        self.scroll.offset -= event.delta.pixel_delta(row).y;
        cx.notify();
    }

    // -- UTF-16 bridge ---------------------------------------------------
    //
    // The platform's input protocol counts UTF-16 units; the buffer stores
    // UTF-8. Cyrillic and IME composition both cross this boundary.

    fn offset_from_utf16(&self, target: usize) -> usize {
        let mut utf8 = 0;
        let mut utf16 = 0;
        for ch in self.buffer.text().chars() {
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
        for ch in self.buffer.text().chars() {
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

/// Where one press of up or down lands.
enum Step {
    To { offset: usize, goal_x: Pixels },
    /// Off the top or bottom of the document: go to the very start or end,
    /// like every other editor does.
    PastTheEdge,
}

impl Focusable for EditorView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// Typed text and IME composition both arrive through this trait.
impl EntityInputHandler for EditorView {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.buffer.text()[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let selection = self.buffer.selection();
        Some(UTF16Selection {
            range: self.range_to_utf16(&selection.range()),
            reversed: selection.head < selection.anchor,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
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
        let replace = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone());

        if let Some(range) = replace {
            self.buffer.set_cursor(range.start);
            self.buffer.move_head(range.end, true);
        }
        self.buffer.insert(new_text);
        self.marked_range = None;
        self.edited(cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let replace = range_utf16
            .as_ref()
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.buffer.selection().range());

        self.buffer.set_cursor(replace.start);
        self.buffer.move_head(replace.end, true);
        self.buffer.insert(new_text);

        let start = replace.start;
        self.marked_range = (!new_text.is_empty()).then(|| start..start + new_text.len());

        // The IME reports the caret inside the text it is composing.
        if let Some(selected) = new_selected_range_utf16 {
            let inside = self.range_from_utf16(&selected);
            self.buffer.set_cursor(start + inside.start);
            if inside.end > inside.start {
                self.buffer.move_head(start + inside.end, true);
            }
        }
        self.edited(cx);
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _element_bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        // Where the IME candidate window should appear.
        let frame = self.last_paint.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let line_index = self.buffer.line_at(range.start);
        let line_start = self.buffer.line_range(line_index).start;
        frame.bounds_for(
            line_index,
            range.start.saturating_sub(line_start),
            range.end.saturating_sub(line_start),
        )
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let offset = self.offset_for_position(point)?;
        Some(self.offset_to_utf16(offset))
    }
}

impl Render for EditorView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context(crate::keymap::EDITOR)
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .size_full()
            .bg(gpui::rgb(self.style.background))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::word_left))
            .on_action(cx.listener(Self::word_right))
            .on_action(cx.listener(Self::select_word_left))
            .on_action(cx.listener(Self::select_word_right))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_word_left))
            .on_action(cx.listener(Self::delete_word_right))
            .on_action(cx.listener(Self::delete_to_row_start))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::toggle_bold))
            .on_action(cx.listener(Self::toggle_italic))
            .on_action(cx.listener(Self::heading_1))
            .on_action(cx.listener(Self::heading_2))
            .on_action(cx.listener(Self::heading_3))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            // In typewriter mode there is no handler at all: the viewport is
            // derived from the cursor, so the wheel has nothing to move
            // (write-mode spec).
            .when(self.viewport == Viewport::Scrolled, |editor| {
                editor.on_scroll_wheel(cx.listener(Self::on_scroll_wheel))
            })
            .child(EditorElement {
                view: cx.entity(),
            })
    }
}

/// Font size for a line. Read from the source, not from the render plan, so a
/// heading keeps its size while the cursor is on it and shows its markers.
fn line_font_size(heading: Option<u8>) -> Pixels {
    match heading {
        Some(level) => px(theme::heading_size(level)),
        None => px(theme::EDITOR_SIZE),
    }
}
