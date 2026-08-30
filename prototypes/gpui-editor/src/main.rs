//! gpui prototype of the esse editor.
//!
//! Throwaway by declaration (see the change proposal). What it has to prove:
//! markdown-lite renders in place as you type, raw markers appear only on the
//! cursor line, typewriter mode keeps that line centred, and the baseline
//! editing operations — selection, clipboard, undo, Cyrillic and IME input —
//! all behave.
//!
//! Layout is done by hand rather than with gpui's element tree, because each
//! line needs its own shaped run list and its own font size.

use std::ops::Range;

use gpui::{
    actions, div, fill, point, prelude::*, px, relative, rgb, rgba, size, App, Bounds,
    ClipboardItem, ContentMask, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, FontStyle, FontWeight, GlobalElementId, Hsla,
    KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad,
    Pixels, Point, ScrollDelta, ScrollWheelEvent, ShapedLine, Style, TextAlign, TextRun,
    UTF16Selection, UnderlineStyle, Window, WindowBounds, WindowOptions,
};
use gpui_platform::application;

use gpui_editor::buffer::Buffer;
use gpui_editor::display::DisplayLine;
use markdown_lite::{parse_line, render_plan};

actions!(
    esse,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectAll,
        Home,
        End,
        SelectHome,
        SelectEnd,
        Newline,
        Undo,
        Redo,
        Copy,
        Cut,
        Paste,
        ToggleTypewriter,
        LoadSample,
        Quit,
    ]
);

// -- appearance ----------------------------------------------------------

const BACKGROUND: u32 = 0xfbfaf7;
const INK: u32 = 0x23211e;
const MARKER_INK: u32 = 0xb4aea3;
const SELECTION: u32 = 0x3b82f636;
const CARET: u32 = 0x2563eb;

const BODY_SIZE: f32 = 19.;
const LINE_SPACING: f32 = 1.65;
const PAGE_PADDING: f32 = 48.;
/// Width of the text column — an essay wants a measure, not the whole window.
const MEASURE: f32 = 680.;

fn heading_size(level: u8) -> f32 {
    match level {
        1 => 33.,
        2 => 27.,
        3 => 23.,
        _ => 21.,
    }
}

// -- view ----------------------------------------------------------------

struct EditorView {
    focus_handle: FocusHandle,
    buffer: Buffer,
    /// Range of text the IME is currently composing, in buffer bytes.
    marked_range: Option<Range<usize>>,
    typewriter: bool,
    /// Distance the content is scrolled up, in pixels.
    scroll: Pixels,
    /// Line the typewriter last centred on, so it only re-centres when the
    /// caret actually changes line.
    centred_line: Option<usize>,
    is_selecting: bool,
    /// What the last paint put on screen, for hit-testing mouse events.
    last_paint: Option<PaintedFrame>,
}

/// The geometry of the last painted frame — enough to turn a mouse position
/// back into a buffer offset.
struct PaintedFrame {
    bounds: Bounds<Pixels>,
    scroll: Pixels,
    lines: Vec<PaintedLine>,
}

struct PaintedLine {
    index: usize,
    display: DisplayLine,
    shaped: ShapedLine,
    /// Top of the line in content space (scroll not yet applied).
    top: Pixels,
    height: Pixels,
}

impl EditorView {
    fn new(cx: &mut Context<Self>) -> Self {
        EditorView {
            focus_handle: cx.focus_handle(),
            buffer: Buffer::new(WELCOME),
            marked_range: None,
            typewriter: true,
            scroll: px(0.),
            centred_line: None,
            is_selecting: false,
            last_paint: None,
        }
    }

