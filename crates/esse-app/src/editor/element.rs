//! Shaping, geometry and painting.
//!
//! Layout is anchored on one paragraph rather than on the document: it is
//! placed at a known y, and neighbours are laid out outwards from it until the
//! viewport is full. Nothing above or below is measured, let alone shaped —
//! which is the reason a 20,000-character essay costs the same per keystroke as
//! an empty one (design.md, D7; tasks 2.7, 2.8).
//!
//! Where the anchor comes from is the [`Viewport`] mode's business: the caret,
//! in Write mode, or the scroll position, in Edit mode (design.md, D5).

use std::ops::Range;

use gpui::{
    fill, point, prelude::*, px, rgb, rgba, size, App, Bounds, ContentMask, Element, ElementId,
    ElementInputHandler, Entity, FontStyle, FontWeight, GlobalElementId, Hsla, InspectorElementId,
    LayoutId, PaintQuad, Pixels, Point, ShapedGlyph, Style, TextAlign, TextRun, TextStyle,
    UnderlineStyle, Window, WrappedLine,
};
use markdown_lite::{parse_line, render_plan, SpanKind, Style as MarkdownStyle};

use super::buffer::Buffer;
use super::display::DisplayLine;
use super::viewport::{clamp_shift, shift_into_view, Scroll, Viewport};
use super::wrap::VisualLine;
use super::{line_font_size, EditorStyle, EditorView};
use crate::theme;

/// One paragraph, shaped and placed.
pub(super) struct PaintedLine {
    pub index: usize,
    pub visual: VisualLine,
    wrapped: WrappedLine,
    /// Top of the paragraph's first row, in window coordinates.
    top: Pixels,
    line_height: Pixels,
}

impl PaintedLine {
    fn height(&self) -> Pixels {
        self.line_height * self.visual.row_count() as f32
    }

    fn bottom(&self) -> Pixels {
        self.top + self.height()
    }

    /// Top of the visual row a source offset in this paragraph is drawn on.
    fn row_top(&self, offset: usize) -> Pixels {
        let (row, _) = self.visual.position_of(offset);
        self.top + self.line_height * row as f32
    }

    /// Where a display offset is drawn, measured from the left edge of the
    /// text column — that is, with the row's own start subtracted off.
    pub fn x_for(&self, display: usize) -> Pixels {
        self.x_in_row(display, self.visual.row_at(display))
    }

    /// The same, for an offset already known to belong to `row`. The end of a
    /// wrapped row *is* the start of the next one, so an offset there has to be
    /// measured against the row that is being drawn, not the row it opens.
    fn x_in_row(&self, display: usize, row: usize) -> Pixels {
        let layout = &self.wrapped.unwrapped_layout;
        layout.x_for_index(display) - layout.x_for_index(self.visual.row_start(row))
    }

    /// The display offset closest to `x` on a row — how a click and a vertical
    /// move both land. Clamped to the row, so overshooting to the right stops
    /// at its end rather than wrapping on to the next one.
    pub fn offset_at(&self, x: Pixels, row: usize) -> usize {
        let range = self.visual.row_range(row);
        let layout = &self.wrapped.unwrapped_layout;
        let row_x = layout.x_for_index(range.start);
        let display = layout
            .closest_index_for_x(x + row_x)
            .clamp(range.start, range.end);
        self.visual.display.source_offset(display)
    }

    fn row_at_y(&self, y: Pixels) -> usize {
        let row = ((y - self.top) / self.line_height).floor().max(0.) as usize;
        row.min(self.visual.row_count() - 1)
    }
}

/// The geometry of the last painted frame — enough to turn a mouse position or
/// an arrow key back into a buffer offset.
pub(super) struct PaintedFrame {
    /// Left edge of the text column.
    left: Pixels,
    lines: Vec<PaintedLine>,
}

impl PaintedFrame {
    pub fn line(&self, index: usize) -> Option<&PaintedLine> {
        self.lines.iter().find(|line| line.index == index)
    }

    /// The paragraph and display offset under a point. Clamped to what was
    /// painted: clicking above the first line lands at its start.
    pub fn hit_test(&self, position: Point<Pixels>) -> Option<(usize, usize)> {
        let first = self.lines.first()?;
        let last = self.lines.last()?;
        if position.y < first.top {
            return Some((first.index, 0));
        }
        if position.y >= last.top + last.height() {
            return Some((last.index, last.visual.display.text.len()));
        }

        let line = self
            .lines
            .iter()
            .find(|line| position.y >= line.top && position.y < line.top + line.height())
            .unwrap_or(last);
        let row = line.row_at_y(position.y);
        let range = line.visual.row_range(row);
        let layout = &line.wrapped.unwrapped_layout;
        let row_x = layout.x_for_index(range.start);
        let display = layout
            .closest_index_for_x(position.x - self.left + row_x)
            .clamp(range.start, range.end);
        Some((line.index, display))
    }

