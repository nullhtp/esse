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
use crate::theme;

actions!(shelf, [Leave]);

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

pub struct ShelfView {
    data: Rc<Data>,
    focus_handle: FocusHandle,
    contents: Contents,
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
            }
            // One unreadable file makes the whole picture doubtful — and a
            // doubtful picture of what is in progress is not one to offer a
            // start from. The screen says so rather than showing half of it.
            Err(error) => {
                log::error!("could not read the shelf: {error}");
                self.contents = Contents::default();
                self.trouble = Some(format!("Полка не читается: {error}"));
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

    // -- the columns ------------------------------------------------------

    fn sparks_column(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // While an essay is in progress the sparks are a list to look at and
        // nothing more: there is no start to offer (shelf-screen spec).
        let startable = self.contents.in_progress.is_none();

        column("sparks", "Искры")
            .children(self.contents.sparks.is_empty().then(|| empty("Пока пусто")))
            .children(self.contents.sparks.iter().map(|spark| {
                let id = spark.id.clone();
                let row = row(spark.id.clone()).child(SharedString::from(spark.text.clone()));
                if startable {
                    pickable(row).on_click(
                        cx.listener(move |_, _, _, cx| cx.emit(ShelfEvent::Start(id.clone()))),
                    )
                } else {
                    row
                }
            }))
    }

    fn in_progress_column(&self, cx: &mut Context<Self>) -> impl IntoElement {
        column("in-progress", "В работе")
            .children(
                self.contents
                    .in_progress
                    .is_none()
                    .then(|| empty("Ничего не пишется")),
            )
            .children(self.contents.in_progress.as_ref().map(|essay| {
                pickable(row(essay.slug.clone()))
                    .child(SharedString::from(title(essay)))
                    .child(
                        div()
                            .pt(px(3.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::MUTED))
                            .child(match essay.status() {
                                EssayStatus::Draft => "черновик",
                                _ => "правится",
                            }),
                    )
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(ShelfEvent::Continue)))
            }))
    }

    fn published_column(&self) -> impl IntoElement {
        column("published", "Опубликовано")
            .children(
                self.contents
                    .published
                    .is_empty()
                    .then(|| empty("Пока ничего")),
            )
            .children(self.contents.published.iter().map(|essay| {
                row(essay.slug.clone())
                    .child(SharedString::from(title(essay)))
                    .children(essay.publication_url.clone().map(|url| {
                        div()
                            .pt(px(3.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::MUTED))
                            .child(SharedString::from(url))
                    }))
            }))
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
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.drawer_open = !this.drawer_open;
                        cx.notify();
                    }))
                    .child(format!("В столе {marker}")),
            )
            .children(self.drawer_open.then(|| {
                div()
                    .id("shelved")
                    .max_h(px(160.))
                    .overflow_y_scroll()
                    .pt(px(8.))
                    .text_size(px(theme::BODY_SIZE))
                    .text_color(rgb(theme::MUTED))
                    .children(self.contents.shelved.is_empty().then(|| empty("Стол пуст")))
                    .children(self.contents.shelved.iter().map(|essay| {
                        row(essay.slug.clone()).child(SharedString::from(title(essay)))
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

fn row(id: String) -> gpui::Stateful<gpui::Div> {
    div()
        .id(SharedString::from(id))
        .py(px(9.))
        .text_size(px(theme::BODY_SIZE))
}

/// A row that answers to the pointer, in the same treatment the Today list
/// uses while a spark is being chosen.
fn pickable(row: gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> {
    row.px(px(8.))
        .ml(px(-8.))
        .rounded(px(5.))
        .cursor_pointer()
        .hover(|style| style.bg(rgb(theme::HIGHLIGHT)))
}

fn empty(text: &'static str) -> impl IntoElement {
    div()
        .py(px(9.))
        .text_size(px(theme::BODY_SIZE))
        .text_color(rgb(theme::MUTED))
        .child(text)
}

/// What to call an essay on the shelf: the spark it grew from, which is how
/// the writer remembers it. A hand-written file without one falls back to its
/// file name.
fn title(essay: &Essay) -> String {
    essay.spark.clone().unwrap_or_else(|| essay.slug.clone())
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
            .key_context("Shelf")
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::INK))
            .on_action(cx.listener(Self::leave))
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
                    .child("Сегодня"),
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