    // -- movement --------------------------------------------------------

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_left(false);
        cx.notify();
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_right(false);
        cx.notify();
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_up(false);
        cx.notify();
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_down(false);
        cx.notify();
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_left(true);
        cx.notify();
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_right(true);
        cx.notify();
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_up(true);
        cx.notify();
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_down(true);
        cx.notify();
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_to_line_start(false);
        cx.notify();
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_to_line_end(false);
        cx.notify();
    }

    fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_to_line_start(true);
        cx.notify();
    }

    fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.move_to_line_end(true);
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.select_all();
        cx.notify();
    }

    // -- editing ---------------------------------------------------------

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.backspace();
        cx.notify();
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.delete_forward();
        cx.notify();
    }

    fn newline(&mut self, _: &Newline, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.insert("\n");
        cx.notify();
    }

    fn undo(&mut self, _: &Undo, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.undo();
        cx.notify();
    }

    fn redo(&mut self, _: &Redo, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer.redo();
        cx.notify();
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
            cx.notify();
        }
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.buffer.insert(&text);
            cx.notify();
        }
    }

    fn toggle_typewriter(&mut self, _: &ToggleTypewriter, _: &mut Window, cx: &mut Context<Self>) {
        self.typewriter = !self.typewriter;
        self.centred_line = None;
        cx.notify();
    }

    /// Load an essay-sized document, for the responsiveness check (task 3.8).
    fn load_sample(&mut self, _: &LoadSample, _: &mut Window, cx: &mut Context<Self>) {
        self.buffer = Buffer::new(sample_essay());
        self.marked_range = None;
        self.scroll = px(0.);
        self.centred_line = None;
        cx.notify();
    }

    // -- mouse -----------------------------------------------------------

    fn offset_for_position(&self, position: Point<Pixels>) -> Option<usize> {
        let frame = self.last_paint.as_ref()?;
        let content_y = position.y - frame.bounds.top() + frame.scroll;

        // Above or below everything painted: clamp to the ends.
        let first = frame.lines.first()?;
        let last = frame.lines.last()?;
        if content_y < first.top {
            return Some(self.buffer.line_range(first.index).start);
        }
        if content_y > last.top + last.height {
            return Some(self.buffer.line_range(last.index).end);
        }

        let line = frame
            .lines
            .iter()
            .find(|line| content_y >= line.top && content_y < line.top + line.height)
            .unwrap_or(last);

        let x = position.x - frame.bounds.left() - px(PAGE_PADDING);
        let display_offset = line.shaped.closest_index_for_x(x);
        let source = line.display.source_offset(display_offset);
        Some(self.buffer.line_range(line.index).start + source)
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let Some(offset) = self.offset_for_position(event.position) else { return };
        self.is_selecting = true;
        self.buffer.move_head(offset, event.modifiers.shift);
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
            self.buffer.move_head(offset, true);
            cx.notify();
        }
    }

    fn on_scroll(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let delta = match event.delta {
            ScrollDelta::Pixels(delta) => delta.y,
            ScrollDelta::Lines(delta) => px(delta.y * BODY_SIZE * LINE_SPACING),
        };
        // Not clamped here: typewriter centring legitimately produces a
        // negative offset. prepaint clamps when typewriter mode is off.
        self.scroll -= delta;
        cx.notify();
    }

    // -- UTF-16 bridge ---------------------------------------------------
    //
    // The platform's input protocol speaks UTF-16; the buffer speaks UTF-8.

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

impl Focusable for EditorView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// Typed text and IME composition both arrive through this trait — it is the
/// hard gate the spike has to clear (spec: Cyrillic and IME input).
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
        let frame = self.last_paint.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let line_index = self.buffer.line_at(range.start);
        let line = frame.lines.iter().find(|line| line.index == line_index)?;

        let line_start = self.buffer.line_range(line_index).start;
        let start_x = line
            .shaped
            .x_for_index(line.display.display_offset(range.start.saturating_sub(line_start)));
        let end_x = line
            .shaped
            .x_for_index(line.display.display_offset(range.end.saturating_sub(line_start)));
        let top = element_bounds.top() + line.top - frame.scroll;

        Some(Bounds::from_corners(
            point(element_bounds.left() + px(PAGE_PADDING) + start_x, top),
            point(
                element_bounds.left() + px(PAGE_PADDING) + end_x,
                top + line.height,
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
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
            .key_context("Editor")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .size_full()
            .bg(rgb(BACKGROUND))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::toggle_typewriter))
            .on_action(cx.listener(Self::load_sample))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .child(EditorElement { view: cx.entity() })
    }
}