    /// Screen bounds of a range within one paragraph, for the IME's candidate
    /// window. Only the start's row is used; a composition never wraps far.
    pub fn bounds_for(&self, index: usize, start: usize, end: usize) -> Option<Bounds<Pixels>> {
        let line = self.line(index)?;
        let from = line.visual.display.display_offset(start);
        let to = line.visual.display.display_offset(end);
        let row = line.visual.row_at(from);
        // A composition that ran on to the next row is reported on this one,
        // ending where this one does; the panel only needs somewhere to sit.
        let range = line.visual.row_range(row);
        let top = line.top + line.line_height * row as f32;
        Some(Bounds::from_corners(
            point(self.left + line.x_in_row(from, row), top),
            point(
                self.left + line.x_in_row(to.min(range.end), row),
                top + line.line_height,
            ),
        ))
    }
}

pub(super) struct EditorElement {
    pub view: Entity<EditorView>,
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub(super) struct PrepaintState {
    lines: Vec<PaintedLine>,
    left: Pixels,
    selections: Vec<PaintQuad>,
    caret: Option<PaintQuad>,
    /// Where the scroll ended up, for the view to keep. `None` in typewriter
    /// mode, which has no position to keep.
    scroll: Option<Scroll>,
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
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = gpui::relative(1.).into();
        style.size.height = gpui::relative(1.).into();
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
        let view = self.view.read(cx);
        let buffer = &view.buffer;
        let style = view.style;
        let text_style = window.text_style();
        let cursor_line = buffer.cursor_line();

        // The text column: a measure, centred, never wider than the window.
        let measure = px(theme::EDITOR_MEASURE)
            .min((bounds.size.width - px(theme::EDITOR_PADDING * 2.)).max(px(120.)));
        let left = bounds.left() + (bounds.size.width - measure) / 2.;

        let shape = |index: usize, window: &Window| {
            shape_paragraph(
                buffer,
                index,
                cursor_line,
                &style,
                &text_style,
                view.marked_range.as_ref(),
                measure,
                window,
            )
        };

        let (lines, scroll) = match view.viewport {
            Viewport::Typewriter => (typewriter(buffer, bounds, &shape, window), None),
            Viewport::Scrolled => {
                let scroll = view.scroll;
                let follow = view.follow_caret;
                let (lines, scroll) = scrolled(buffer, scroll, follow, bounds, &shape, window);
                (lines, Some(scroll))
            }
        };

        let frame = PaintedFrame { left, lines };
        let selections = selection_quads(buffer, &frame, &style);
        let caret = caret_quad(buffer, &frame, &style);

        PrepaintState {
            lines: frame.lines,
            left,
            selections,
            caret,
            scroll,
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
        let focus_handle = self.view.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );

        let left = prepaint.left;
        let lines = std::mem::take(&mut prepaint.lines);
        let selections = std::mem::take(&mut prepaint.selections);
        let caret = prepaint.caret.take();
        let focused = focus_handle.is_focused(window);

        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for quad in selections {
                window.paint_quad(quad);
            }
            for line in &lines {
                if let Err(error) = line.wrapped.paint(
                    point(left, line.top),
                    line.line_height,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                ) {
                    log::error!("could not paint line {}: {error:?}", line.index);
                }
            }
            if focused {
                if let Some(caret) = caret {
                    window.paint_quad(caret);
                }
            }
        });

        let scroll = prepaint.scroll;
        self.view.update(cx, |view, _| {
            view.last_paint = Some(PaintedFrame { left, lines });
            // The caret has been chased, and the scroll clamped to what the
            // document actually allows. Neither notifies: this is the frame
            // that is already on screen catching the view up, not a change.
            if let Some(scroll) = scroll {
                view.scroll = scroll;
            }
            view.follow_caret = false;
        });
    }
}

