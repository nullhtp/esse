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
    actions, div, prelude::*, px, rgb, AnyElement, App, Context, Entity, EventEmitter, FocusHandle,
    Focusable, SharedString, Subscription, Window,
};

use crate::calendar;
use crate::data::Data;
use crate::essay::title;
use crate::keymap;
use crate::line_input::{LineInput, Submitted};
use crate::theme;

// The screen's two moves from the keyboard. Both are bound with a modifier, so
// that plain typing keeps landing in the capture line (keyboard-shortcuts
// spec) — and the four below them are not, because choosing a spark takes the
// focus off the capture line for as long as it lasts (design.md, D5).
actions!(today, [Write, Shelf, Previous, Next, Choose, Cancel]);

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
    /// Which spark the choice is on, while one is being chosen. An index into
    /// `sparks`, which is newest first — so it starts at the newest.
    highlight: usize,
    /// Focus while choosing. The capture line gives it up for as long as the
    /// choice is open and takes it back on the way out (start-from-spark spec).
    choice_focus: FocusHandle,
    /// The app is still empty enough to be asking for a boxful of sparks.
    asking: bool,
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
        let input = cx.new(|cx| LineInput::new("New spark", cx));
        let submitted = cx.subscribe(&input, Self::on_submitted);

        let mut view = TodayView {
            data,
            sparks: Vec::new(),
            published: Vec::new(),
            days_written: HashSet::new(),
            input,
            picking: false,
            highlight: 0,
            choice_focus: cx.focus_handle(),
            asking: false,
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
    pub fn refresh(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.stop_choosing(window, cx);
        self.notice = None;
        self.reload();
        cx.notify();
    }

    /// The slot is free and there are sparks: let the writer choose one. The
    /// choice takes focus, so the arrows and Enter mean the list rather than
    /// the capture line, and the newest spark is the one under the highlight.
    pub fn offer_sparks(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.picking = true;
        self.highlight = 0;
        self.notice = None;
        window.focus(&self.choice_focus, cx);
        cx.notify();
    }

    /// Whether a spark is being chosen — which is a different room to be told
    /// the shortcuts of (shortcut-help spec).
    pub fn is_choosing(&self) -> bool {
        self.picking
    }

    /// Something the writer has to know — no sparks to start from, or a start
    /// the storage layer refused. Whatever it says, the choice is over and the
    /// capture line has the keys back.
    pub fn say(&mut self, notice: impl Into<String>, window: &mut Window, cx: &mut Context<Self>) {
        self.stop_choosing(window, cx);
        self.notice = Some(notice.into());
        cx.notify();
    }

    /// Leave the choosing state, whatever ended it. The capture line takes the
    /// keys back the moment the choice is gone, so no keystroke ever falls
    /// through to a list that is no longer on screen.
    fn stop_choosing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.picking {
            self.picking = false;
            window.focus(&self.input.read(cx).focus_handle(cx), cx);
        }
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
                self.trouble = Some(format!("Cannot read the sparks: {error}"));
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

        self.asking = self.asking_for_sparks();
    }

    /// Whether the app is still empty enough to ask for sparks: fewer than a
    /// boxful captured and nothing written yet, in any state. Computed from
    /// the disk every time rather than remembered — there is no onboarding
    /// flag anywhere (first-run-onboarding spec, design.md D1).
    ///
    /// Past a boxful of sparks the answer is already no, and the essays are
    /// not read to confirm it. A disk that refuses to answer is not an empty
    /// app, so the ask stays away.
    fn asking_for_sparks(&self) -> bool {
        self.sparks.len() < ENOUGH_SPARKS
            && self
                .data
                .essays
                .load_all()
                .inspect_err(|error| log::error!("could not read the essays: {error}"))
                .is_ok_and(|essays| essays.is_empty())
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
                // And the boxful being asked for may have just been reached.
                self.asking = self.asking_for_sparks();
                input.update(cx, |input, cx| input.clear(cx));
            }
            Err(error) => {
                // The text stays in the field: a spark that failed to save is
                // still on the screen rather than lost.
                log::error!("could not save a spark: {error}");
                self.trouble = Some(format!("Did not save: {error}"));
            }
        }
        cx.notify();
    }

    /// The Write button from the keyboard, and the Shelf from the keyboard:
    /// each emits the very event its control emits, so a shortcut cannot
    /// drift into meaning something else (keyboard-shortcuts spec).
    fn write(&mut self, _: &Write, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(TodayEvent::Write);
    }

    fn shelf(&mut self, _: &Shelf, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(TodayEvent::Shelf);
    }

    // -- choosing a spark --------------------------------------------------

    fn previous(&mut self, _: &Previous, _: &mut Window, cx: &mut Context<Self>) {
        self.highlight = self.highlight.saturating_sub(1);
        cx.notify();
    }

    fn next(&mut self, _: &Next, _: &mut Window, cx: &mut Context<Self>) {
        self.highlight = (self.highlight + 1).min(self.sparks.len().saturating_sub(1));
        cx.notify();
    }

    /// Enter starts from the highlighted spark through the very event a click
    /// on it emits, so the two ways in cannot mean different things.
    fn choose(&mut self, _: &Choose, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(spark) = self.sparks.get(self.highlight) {
            cx.emit(TodayEvent::Start(spark.id.clone()));
        }
    }

    fn cancel(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>) {
        self.stop_choosing(window, cx);
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
            .child("Write")
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
            .child("Shelf")
    }

    /// The whole of the onboarding: two quiet lines under the capture line,
    /// asking for a boxful of sparks. Not a modal, not a step, and nothing to
    /// dismiss — it goes away when the box fills or the writing starts
    /// (first-run-onboarding spec).
    fn ask(&self) -> Option<impl IntoElement> {
        if !self.asking {
            return None;
        }

        Some(
            div()
                .pt(px(12.))
                .text_size(px(theme::SMALL_SIZE))
                .text_color(rgb(theme::MUTED))
                .child(if self.sparks.is_empty() {
                    "Ideas arrive every day and get lost. Write down three to five \
                     right now — the first essay starts from one of them."
                } else {
                    "Add a couple more — so there is something to choose from."
                }),
        )
    }

    fn list(&self, cx: &mut Context<Self>) -> AnyElement {
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

        // The ask, when it is up, already says what an empty box means; the
        // screen does not say it twice.
        if self.sparks.is_empty() && !self.asking {
            list = list.child(
                div()
                    .pt(px(8.))
                    .text_color(rgb(theme::MUTED))
                    .child("Empty for now. The first thought that comes will be the first spark."),
            );
        }

        let picking = self.picking;
        let highlight = self.highlight;
        let list = list.children(self.sparks.iter().enumerate().map(|(index, spark)| {
            let id = spark.id.clone();
            let row = div()
                .id(SharedString::from(spark.id.clone()))
                .py(px(9.))
                .child(SharedString::from(spark.text.clone()));
            if picking {
                row.px(px(8.))
                    .ml(px(-8.))
                    .rounded(px(5.))
                    .when(index == highlight, |row| row.bg(rgb(theme::HIGHLIGHT)))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(theme::HIGHLIGHT)))
                    .on_click(
                        cx.listener(move |_, _, _, cx| cx.emit(TodayEvent::Start(id.clone()))),
                    )
            } else {
                row
            }
        }));

        // The list only takes focus while there is a choice to make: at rest
        // the capture line owns the screen's keys, and a click anywhere must
        // not take them off it (today-screen spec).
        if picking {
            list.track_focus(&self.choice_focus).into_any_element()
        } else {
            list.into_any_element()
        }
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