// -- element -------------------------------------------------------------

struct EditorElement {
    view: Entity<EditorView>,
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

struct PrepaintState {
    lines: Vec<PaintedLine>,
    selections: Vec<PaintQuad>,
    caret: Option<PaintQuad>,
    scroll: Pixels,
    cursor_line: usize,
}

impl Element for EditorElement {
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
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let view = self.view.read(cx);
        let buffer = &view.buffer;
        let cursor_line = buffer.cursor_line();
        let text_style = window.text_style();

        // Heights come from a parse, which is cheap; only visible lines get
        // shaped, which is not. That keeps an essay-sized document affordable.
        let mut tops = Vec::with_capacity(buffer.line_count());
        let mut heights = Vec::with_capacity(buffer.line_count());
        let mut y = px(PAGE_PADDING);
        for index in 0..buffer.line_count() {
            let height = px(line_font_size(buffer.line(index)) * LINE_SPACING);
            tops.push(y);
            heights.push(height);
            y += height;
        }
        let content_height = y + px(PAGE_PADDING);

        // Typewriter mode re-centres when the caret changes line — on Enter, on
        // an arrow, on a click. Between those the stored offset stands, so the
        // wheel still works and the view does not fight the reader.
        let recentre = view.typewriter && view.centred_line != Some(cursor_line);
        let scroll = if recentre {
            tops[cursor_line] + heights[cursor_line] / 2. - bounds.size.height / 2.
        } else if view.typewriter {
            // Already centred; keep it, including the negative offset that
            // centring the first line needs.
            view.scroll
        } else {
            view.scroll
                .min((content_height - bounds.size.height).max(px(0.)))
                .max(px(0.))
        };

        let visible = |index: usize| {
            let top = tops[index] - scroll;
            top + heights[index] >= px(0.) && top <= bounds.size.height
        };

        let mut lines = Vec::new();
        for index in 0..buffer.line_count() {
            if !visible(index) {
                continue;
            }
            let text = buffer.line(index);
            let plan = render_plan(text, index == cursor_line);
            let display = DisplayLine::new(text, &plan);
            let font_size = px(line_font_size(text));

            let marked = view.marked_range.as_ref().and_then(|marked| {
                let line = buffer.line_range(index);
                (marked.start >= line.start && marked.end <= line.end).then(|| {
                    display.display_offset(marked.start - line.start)
                        ..display.display_offset(marked.end - line.start)
                })
            });

            let runs = build_runs(&display, &text_style, marked.as_ref());
            let shaped =
                window
                    .text_system()
                    .shape_line(display.text.clone().into(), font_size, &runs, None);

            lines.push(PaintedLine {
                index,
                display,
                shaped,
                top: tops[index],
                height: heights[index],
            });
        }

        // Selection highlight, one quad per line it covers.
        let selection = buffer.selection().range();
        let mut selections = Vec::new();
        if !selection.is_empty() {
            for line in &lines {
                let line_range = buffer.line_range(line.index);
                let start = selection.start.max(line_range.start);
                let end = selection.end.min(line_range.end);
                if start > end {
                    continue;
                }
                // A selection crossing a line end should show the newline as
                // selected, so full lines read as full bars.
                let covers_newline = selection.end > line_range.end;

                let from = line
                    .shaped
                    .x_for_index(line.display.display_offset(start - line_range.start));
                let to = line
                    .shaped
                    .x_for_index(line.display.display_offset(end - line_range.start));
                let to = if covers_newline { to + px(6.) } else { to };
                if to <= from && !covers_newline {
                    continue;
                }

                let top = bounds.top() + line.top - scroll;
                selections.push(fill(
                    Bounds::from_corners(
                        point(bounds.left() + px(PAGE_PADDING) + from, top),
                        point(bounds.left() + px(PAGE_PADDING) + to, top + line.height),
                    ),
                    rgba(SELECTION),
                ));
            }
        }

