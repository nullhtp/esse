//! The Today screen: press Write, capture a spark, and see the ones already
//! captured.
//!
//! The screen owns no routing. Pressing Write says so and nothing more — the
//! router decides whether that continues an essay or asks for a spark, and
//! hands the answer back through [`TodayView::offer_sparks`] and
//! [`TodayView::say`]. That is what keeps "the button carries no other
//! behavior" (today-screen spec) true in the code and not just on paper.

use std::rc::Rc;

use esse_core::Spark;
use gpui::{
    div, prelude::*, px, rgb, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    SharedString, Subscription, Window,
};

use crate::data::Data;
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
            input,
            picking: false,
            notice: None,
            trouble: None,
            _submitted: submitted,
        };
        view.reload();
        view
    }

    /// Re-read the spark box. Called when coming back from Write mode, where a
    /// spark may have been consumed (spark-capture spec).
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
    }

    fn on_submitted(&mut self, input: Entity<LineInput>, event: &Submitted, cx: &mut Context<Self>) {
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
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.emit(TodayEvent::Start(id.clone()))
                    }))
            } else {
                row
            }
        }))
    }
}

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
                    .child(list),
            )
    }
}