/// A boxful — the number of sparks the first launch asks for, and the point
/// at which it stops asking. One spark is not yet something to choose from.
const ENOUGH_SPARKS: usize = 3;

impl Focusable for TodayView {
    /// The screen's focus is the capture line: there is nothing else to type
    /// into, so launching the app is already being ready to write. The one
    /// exception is the spark choice, which borrows it while it is open.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        if self.picking {
            self.choice_focus.clone()
        } else {
            self.input.read(cx).focus_handle(cx)
        }
    }
}

impl Render for TodayView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let button = self.write_button(cx);
        let ask = self.ask();
        let list = self.list(cx);
        let shelf = self.shelf_control(cx);
        let showcase = self.showcase();

        div()
            // Choosing a spark is a room of its own, entered by one action and
            // left by one key — which is what makes the bare arrows and Enter
            // safe there and nowhere else on Today (design.md, D5).
            .key_context(if self.picking {
                keymap::CHOOSING
            } else {
                keymap::TODAY
            })
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::INK))
            .on_action(cx.listener(Self::write))
            .on_action(cx.listener(Self::shelf))
            .on_action(cx.listener(Self::previous))
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::choose))
            .on_action(cx.listener(Self::cancel))
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
                            .child("Which spark to start from? Pick one in the list.")
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
                    .children(ask)
                    .child(list)
                    .children(showcase),
            )
    }
}