        // The caret sits on the cursor line, which always renders raw — so the
        // display offset is just the offset within the line.
        let caret = lines
            .iter()
            .find(|line| line.index == cursor_line)
            .map(|line| {
                let line_range = buffer.line_range(cursor_line);
                let offset = buffer.cursor().saturating_sub(line_range.start);
                let x = line.shaped.x_for_index(line.display.display_offset(offset));
                let top = bounds.top() + line.top - scroll;
                fill(
                    Bounds::new(
                        point(bounds.left() + px(PAGE_PADDING) + x, top),
                        size(px(2.), line.height),
                    ),
                    rgb(CARET),
                )
            });

        PrepaintState { lines, selections, caret, scroll, cursor_line }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.view.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );

        let scroll = prepaint.scroll;
        let cursor_line = prepaint.cursor_line;
        let lines = std::mem::take(&mut prepaint.lines);
        let selections = std::mem::take(&mut prepaint.selections);
        let caret = prepaint.caret.take();
        let focused = focus_handle.is_focused(window);

        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for quad in selections {
                window.paint_quad(quad);
            }
            for line in &lines {
                let origin = point(
                    bounds.left() + px(PAGE_PADDING),
                    bounds.top() + line.top - scroll,
                );
                if let Err(error) =
                    line.shaped
                        .paint(origin, line.height, TextAlign::Left, None, window, cx)
                {
                    eprintln!("failed to paint line {}: {error:?}", line.index);
                }
            }
            if focused {
                if let Some(caret) = caret {
                    window.paint_quad(caret);
                }
            }
        });

        self.view.update(cx, |view, _| {
            view.scroll = scroll;
            if view.typewriter {
                view.centred_line = Some(cursor_line);
            }
            view.last_paint = Some(PaintedFrame { bounds, scroll, lines });
        });
    }
}

/// Font size for a line: headings are larger, and stay larger while the cursor
/// is on them — otherwise entering a heading would make the page jump.
fn line_font_size(line: &str) -> f32 {
    match parse_line(line).heading_level() {
        Some(level) => heading_size(level),
        None => BODY_SIZE,
    }
}

/// Turn the display line into shaper runs: bold and italic per segment, an
/// underline under whatever the IME is composing.
fn build_runs(
    display: &DisplayLine,
    text_style: &gpui::TextStyle,
    marked: Option<&Range<usize>>,
) -> Vec<TextRun> {
    let mut boundaries = vec![0, display.text.len()];
    for segment in &display.segments {
        boundaries.push(segment.display.start);
        boundaries.push(segment.display.end);
    }
    if let Some(marked) = marked {
        boundaries.push(marked.start);
        boundaries.push(marked.end);
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    let heading = display.heading.is_some();
    let ink: Hsla = rgb(INK).into();
    let marker_ink: Hsla = rgb(MARKER_INK).into();

    let mut runs = Vec::new();
    for pair in boundaries.windows(2) {
        let (start, end) = (pair[0], pair[1]);
        if start >= end {
            continue;
        }
        let style = display.style_at(start);
        let mut font = text_style.font();
        font.weight = if style.bold || heading {
            FontWeight::BOLD
        } else {
            FontWeight::NORMAL
        };
        font.style = if style.italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };

        // On the raw cursor line the markers are visible; grey them back so
        // the eye still reads the prose rather than the punctuation.
        let is_marker = display.segments.is_empty()
            || !display
                .segments
                .iter()
                .any(|segment| segment.display.contains(&start));

        runs.push(TextRun {
            len: end - start,
            font,
            color: if is_marker { marker_ink } else { ink },
            background_color: None,
            underline: marked
                .filter(|marked| start >= marked.start && end <= marked.end)
                .map(|_| UnderlineStyle {
                    color: Some(ink),
                    thickness: px(1.),
                    wavy: false,
                }),
            strikethrough: None,
        });
    }
    runs
}

