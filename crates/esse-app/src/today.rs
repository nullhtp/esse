//! The Today screen: capture a spark, and see the ones already captured.
//!
//! For now that is the whole app. Later stages hang the "Write" button, the
//! row of published essays and the session dots on this same frame.

use esse_core::{Spark, SparkStore};
use gpui::{
    div, prelude::*, px, rgb, App, Context, Entity, FocusHandle, Focusable, SharedString,
    Subscription, Window,
};

use crate::spark_input::{SparkInput, Submitted};
use crate::theme;

pub struct TodayView {
    store: SparkStore,
    /// Newest first, as the list shows them.
    sparks: Vec<Spark>,
    input: Entity<SparkInput>,
    /// Shown when the disk refuses; the app's memory is worth complaining
    /// about out loud.
    trouble: Option<String>,
    _submitted: Subscription,
}

impl TodayView {
    pub fn new(store: SparkStore, cx: &mut Context<Self>) -> Self {
        let input = cx.new(SparkInput::new);
        let submitted = cx.subscribe(&input, Self::on_submitted);

        let (sparks, trouble) = match store.load_all() {
            Ok(sparks) => (sparks, None),
            Err(error) => {
                log::error!("could not read the sparks: {error}");
                (Vec::new(), Some(format!("Искры не читаются: {error}")))
            }
        };

        TodayView {
            store,
            sparks,
            input,
            trouble,
            _submitted: submitted,
        }
    }

    fn on_submitted(
        &mut self,
        input: Entity<SparkInput>,
        event: &Submitted,
        cx: &mut Context<Self>,
    ) {
        match self.store.capture(&event.0) {
            // Nothing but whitespace: not a spark, and the line stays as it is.
            Ok(None) => {}
            Ok(Some(spark)) => {
                self.sparks.insert(0, spark);
                self.trouble = None;
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

    fn list(&self) -> impl IntoElement {
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

        list.children(
            self.sparks
                .iter()
                .map(|spark| div().py(px(9.)).child(SharedString::from(spark.text.clone()))),
        )
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
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("Today")
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::INK))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .h_full()
                    .w(px(theme::MEASURE))
                    .max_w_full()
                    .px(px(theme::PAGE_PADDING))
                    .pt(px(theme::PAGE_PADDING))
                    .child(self.input.clone())
                    .children(self.trouble.clone().map(|text| {
                        div()
                            .pt(px(10.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::ALARM))
                            .child(text)
                    }))
                    .child(self.list()),
            )
    }
}
