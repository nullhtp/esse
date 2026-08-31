//! The Today screen: press Write, capture a spark, see the ones already
//! captured — and, below them, what the writing has come to: the essays
//! already published and the dotted line of the days they were written on.
//!
//! The screen owns no routing. Pressing Write says so and nothing more — the
//! router decides whether that continues an essay or asks for a spark, and
//! hands the answer back through [`TodayView::offer_sparks`] and
//! [`TodayView::say`]. That is what keeps "the button carries no other
//! behavior" (today-screen spec) true in the code and not just on paper.
//!
//! The two lower sections are read-only and quiet on purpose: neither can be
//! started from, opened, or counted, and both vanish when they have nothing to
//! show. Today stays a launchpad rather than becoming a dashboard.

use std::collections::HashSet;
use std::rc::Rc;

use chrono::{Local, NaiveDate};
use esse_core::{Essay, Spark};
use gpui::{
    div, prelude::*, px, rgb, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    SharedString, Subscription, Window,
};

use crate::calendar;
use crate::data::Data;
use crate::essay::title;
use crate::line_input::{LineInput, Submitted};
use crate::theme;

/// What the screen asks the router for.
pub enum TodayEvent {
    /// The Write button was pressed.
    Write,
    /// A spark was chosen to start an essay from.
    Start(String),
    /// The Shelf was asked for.
    Shelf,
}

pub struct TodayView {
    data: Rc<Data>,
    /// Newest first, as the list shows them.
    sparks: Vec<Spark>,
    /// Newest published first, as the row shows them.
    published: Vec<Essay>,
    /// The days at least one session started on — the filled dots.
    days_written: HashSet<NaiveDate>,
    input: Entity<LineInput>,
    /// The list is waiting for a spark to be chosen.
    picking: bool,
    /// What the router had to say — an empty spark box, a refused start.
    notice: Option<String>,
    /// Shown when the disk refuses; the app's memory is worth complaining
    /// about out loud.
    trouble: Option<String>,
    _submitted: Subscription,
}

impl EventEmitter<TodayEvent> for TodayView {}

impl TodayView {
    pub fn new(data: Rc<Data>, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| LineInput::new("Новая искра", cx));
        let submitted = cx.subscribe(&input, Self::on_submitted);