// -- content -------------------------------------------------------------

const WELCOME: &str = "\
# esse — прототип редактора

Это *живой* markdown: разметка **рисуется на месте**, а сырые символы
видны только на той строке, где стоит курсор.

## Что проверяем

Наберите здесь абзац по-русски, попробуйте IME, выделение, копирование
и отмену. Строка с курсором держится по центру экрана.

Прочая разметка остаётся текстом: - список, [ссылка](url), `код`.

cmd-shift-t — типографская центровка, cmd-shift-l — эссе на 20 000 знаков.
";

/// ~20,000 characters of plausible prose, for the responsiveness check.
fn sample_essay() -> String {
    let paragraphs = [
        "Письмо начинается не с идеи, а с готовности сесть за стол и написать \
         первую **плохую** строчку, за которой прячется вторая.",
        "Черновик существует, чтобы быть *переписанным*: его задача — вытащить \
         мысль наружу, а не понравиться читателю с первого раза.",
        "Правка — отдельная работа, и смешивать её с письмом значит не сделать \
         толком ни того, ни другого.",
        "Ограничение в одно эссе за раз бьёт по привычке начинать новое, как \
         только старое становится трудным.",
    ];

    let mut essay = String::with_capacity(21_000);
    essay.push_str("# Эссе для проверки отзывчивости\n\n");
    let mut index = 0;
    while essay.chars().count() < 20_000 {
        if index % 8 == 0 {
            essay.push_str(&format!("## Раздел {}\n\n", index / 8 + 1));
        }
        essay.push_str(paragraphs[index % paragraphs.len()]);
        essay.push_str("\n\n");
        index += 1;
    }
    essay
}

// -- app -----------------------------------------------------------------

fn main() {
    env_logger::init();
    application().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("left", Left, Some("Editor")),
            KeyBinding::new("right", Right, Some("Editor")),
            KeyBinding::new("up", Up, Some("Editor")),
            KeyBinding::new("down", Down, Some("Editor")),
            KeyBinding::new("shift-left", SelectLeft, Some("Editor")),
            KeyBinding::new("shift-right", SelectRight, Some("Editor")),
            KeyBinding::new("shift-up", SelectUp, Some("Editor")),
            KeyBinding::new("shift-down", SelectDown, Some("Editor")),
            KeyBinding::new("home", Home, Some("Editor")),
            KeyBinding::new("end", End, Some("Editor")),
            KeyBinding::new("cmd-left", Home, Some("Editor")),
            KeyBinding::new("cmd-right", End, Some("Editor")),
            KeyBinding::new("shift-home", SelectHome, Some("Editor")),
            KeyBinding::new("shift-end", SelectEnd, Some("Editor")),
            KeyBinding::new("backspace", Backspace, Some("Editor")),
            KeyBinding::new("delete", Delete, Some("Editor")),
            KeyBinding::new("enter", Newline, Some("Editor")),
            KeyBinding::new("cmd-a", SelectAll, Some("Editor")),
            KeyBinding::new("cmd-c", Copy, Some("Editor")),
            KeyBinding::new("cmd-x", Cut, Some("Editor")),
            KeyBinding::new("cmd-v", Paste, Some("Editor")),
            KeyBinding::new("cmd-z", Undo, Some("Editor")),
            KeyBinding::new("cmd-shift-z", Redo, Some("Editor")),
            KeyBinding::new("cmd-shift-t", ToggleTypewriter, Some("Editor")),
            KeyBinding::new("cmd-shift-l", LoadSample, Some("Editor")),
            KeyBinding::new("cmd-q", Quit, None),
        ]);
        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());

        let bounds = Bounds::centered(None, size(px(MEASURE + PAGE_PADDING * 2.), px(760.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(EditorView::new),
            )
            .unwrap();

        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx), cx);
                cx.activate(true);
            })
            .unwrap();
    });
}
