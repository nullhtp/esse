//! The Shelf: the whole conveyor at a glance.
//!
//! Three columns in the order the pipeline runs — Sparks, In progress,
//! Published — and the drawer of shelved essays under them, closed until it is
//! asked for. That is the entire screen: no counts, no statistics, no ordering
//! options, no folders (shelf-screen spec, anti-features).
//!
//! Like Today, it owns no routing. Choosing a spark or the card in progress
//! says so and nothing more; the router decides what that means, and it decides
//! it with the same `start-from-spark` code the Write button uses — the WIP
//! rule has one implementation, not two (design.md, D6).

use std::rc::Rc;

use esse_core::{Essay, EssayStatus, Spark};
use gpui::{
    actions, div, prelude::*, px, rgb, App, Context, EventEmitter, FocusHandle, Focusable,
    SharedString, Window,
};

use crate::data::Data;
use crate::essay::title;
use crate::keymap;
use crate::theme;

// Nothing on the Shelf is typed into, so the bare arrows and Enter are free to
// mean the cards (design.md, D6).
actions!(shelf, [Leave, Left, Right, Up, Down, Activate, Drawer]);

/// What the screen asks the router for.
pub enum ShelfEvent {
    /// Back to Today.
    Left,
    /// A spark was chosen to start an essay from.
    Start(String),
    /// The card in progress was activated: carry on with that essay.
    Continue,
}

/// The four lists the screen is made of, read in one go: either the whole
/// picture or none of it.
#[derive(Default)]
struct Contents {
    /// Newest first, as the column shows them.
    sparks: Vec<Spark>,
    /// The zero or one essay occupying the slot.
    in_progress: Option<Essay>,
    /// Newest published first.
    published: Vec<Essay>,
    shelved: Vec<Essay>,
}

/// Which card the keyboard is on: a column, and a row inside it. The columns
/// are numbered as they are drawn — sparks, in progress, published.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Spot {
    column: usize,
    row: usize,
}

pub struct ShelfView {
    data: Rc<Data>,
    focus_handle: FocusHandle,
    contents: Contents,
    /// The card under the highlight, when there is a card to be on at all.
    highlight: Option<Spot>,
    /// The drawer opens only to be looked at, and starts closed every time.
    drawer_open: bool,
    /// What the router had to say — a start it refused.
    notice: Option<String>,
    /// Shown when the disk refuses; the app's memory is worth complaining
    /// about out loud.
    trouble: Option<String>,
}

impl EventEmitter<ShelfEvent> for ShelfView {}

impl ShelfView {
    pub fn new(data: Rc<Data>, cx: &mut Context<Self>) -> Self {
        let mut view = ShelfView {
            data,
            focus_handle: cx.focus_handle(),
            contents: Contents::default(),
            highlight: None,
            drawer_open: false,
            notice: None,
            trouble: None,
        };
        view.reload();
        view
    }