/// Lay the document out from an anchor: paragraph `anchor`'s first row starts
/// at `top`, and neighbours are filled outwards until the viewport is covered.
/// Nothing past it is shaped — the whole point of anchored layout.
fn lay_out(
    buffer: &Buffer,
    anchor: usize,
    top: Pixels,
    bounds: Bounds<Pixels>,
    shape: &impl Fn(usize, &Window) -> (VisualLine, WrappedLine, Pixels),
    window: &Window,
) -> Vec<PaintedLine> {
    let (visual, wrapped, line_height) = shape(anchor, window);
    let mut lines = vec![PaintedLine {
        index: anchor,
        visual,
        wrapped,
        top,
        line_height,
    }];

    // One row of slack past each edge, so vertical movement always has a
    // neighbour to land on.
    let slack = px(theme::EDITOR_SIZE * theme::EDITOR_LINE_SPACING);
    let mut above = top;
    let mut index = anchor;
    while index > 0 && above > bounds.top() - slack {
        index -= 1;
        let (visual, wrapped, line_height) = shape(index, window);
        above -= line_height * visual.row_count() as f32;
        lines.push(PaintedLine {
            index,
            visual,
            wrapped,
            top: above,
            line_height,
        });
    }
    lines.reverse();

    // After the reverse the anchor is last; the rest go under it.
    let mut below = lines[lines.len() - 1].bottom();
    let mut index = anchor + 1;
    while index < buffer.line_count() && below < bounds.bottom() + slack {
        let (visual, wrapped, line_height) = shape(index, window);
        let height = line_height * visual.row_count() as f32;
        lines.push(PaintedLine {
            index,
            visual,
            wrapped,
            top: below,
            line_height,
        });
        below += height;
        index += 1;
    }
    lines
}

/// Write mode's viewport: the caret's visual row is centred, and the document
/// hangs from it. There is nothing to remember between frames — the anchor is
/// derived afresh, which is what makes the wheel inert (write-mode spec).
fn typewriter(
    buffer: &Buffer,
    bounds: Bounds<Pixels>,
    shape: &impl Fn(usize, &Window) -> (VisualLine, WrappedLine, Pixels),
    window: &Window,
) -> Vec<PaintedLine> {
    let cursor_line = buffer.cursor_line();
    let (visual, _, line_height) = shape(cursor_line, window);
    let line_start = buffer.line_range(cursor_line).start;
    let (row, _) = visual.position_of(buffer.cursor() - line_start);

    let top = bounds.top() + bounds.size.height / 2. - line_height / 2. - line_height * row as f32;
    lay_out(buffer, cursor_line, top, bounds, shape, window)
}

/// Edit mode's viewport: the anchor is wherever the scroll left it. The caret
/// is chased only when it has just moved, and the document is never let float
/// off either end (edit-mode spec). Returns the position to keep for the next
/// frame, which is the one the corrections below landed on.
fn scrolled(
    buffer: &Buffer,
    scroll: Scroll,
    follow_caret: bool,
    bounds: Bounds<Pixels>,
    shape: &impl Fn(usize, &Window) -> (VisualLine, WrappedLine, Pixels),
    window: &Window,
) -> (Vec<PaintedLine>, Scroll) {
    let (mut anchor, offset) = normalise(buffer, scroll, shape, window);
    let mut top = bounds.top() - offset;
    let mut lines = lay_out(buffer, anchor, top, bounds, shape, window);

    if follow_caret {
        if let Some((line, caret_top)) = caret_anchor(buffer, &lines, bounds, shape, window) {
            anchor = line;
            top = caret_top;
            lines = lay_out(buffer, anchor, top, bounds, shape, window);
        }
    }

    // `lay_out` stops filling only at an edge of the document, so a frame that
    // opens on paragraph 0 or ends on the last one is a frame that has reached
    // that end.
    let first = &lines[0];
    let last = &lines[lines.len() - 1];
    let shift = clamp_shift(
        first.index == 0,
        first.top,
        last.index + 1 == buffer.line_count(),
        last.bottom(),
        bounds.top(),
        bounds.bottom(),
    );
    if shift != px(0.) {
        top += shift;
        lines = lay_out(buffer, anchor, top, bounds, shape, window);
    }

    (
        lines,
        Scroll {
            line: anchor,
            offset: bounds.top() - top,
        },
    )
}

/// Walk the anchor to the paragraph the offset actually lands in, so layout
/// starts at the window's top edge and not somewhere off it. A wheel tick moves
/// the view by less than a paragraph, so this almost always walks nowhere.
fn normalise(
    buffer: &Buffer,
    scroll: Scroll,
    shape: &impl Fn(usize, &Window) -> (VisualLine, WrappedLine, Pixels),
    window: &Window,
) -> (usize, Pixels) {
    let height = |index: usize| {
        let (visual, _, line_height) = shape(index, window);
        line_height * visual.row_count() as f32
    };

    let last = buffer.line_count().saturating_sub(1);
    let mut line = scroll.line.min(last);
    let mut offset = scroll.offset;

    while offset < px(0.) && line > 0 {
        line -= 1;
        offset += height(line);
    }
    while line < last {
        let paragraph = height(line);
        if offset < paragraph {
            break;
        }
        offset -= paragraph;
        line += 1;
    }
    (line, offset.max(px(0.)))
}