        let mut view = TodayView {
            data,
            sparks: Vec::new(),
            published: Vec::new(),
            days_written: HashSet::new(),
            input,
            picking: false,
            notice: None,
            trouble: None,
            _submitted: submitted,
        };
        view.reload();
        view
    }

    /// Re-read the disk. Called when coming back from Write mode, where a
    /// spark may have been consumed (spark-capture spec), a session recorded,
    /// and an essay published.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.picking = false;
        self.notice = None;
        self.reload();
        cx.notify();
    }

    /// The slot is free and there are sparks: let the writer choose one.
    pub fn offer_sparks(&mut self, cx: &mut Context<Self>) {
        self.picking = true;
        self.notice = None;
        cx.notify();
    }

    /// Something the writer has to know — no sparks to start from, or a start
    /// the storage layer refused.
    pub fn say(&mut self, notice: impl Into<String>, cx: &mut Context<Self>) {
        self.picking = false;
        self.notice = Some(notice.into());
        cx.notify();
    }

    fn reload(&mut self) {
        match self.data.sparks.load_all() {
            Ok(sparks) => {
                self.sparks = sparks;
                self.trouble = None;
            }
            Err(error) => {
                log::error!("could not read the sparks: {error}");
                self.sparks = Vec::new();
                self.trouble = Some(format!("Искры не читаются: {error}"));
            }
        }

        // The two lower sections are a view of what is already done, and a
        // showcase that cannot be read is simply not shown: the trouble goes to
        // the log, the screen stays a place to start writing from. The Shelf is
        // where the disk gets to complain about essays out loud.
        self.published = self.data.essays.published().unwrap_or_else(|error| {
            log::error!("could not read the published essays: {error}");
            Vec::new()
        });
        self.days_written = self
            .data
            .sessions
            .load_all()
            .map(|sessions| calendar::days_written(&sessions))
            .unwrap_or_else(|error| {
                log::error!("could not read the session history: {error}");
                HashSet::new()
            });
    }

    fn on_submitted(
        &mut self,
        input: Entity<LineInput>,
        event: &Submitted,
        cx: &mut Context<Self>,
    ) {
        match self.data.sparks.capture(&event.0) {
            // Nothing but whitespace: not a spark, and the line stays as it is.
            Ok(None) => {}
            Ok(Some(spark)) => {
                self.sparks.insert(0, spark);
                self.trouble = None;
                // A spark in the box answers "a spark is needed".
                self.notice = None;
                input.update(cx, |input, cx| input.clear(cx));
            }
            Err(error) => {
                // The text stays in the field: a spark that failed to save is
                // still on the screen rather than lost.
                log::error!("could not save a spark: {error}");
                self.trouble = Some(format!("Не сохранилось: {error}"));
            }
        }
        cx.notify();
    }

    fn write_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("write")
            .flex()
            .items_center()
            .justify_center()
            .w_full()
            .h(px(60.))
            .rounded(px(8.))
            .bg(rgb(theme::INK))
            .text_color(rgb(theme::BACKGROUND))
            .text_size(px(21.))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(theme::INK_HOVER)))
            .on_click(cx.listener(|_, _, _, cx| cx.emit(TodayEvent::Write)))
            .child("Писать")
    }

    /// The way to the Shelf: a corner control, in the same quiet register as
    /// the editor's. Navigation, not a second thing to do — the Write button
    /// and the capture line keep the screen (today-screen spec).
    fn shelf_control(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("shelf")
            .absolute()
            .top(px(18.))
            .right(px(22.))
            .text_size(px(theme::SMALL_SIZE))
            .text_color(rgb(theme::MUTED))
            .cursor_pointer()
            .hover(|style| style.text_color(rgb(theme::INK)))
            .on_click(cx.listener(|_, _, _, cx| cx.emit(TodayEvent::Shelf)))
            .child("Полка")
    }

    fn list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut list = div()
            .id("sparks")
            .flex_1()
            // The list gives way to the showcase below rather than pushing it
            // off the screen; what does not fit scrolls.
            .min_h(px(0.))
            .overflow_y_scroll()
            .pt(px(20.))
            .flex()
            .flex_col()
            .text_size(px(theme::BODY_SIZE));

        if self.sparks.is_empty() {
            list = list.child(
                div()
                    .pt(px(8.))
                    .text_color(rgb(theme::MUTED))
                    .child("Пока пусто. Первая мысль, которая придёт, станет первой искрой."),
            );
        }

        let picking = self.picking;
        list.children(self.sparks.iter().map(|spark| {
            let id = spark.id.clone();
            let row = div()
                .id(SharedString::from(spark.id.clone()))
                .py(px(9.))
                .child(SharedString::from(spark.text.clone()));
            if picking {
                row.px(px(8.))
                    .ml(px(-8.))
                    .rounded(px(5.))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(theme::HIGHLIGHT)))
                    .on_click(
                        cx.listener(move |_, _, _, cx| cx.emit(TodayEvent::Start(id.clone()))),
                    )
            } else {
                row
            }
        }))
    }

    // -- what the writing has come to -------------------------------------

    /// The showcase at the quiet bottom edge: the published essays, then the
    /// dotted calendar. Both are dimmer and smaller than the spark list above
    /// them, and the whole block — the rule included — is absent while neither
    /// has anything to show (today-screen spec).
    fn showcase(&self) -> Option<impl IntoElement> {
        let published = self.published_row();
        let dots = self.session_dots();
        if published.is_none() && dots.is_none() {
            return None;
        }

        Some(
            div()
                .flex_none()
                .mt(px(24.))
                .pt(px(4.))
                .border_t_1()
                .border_color(rgb(theme::RULE))
                .children(published)
                .children(dots),
        )
    }

    /// The finished essays, newest first: a card each, with the link if the
    /// writer recorded one. Nothing here starts, opens or changes an essay —
    /// it is a shelf to look at, not a workspace (published-row spec).
    fn published_row(&self) -> Option<impl IntoElement> {
        if self.published.is_empty() {
            return None;
        }

        Some(
            div()
                .id("published")
                .pt(px(18.))
                .flex()
                .gap(px(10.))
                .overflow_x_scroll()
                .children(self.published.iter().map(|essay| {
                    div()
                        .flex_none()
                        .w(px(CARD_WIDTH))
                        .p(px(10.))
                        .rounded(px(6.))
                        .border_1()
                        .border_color(rgb(theme::RULE))
                        .text_size(px(theme::SMALL_SIZE))
                        .child(div().truncate().child(SharedString::from(title(essay))))
                        .children(essay.publication_url.clone().map(|url| {
                            div()
                                .pt(px(4.))
                                .truncate()
                                .text_color(rgb(theme::MUTED))
                                .child(SharedString::from(url))
                        }))
                })),
        )
    }

    /// The last four weeks as a dotted line, today rightmost. A dot is filled
    /// where there was writing and empty where there was none — no numbers, no
    /// labels, nothing to hover or click, and no line at all through a month
    /// without writing (session-calendar spec).
    fn session_dots(&self) -> Option<impl IntoElement> {
        let strip = calendar::strip(&self.days_written, Local::now().date_naive())?;

        Some(
            div()
                .pt(px(20.))
                .pb(px(2.))
                .flex()
                .items_center()
                .gap(px(6.))
                .children(strip.into_iter().map(|written| {
                    div().size(px(5.)).rounded(px(2.5)).bg(rgb(if written {
                        theme::MUTED
                    } else {
                        theme::RULE
                    }))
                })),
        )
    }
}

/// A published card: wide enough for a few words of a title, narrow enough
/// that the row reads as a shelf of finished things rather than a list.
const CARD_WIDTH: f32 = 172.;

impl Focusable for TodayView {
    /// The screen's focus is the capture line: there is nothing else to type
    /// into, so launching the app is already being ready to write.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }
}

impl Render for TodayView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let button = self.write_button(cx);
        let list = self.list(cx);
        let shelf = self.shelf_control(cx);
        let showcase = self.showcase();

        div()
            .key_context("Today")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::INK))
            .child(shelf)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .h_full()
                    .w(px(theme::MEASURE))
                    .max_w_full()
                    .px(px(theme::PAGE_PADDING))
                    .pt(px(theme::PAGE_PADDING))
                    .pb(px(theme::PAGE_PADDING))
                    .child(button)
                    .children(self.picking.then(|| {
                        div()
                            .pt(px(14.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::MUTED))
                            .child("С какой искры начать? Выберите её в списке.")
                    }))
                    .children(self.notice.clone().map(|text| {
                        div()
                            .pt(px(14.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::MUTED))
                            .child(text)
                    }))
                    .child(div().pt(px(28.)).child(self.input.clone()))
                    .children(self.trouble.clone().map(|text| {
                        div()
                            .pt(px(10.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::ALARM))
                            .child(text)
                    }))
                    .child(list)
                    .children(showcase),
            )
    }
}