    /// Re-read everything. The Shelf is opened afresh each time and refreshed
    /// on the way back from the editor, where all four columns can have moved.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.notice = None;
        self.reload();
        cx.notify();
    }

    /// Something the writer has to know — a start the storage layer refused.
    pub fn say(&mut self, notice: impl Into<String>, cx: &mut Context<Self>) {
        self.notice = Some(notice.into());
        cx.notify();
    }

    fn reload(&mut self) {
        match self.read() {
            Ok(contents) => {
                self.contents = contents;
                self.trouble = None;
                self.highlight = first_spot(self.columns(), self.startable());
            }
            // One unreadable file makes the whole picture doubtful — and a
            // doubtful picture of what is in progress is not one to offer a
            // start from. The screen says so rather than showing half of it.
            Err(error) => {
                log::error!("could not read the shelf: {error}");
                self.contents = Contents::default();
                self.highlight = None;
                self.trouble = Some(format!("Cannot read the shelf: {error}"));
            }
        }
    }

    fn read(&self) -> esse_core::Result<Contents> {
        Ok(Contents {
            sparks: self.data.sparks.load_all()?,
            in_progress: self.data.essays.in_progress()?,
            published: self.data.essays.published()?,
            shelved: self.data.essays.shelved()?,
        })
    }

    fn leave(&mut self, _: &Leave, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(ShelfEvent::Left);
    }

    // -- the highlight -----------------------------------------------------

    /// While an essay is in progress the sparks are a list to look at and
    /// nothing more: there is no start to offer (shelf-screen spec).
    fn startable(&self) -> bool {
        self.contents.in_progress.is_none()
    }

    /// How many cards each column holds, in the order they are drawn.
    fn columns(&self) -> [usize; COLUMNS] {
        [
            self.contents.sparks.len(),
            usize::from(self.contents.in_progress.is_some()),
            self.contents.published.len(),
        ]
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        self.step(step_column, -1, cx);
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        self.step(step_column, 1, cx);
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.step(step_row, -1, cx);
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.step(step_row, 1, cx);
    }

    fn step(
        &mut self,
        how: fn(Spot, [usize; COLUMNS], isize) -> Spot,
        delta: isize,
        cx: &mut Context<Self>,
    ) {
        let columns = self.columns();
        if let Some(spot) = self.highlight {
            self.highlight = Some(how(spot, columns, delta));
            cx.notify();
        }
    }

    /// Enter does to the highlighted card exactly what a click does to it: a
    /// spark starts an essay while the slot is free, the card in progress
    /// carries on with it, and a published card offers nothing to either.
    fn activate(&mut self, _: &Activate, _: &mut Window, cx: &mut Context<Self>) {
        let Some(spot) = self.highlight else {
            return;
        };
        match spot.column {
            SPARKS if self.startable() => {
                if let Some(spark) = self.contents.sparks.get(spot.row) {
                    cx.emit(ShelfEvent::Start(spark.id.clone()));
                }
            }
            IN_PROGRESS if self.contents.in_progress.is_some() => {
                cx.emit(ShelfEvent::Continue)
            }
            _ => {}
        }
    }

    fn drawer_toggled(&mut self, _: &Drawer, _: &mut Window, cx: &mut Context<Self>) {
        self.drawer_open = !self.drawer_open;
        cx.notify();
    }

    /// Whether the card at `column`/`row` is the one under the highlight.
    fn is_on(&self, column: usize, row: usize) -> bool {
        self.highlight == Some(Spot { column, row })
    }

    // -- the columns ------------------------------------------------------

    fn sparks_column(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let startable = self.startable();

        column("sparks", "Sparks")
            .children(self.contents.sparks.is_empty().then(|| empty("Empty for now")))
            .children(
                self.contents
                    .sparks
                    .iter()
                    .enumerate()
                    .map(|(index, spark)| {
                        let id = spark.id.clone();
                        let row = row(spark.id.clone(), self.is_on(SPARKS, index))
                            .child(SharedString::from(spark.text.clone()));
                        if startable {
                            pickable(row).on_click(cx.listener(move |_, _, _, cx| {
                                cx.emit(ShelfEvent::Start(id.clone()))
                            }))
                        } else {
                            row
                        }
                    }),
            )
    }

    fn in_progress_column(&self, cx: &mut Context<Self>) -> impl IntoElement {
        column("in-progress", "In progress")
            .children(
                self.contents
                    .in_progress
                    .is_none()
                    .then(|| empty("Nothing being written")),
            )
            .children(self.contents.in_progress.as_ref().map(|essay| {
                pickable(row(essay.slug.clone(), self.is_on(IN_PROGRESS, 0)))
                    .child(SharedString::from(title(essay)))
                    .child(
                        div()
                            .pt(px(3.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::MUTED))
                            .child(match essay.status() {
                                EssayStatus::Draft => "draft",
                                _ => "editing",
                            }),
                    )
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(ShelfEvent::Continue)))
            }))
    }

    fn published_column(&self) -> impl IntoElement {
        column("published", "Published")
            .children(
                self.contents
                    .published
                    .is_empty()
                    .then(|| empty("Nothing yet")),
            )
            .children(
                self.contents
                    .published
                    .iter()
                    .enumerate()
                    .map(|(index, essay)| {
                        row(essay.slug.clone(), self.is_on(PUBLISHED, index))
                            .child(SharedString::from(title(essay)))
                            .children(essay.publication_url.clone().map(|url| {
                                div()
                                    .pt(px(3.))
                                    .text_size(px(theme::SMALL_SIZE))
                                    .text_color(rgb(theme::MUTED))
                                    .child(SharedString::from(url))
                            }))
                    }),
            )
    }

    /// The drawer. Shelved essays are here to be seen and nothing else — the
    /// lifecycle forbids leaving Shelved, so no action on one is offered.
    fn drawer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let marker = if self.drawer_open { "﹀" } else { "›" };

        div()
            .pt(px(22.))
            .border_t_1()
            .border_color(rgb(theme::RULE))
            .child(
                div()
                    .id("drawer")
                    .text_size(px(theme::SMALL_SIZE))
                    .text_color(rgb(theme::MUTED))
                    .cursor_pointer()
                    .hover(|style| style.text_color(rgb(theme::INK)))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.drawer_toggled(&Drawer, window, cx)
                    }))
                    .child(format!("Shelved {marker}")),
            )
            .children(self.drawer_open.then(|| {
                div()
                    .id("shelved")
                    .max_h(px(160.))
                    .overflow_y_scroll()
                    .pt(px(8.))
                    .text_size(px(theme::BODY_SIZE))
                    .text_color(rgb(theme::MUTED))
                    .children(self.contents.shelved.is_empty().then(|| empty("Nothing shelved")))
                    .children(self.contents.shelved.iter().map(|essay| {
                        row(essay.slug.clone(), false).child(SharedString::from(title(essay)))
                    }))
            }))
    }
}