/// Where the anchor has to move for the caret to be on screen, or `None` if it
/// already is.
fn caret_anchor(
    buffer: &Buffer,
    lines: &[PaintedLine],
    bounds: Bounds<Pixels>,
    shape: &impl Fn(usize, &Window) -> (VisualLine, WrappedLine, Pixels),
    window: &Window,
) -> Option<(usize, Pixels)> {
    let cursor_line = buffer.cursor_line();
    let offset = buffer.cursor() - buffer.line_range(cursor_line).start;

    if let Some(line) = lines.iter().find(|line| line.index == cursor_line) {
        // On screen already, or a row or so off it: shift the whole frame by
        // the least that brings the caret's row back.
        let top = line.row_top(offset);
        let shift = shift_into_view(top, top + line.line_height, bounds.top(), bounds.bottom());
        return (shift != px(0.)).then(|| (lines[0].index, lines[0].top + shift));
    }

    // Far enough away that the frame does not hold it — a click on the Today
    // screen and back, or Home in a long essay. Its paragraph becomes the
    // anchor, placed so the caret's row rests against the edge it came from.
    let (visual, _, line_height) = shape(cursor_line, window);
    let (row, _) = visual.position_of(offset);
    let top = if cursor_line < lines[0].index {
        bounds.top() - line_height * row as f32
    } else {
        bounds.bottom() - line_height * (row + 1) as f32
    };
    Some((cursor_line, top))
}

