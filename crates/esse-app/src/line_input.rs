//! The app's one plain line of text: the capture line on Today, and the
//! publication link in the completion overlay.
//!
//! A hand-built single-line field, not a small editor: text, a caret, IME
//! composition, paste and Enter, and deliberately nothing else. Selection and
//! undo belong to the real editor; a box you type one line into does not need
//! them, and each of them added here would have to be written twice. Paste is
//! the exception — a publication link arrives from the clipboard far more often
//! than it is typed out.

use std::ops::Range;

use gpui::{
    actions, div, fill, point, prelude::*, px, relative, rgb, size, App, Bounds, Context,
    CursorStyle, Element, ElementId, ElementInputHandler, Entity, EntityInputHandler, EventEmitter,
    FocusHandle, Focusable, FontStyle, GlobalElementId, InspectorElementId, LayoutId, PaintQuad,
    Pixels, ShapedLine, SharedString, Style, TextAlign, TextRun, UTF16Selection, UnderlineStyle,
    Window,
};

use crate::theme;

actions!(
    line_input,
    [Backspace, Delete, Left, Right, Home, End, Paste, Submit]
);

/// The field's baseline: a caret, deletion, paste, and Enter. Like the editor's
/// own set these are platform conventions rather than vocabulary this app
/// invented, so they live here rather than in the keymap table and help does
/// not list them (design.md, D1).
pub fn key_bindings() -> Vec<gpui::KeyBinding> {
    use gpui::KeyBinding as Key;
    const INPUT: Option<&str> = Some(crate::keymap::LINE_INPUT);
    vec![
        Key::new("enter", Submit, INPUT),
        Key::new("backspace", Backspace, INPUT),
        Key::new("delete", Delete, INPUT),
        Key::new("left", Left, INPUT),
        Key::new("right", Right, INPUT),
        Key::new("home", Home, INPUT),
        Key::new("end", End, INPUT),
        Key::new("cmd-left", Home, INPUT),
        Key::new("cmd-right", End, INPUT),
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
    /// The caret, as a byte offset into `text`.
    cursor: usize,
}

impl Line {
    fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    /// Deletes the character before the caret. False when there is none.
    fn backspace(&mut self) -> bool {
        let Some(previous) = self.previous_boundary() else {
            return false;
        };
        self.text.replace_range(previous..self.cursor, "");
        self.cursor = previous;
        true
    }

    /// Deletes the character after the caret. False when there is none.
    fn delete(&mut self) -> bool {
        let Some(next) = self.next_boundary() else {
            return false;
        };
        self.text.replace_range(self.cursor..next, "");
        true
    }

    fn move_left(&mut self) -> bool {
        match self.previous_boundary() {
            Some(previous) => {
                self.cursor = previous;
                true
            }
            None => false,
        }
    }

    fn move_right(&mut self) -> bool {
        match self.next_boundary() {
            Some(next) => {
                self.cursor = next;
                true
            }
            None => false,
        }
    }

    fn previous_boundary(&self) -> Option<usize> {
        self.text[..self.cursor]
            .chars()
            .next_back()
            .map(|ch| self.cursor - ch.len_utf8())
    }

    fn next_boundary(&self) -> Option<usize> {
        self.text[self.cursor..]
            .chars()
            .next()
            .map(|ch| self.cursor + ch.len_utf8())
    }

    /// Replaces a byte range with new text and leaves the caret after it. This
    /// is one line, so anything the platform hands over is flattened.
    fn splice(&mut self, range: Range<usize>, new_text: &str) {
        let flattened = if new_text.contains(['\n', '\r']) {
            new_text.replace(['\n', '\r'], " ")
        } else {
            new_text.to_string()
        };
        self.text.replace_range(range.clone(), &flattened);
        self.cursor = range.start + flattened.len();
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
    /// The last painted frame, for placing the IME candidate window.
    last_bounds: Option<Bounds<Pixels>>,
    last_line: Option<ShapedLine>,
}

impl EventEmitter<Submitted> for LineInput {}

impl LineInput {
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        LineInput {
            focus_handle: cx.focus_handle(),
            placeholder: placeholder.into(),
            line: Line::default(),
            marked_range: None,
            last_bounds: None,
            last_line: None,
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

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_left() {
            cx.notify();
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.line.move_right() {
            cx.notify();
        }
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.line.cursor = 0;
        cx.notify();
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.line.cursor = self.line.text.len();
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
        self.line.splice(self.line.cursor..self.line.cursor, &text);
        cx.notify();
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
        // There is no selection in this field; the caret is an empty one.
        let cursor = self.line.offset_to_utf16(self.line.cursor);
        Some(UTF16Selection {
            range: cursor..cursor,
            reversed: false,
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
            .unwrap_or(self.line.cursor..self.line.cursor);

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
            .unwrap_or(self.line.cursor..self.line.cursor);

        let start = range.start;
        self.line.splice(range, new_text);
        self.marked_range = (!new_text.is_empty()).then_some(start..self.line.cursor);

        // The IME reports where the caret sits inside the text it is composing.
        if let Some(selected) = new_selected_range_utf16 {
            self.line.cursor = start + self.line.range_from_utf16(&selected).start;
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
        // Where the IME candidate window should appear.
        let line = self.last_line.as_ref()?;
        let range = self.line.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                element_bounds.left() + line.x_for_index(range.start),
                element_bounds.top(),
            ),
            point(
                element_bounds.left() + line.x_for_index(range.end),
                element_bounds.bottom(),
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
        let line = self.last_line.as_ref()?;
        let index = line.closest_index_for_x(point.x - bounds.left());
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
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::paste))
            .child(LineInputElement { input: cx.entity() })
    }
}

// -- element -------------------------------------------------------------
//
// The line is painted by hand rather than with a text element: the caret and
// the IME underline need the shaped line's own geometry.

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
    line: Option<ShapedLine>,
    caret: PaintQuad,
    /// The line is the placeholder, so it must not be kept for hit-testing.
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
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = px(theme::INPUT_SIZE * theme::LINE_SPACING).into();
        (window.request_layout(style, [], cx), ())
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
        let mut font = window.text_style().font();
        let font_size = px(theme::INPUT_SIZE);
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

        let mut runs = Vec::new();
        let color = if placeholder {
            rgb(theme::MUTED).into()
        } else {
            rgb(theme::INK).into()
        };
        // Text the IME is still composing is underlined, so it reads as
        // provisional.
        let marked = input.marked_range.clone().filter(|_| !placeholder);
        for (range, underlined) in split_at(text.len(), marked) {
            runs.push(TextRun {
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
            });
        }

        let line = window
            .text_system()
            .shape_line(text, font_size, &runs, None);

        let caret_x = if placeholder {
            px(0.)
        } else {
            line.x_for_index(input.line.cursor)
        };
        let caret = fill(
            Bounds::new(
                point(bounds.left() + caret_x, bounds.top()),
                size(px(2.), bounds.size.height),
            ),
            rgb(theme::CARET),
        );

        PrepaintState {
            line: Some(line),
            caret,
            placeholder,
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

        let Some(line) = prepaint.line.take() else {
            return;
        };
        if let Err(error) = line.paint(
            bounds.origin,
            bounds.size.height,
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
            input.last_line = (!placeholder).then_some(line);
        });
    }
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
        Line {
            text: text.to_string(),
            cursor: text.len(),
        }
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
        assert!(line.move_left());
        assert_eq!(line.cursor, 2);
        assert!(line.move_left());
        assert_eq!(line.cursor, 0);
        assert!(!line.move_left());

        assert!(line.move_right());
        assert_eq!(line.cursor, 2);
        line.cursor = line.text.len();
        assert!(!line.move_right());
    }

    #[test]
    fn delete_removes_the_character_after_the_caret() {
        let mut line = line("мысль");
        line.cursor = 0;
        assert!(line.delete());

        assert_eq!(line.text, "ысль");
        assert_eq!(line.cursor, 0);
    }

    #[test]
    fn splicing_leaves_the_caret_after_the_new_text() {
        let mut line = line("иса");
        // Four bytes in: after "ис", the third character.
        line.cursor = 4;
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