/// A column: its quiet heading, and the rows under a rule.
fn column(id: &'static str, heading: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .flex_1()
        .min_w(px(0.))
        .h_full()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .child(
            div()
                .pb(px(8.))
                .mb(px(10.))
                .border_b_1()
                .border_color(rgb(theme::RULE))
                .text_size(px(theme::SMALL_SIZE))
                .text_color(rgb(theme::MUTED))
                .child(heading),
        )
}

/// A row, in its nook, lit when the keyboard is on it.
fn row(id: String, highlighted: bool) -> gpui::Stateful<gpui::Div> {
    div()
        .id(SharedString::from(id))
        .py(px(9.))
        .px(px(8.))
        .ml(px(-8.))
        .rounded(px(5.))
        .text_size(px(theme::BODY_SIZE))
        .when(highlighted, |row| row.bg(rgb(theme::HIGHLIGHT)))
}

/// A row that answers to the pointer, in the same treatment the Today list
/// uses while a spark is being chosen.
fn pickable(row: gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> {
    row.cursor_pointer()
        .hover(|style| style.bg(rgb(theme::HIGHLIGHT)))
}

/// The three columns, in the order the pipeline runs and the screen draws them.
const SPARKS: usize = 0;
const IN_PROGRESS: usize = 1;
const PUBLISHED: usize = 2;
const COLUMNS: usize = 3;

/// Where the highlight sits when the Shelf opens: the first card that can
/// actually be acted on, and failing that the first card there is
/// (shelf-screen spec).
fn first_spot(columns: [usize; COLUMNS], startable: bool) -> Option<Spot> {
    let actionable = [(SPARKS, startable), (IN_PROGRESS, true)]
        .into_iter()
        .find(|&(column, offered)| offered && columns[column] > 0);

    let column = match actionable {
        Some((column, _)) => column,
        None => (0..COLUMNS).find(|&column| columns[column] > 0)?,
    };
    Some(Spot { column, row: 0 })
}

/// One step sideways, over the columns that have something in them. The ends
/// are ends: there is no wrapping round a picture of a conveyor.
fn step_column(spot: Spot, columns: [usize; COLUMNS], delta: isize) -> Spot {
    let mut column = spot.column;
    loop {
        let next = column as isize + delta;
        if next < 0 || next >= COLUMNS as isize {
            break;
        }
        column = next as usize;
        if columns[column] > 0 {
            return Spot {
                column,
                row: spot.row.min(columns[column] - 1),
            };
        }
    }
    spot
}

/// One step along a column, stopping at its ends.
fn step_row(spot: Spot, columns: [usize; COLUMNS], delta: isize) -> Spot {
    let row = (spot.row as isize + delta).clamp(0, columns[spot.column].saturating_sub(1) as isize);
    Spot {
        column: spot.column,
        row: row as usize,
    }
}

fn empty(text: &'static str) -> impl IntoElement {
    div()
        .py(px(9.))
        .text_size(px(theme::BODY_SIZE))
        .text_color(rgb(theme::MUTED))
        .child(text)
}

impl Focusable for ShelfView {
    /// Nothing here is typed into; the screen holds focus so that Escape has
    /// somewhere to land.
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShelfView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sparks = self.sparks_column(cx);
        let in_progress = self.in_progress_column(cx);
        let published = self.published_column();
        let drawer = self.drawer(cx);

        div()
            .key_context(keymap::SHELF)
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::INK))
            .on_action(cx.listener(Self::leave))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::activate))
            .on_action(cx.listener(Self::drawer_toggled))
            .child(
                // The way back, named for the room it leads to — the same
                // quiet corner control the editor rooms use.
                div()
                    .id("today")
                    .absolute()
                    .top(px(18.))
                    .right(px(22.))
                    .text_size(px(theme::SMALL_SIZE))
                    .text_color(rgb(theme::MUTED))
                    .cursor_pointer()
                    .hover(|style| style.text_color(rgb(theme::INK)))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(ShelfEvent::Left)))
                    .child("Today"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .h_full()
                    .w(px(SHELF_MEASURE))
                    .max_w_full()
                    .px(px(theme::PAGE_PADDING))
                    .pt(px(theme::PAGE_PADDING + 14.))
                    .pb(px(theme::PAGE_PADDING))
                    .children(self.notice.clone().map(|text| {
                        div()
                            .pb(px(14.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::MUTED))
                            .child(text)
                    }))
                    .children(self.trouble.clone().map(|text| {
                        div()
                            .pb(px(14.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::ALARM))
                            .child(text)
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .min_h(px(0.))
                            .gap(px(26.))
                            .child(sparks)
                            .child(in_progress)
                            .child(published),
                    )
                    .child(drawer),
            )
    }
}