/// Shape one paragraph: parse it, resolve its markers, hand it to the shaper
/// with a wrap width, and record where the shaper broke it.
#[allow(clippy::too_many_arguments)]
fn shape_paragraph(
    buffer: &Buffer,
    index: usize,
    cursor_line: usize,
    style: &EditorStyle,
    text_style: &TextStyle,
    marked: Option<&Range<usize>>,
    wrap_width: Pixels,
    window: &Window,
) -> (VisualLine, WrappedLine, Pixels) {
    let text = buffer.line(index);
    let raw = index == cursor_line;
    let heading = parse_line(text).heading_level();
    let font_size = line_font_size(heading);
    let line_height = font_size * theme::EDITOR_LINE_SPACING;

    let display = DisplayLine::new(text, &render_plan(text, raw));

    // The composed text is underlined so it reads as provisional. Its range is
    // in buffer bytes; only the part inside this paragraph matters.
    let line_range = buffer.line_range(index);
    let marked = marked.and_then(|marked| {
        (marked.start >= line_range.start && marked.end <= line_range.end).then(|| {
            display.display_offset(marked.start - line_range.start)
                ..display.display_offset(marked.end - line_range.start)
        })
    });

    // Only what is already written fades; the paragraph being written and
    // anything after it stay at full strength (write-mode spec).
    let dimmed = style.dim_above && index < cursor_line;
    let runs = build_runs(
        text,
        &display,
        raw,
        heading.is_some(),
        dimmed,
        marked.as_ref(),
        style,
        text_style,
    );

    let mut shaped = window
        .text_system()
        .shape_text(
            display.text.clone().into(),
            font_size,
            &runs,
            Some(wrap_width),
            None,
        )
        .unwrap_or_default();
    // A paragraph holds no newline, so the shaper returns exactly one line.
    let wrapped = if shaped.is_empty() {
        WrappedLine::default()
    } else {
        shaped.remove(0)
    };

    let boundaries = boundary_offsets(&wrapped);
    (VisualLine::new(display, boundaries), wrapped, line_height)
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

/// Turn a paragraph into shaper runs: emphasis per span, grey for the raw
/// markers, one ink for the whole line when it is dimmed.
#[allow(clippy::too_many_arguments)]
fn build_runs(
    text: &str,
    display: &DisplayLine,
    raw: bool,
    heading: bool,
    dimmed: bool,
    marked: Option<&Range<usize>>,
    style: &EditorStyle,
    text_style: &TextStyle,
) -> Vec<TextRun> {
    // (display range, emphasis, is a syntax marker)
    let mut spans: Vec<(Range<usize>, MarkdownStyle, bool)> = if raw {
        // The cursor's line is drawn as typed, so display offsets are source
        // offsets — the parse can be used directly, markers and all.
        parse_line(text)
            .spans
            .into_iter()
            .map(|span| match span.kind {
                SpanKind::Text(emphasis) => (span.range, emphasis, false),
                SpanKind::Marker => (span.range, MarkdownStyle::PLAIN, true),
            })
            .collect()
    } else {
        display
            .segments
            .iter()
            .map(|segment| (segment.display.clone(), segment.style, false))
            .collect()
    };
    // The IME underline can start and end inside a span, so cut them apart.
    if let Some(marked) = marked {
        spans = split_spans(spans, marked);
    }

    let ink: Hsla = rgb(if dimmed { style.dim_ink } else { style.ink }).into();
    let marker_ink: Hsla = rgb(if dimmed { style.dim_ink } else { style.marker_ink }).into();

    spans
        .into_iter()
        .filter(|(range, _, _)| range.start < range.end)
        .map(|(range, emphasis, marker)| {
            let mut font = text_style.font();
            font.weight = if emphasis.bold || heading {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            };
            font.style = if emphasis.italic {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            };

            TextRun {
                len: range.end - range.start,
                font,
                color: if marker { marker_ink } else { ink },
                background_color: None,
                underline: marked
                    .filter(|marked| range.start >= marked.start && range.end <= marked.end)
                    .map(|_| UnderlineStyle {
                        color: Some(ink),
                        thickness: px(1.),
                        wavy: false,
                    }),
                strikethrough: None,
            }
        })
        .collect()
}

/// Cut spans at the edges of `at`, so no run straddles the composition.
fn split_spans(
    spans: Vec<(Range<usize>, MarkdownStyle, bool)>,
    at: &Range<usize>,
) -> Vec<(Range<usize>, MarkdownStyle, bool)> {
    let mut split = Vec::with_capacity(spans.len() + 2);
    for (range, emphasis, marker) in spans {
        let mut cuts = vec![range.start, range.end];
        for edge in [at.start, at.end] {
            if range.start < edge && edge < range.end {
                cuts.push(edge);
            }
        }
        cuts.sort_unstable();
        cuts.dedup();
        for pair in cuts.windows(2) {
            split.push((pair[0]..pair[1], emphasis, marker));
        }
    }
    split
}

/// The selection highlight, one quad per visual row it covers.
fn selection_quads(buffer: &Buffer, frame: &PaintedFrame, style: &EditorStyle) -> Vec<PaintQuad> {
    let selection = buffer.selection().range();
    if selection.is_empty() {
        return Vec::new();
    }

    let mut quads = Vec::new();
    for line in &frame.lines {
        let line_range = buffer.line_range(line.index);
        let start = selection.start.max(line_range.start);
        let end = selection.end.min(line_range.end);
        if start > end {
            continue;
        }
        // A selection running past the end of a paragraph shows the newline as
        // selected, so whole paragraphs read as full bars.
        let covers_newline = selection.end > line_range.end;

        let from = line.visual.display.display_offset(start - line_range.start);
        let to = line.visual.display.display_offset(end - line_range.start);
        let first_row = line.visual.row_at(from);
        let last_row = line.visual.row_at(to);

        for row in first_row..=last_row {
            let range = line.visual.row_range(row);
            let left = line.x_in_row(from.max(range.start), row);
            let right = line.x_in_row(to.min(range.end), row);
            let right = if row == last_row && covers_newline {
                right + px(6.)
            } else {
                right
            };
            if right <= left && !(row == last_row && covers_newline) {
                continue;
            }

            let top = line.top + line.line_height * row as f32;
            quads.push(fill(
                Bounds::from_corners(
                    point(frame.left + left, top),
                    point(frame.left + right, top + line.line_height),
                ),
                rgba(style.selection),
            ));
        }
    }
    quads
}

/// The caret, on the cursor's visual row. That row always renders raw, so its
/// display offsets are the source ones.
fn caret_quad(buffer: &Buffer, frame: &PaintedFrame, style: &EditorStyle) -> Option<PaintQuad> {
    let line = frame.line(buffer.cursor_line())?;
    let line_start = buffer.line_range(line.index).start;
    let offset = buffer.cursor() - line_start;
    let display = line.visual.display.display_offset(offset);
    let (row, _) = line.visual.position_of(offset);

    let top = line.top + line.line_height * row as f32;
    Some(fill(
        Bounds::new(
            point(frame.left + line.x_for(display), top),
            size(px(2.), line.line_height),
        ),
        rgb(style.caret),
    ))
}