/// Wider than the writing measure: three columns of short lines, not a column
/// of prose.
const SHELF_MEASURE: f32 = 960.;

#[cfg(test)]
mod tests {
    use super::*;

    fn spot(column: usize, row: usize) -> Option<Spot> {
        Some(Spot { column, row })
    }

    #[test]
    fn the_highlight_starts_on_the_first_card_worth_pressing() {
        // A free slot: the sparks are startable, so the first one it is.
        assert_eq!(first_spot([3, 0, 2], true), spot(SPARKS, 0));
        // Occupied: the sparks offer nothing, and the essay in progress does.
        assert_eq!(first_spot([3, 1, 2], false), spot(IN_PROGRESS, 0));
        // Nothing to act on at all: the first column with anything in it.
        assert_eq!(first_spot([0, 0, 2], true), spot(PUBLISHED, 0));
        assert_eq!(first_spot([0, 0, 0], true), None);
    }

    #[test]
    fn sideways_steps_skip_the_empty_columns_and_stop_at_the_ends() {
        let columns = [2, 0, 3];

        let start = Spot { column: 0, row: 1 };
        assert_eq!(step_column(start, columns, 1), Spot { column: 2, row: 1 });
        assert_eq!(step_column(start, columns, -1), start, "already leftmost");

        let end = Spot { column: 2, row: 2 };
        assert_eq!(step_column(end, columns, 1), end, "already rightmost");
        assert_eq!(
            step_column(end, columns, -1),
            Spot { column: 0, row: 1 },
            "a short column clamps the row"
        );
    }

    #[test]
    fn stepping_along_a_column_stops_at_its_ends() {
        let columns = [3, 1, 0];

        let top = Spot { column: 0, row: 0 };
        assert_eq!(step_row(top, columns, -1), top);
        assert_eq!(step_row(top, columns, 1), Spot { column: 0, row: 1 });

        let bottom = Spot { column: 0, row: 2 };
        assert_eq!(step_row(bottom, columns, 1), bottom);
    }
}
