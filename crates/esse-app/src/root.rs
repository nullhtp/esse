//! The router: three screens, and the one rule that matters.
//!
//! An editor mode is reachable only with an essay in hand — both editor arms of
//! [`Screen`] carry a live view, and a view cannot be built without an essay.
//! There is no route that opens the editor without one, which is how "the
//! empty-document screen does not exist" is enforced by the shape of the code
//! rather than by a disabled button (design.md, D3, D1).

use std::rc::Rc;

use esse_core::{start_essay_from_spark, Error, Essay, EssayStatus, Result};
use gpui::{div, prelude::*, App, Context, Entity, FocusHandle, Focusable, Subscription, Window};

use crate::data::Data;
use crate::edit::{EditEvent, EditView};
use crate::today::{TodayEvent, TodayView};
use crate::write::{WriteEvent, WriteView};

/// What is on screen. The editor arms carry their view, so the essay travels
/// with the route rather than beside it.
enum Screen {
    Today,
    Write(Entity<WriteView>),
    Edit(Entity<EditView>),
}

pub struct RootView {
    data: Rc<Data>,
    today: Entity<TodayView>,
    screen: Screen,
    /// The window state to put back when the editor is left for Today.
    was_fullscreen: bool,
    _today: Subscription,
    _editor: Option<Subscription>,
}

impl RootView {
    pub fn new(data: Rc<Data>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = cx.new(|cx| TodayView::new(data.clone(), cx));
        let subscription = cx.subscribe_in(&today, window, Self::on_today);

        // Closing the window is leaving the editor: the text — and, in Write
        // mode, the session — are owed to the disk before it goes.
        let this = cx.entity().downgrade();
        window.on_window_should_close(cx, move |_, cx| {
            this.update(cx, |this, cx| match &this.screen {
                Screen::Write(write) => {
                    write.clone().update(cx, |write, cx| write.finish(cx));
                }
                Screen::Edit(edit) => {
                    edit.clone().update(cx, |edit, cx| edit.finish(cx));
                }
                Screen::Today => {}
            })
            .ok();
            true
        });

        RootView {
            data,
            today,
            screen: Screen::Today,
            was_fullscreen: false,
            _today: subscription,
            _editor: None,
        }
    }

    fn on_today(
        &mut self,
        _: &Entity<TodayView>,
        event: &TodayEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            TodayEvent::Write => self.write_pressed(window, cx),
            TodayEvent::Start(spark_id) => self.start_from_spark(spark_id, window, cx),
        }
    }

    /// The Write button: continue the essay in progress, or ask for a spark to
    /// start one (start-from-spark spec).
    fn write_pressed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.data.essays.in_progress() {
            Ok(Some(essay)) => self.open_editor(essay, window, cx),
            Ok(None) => {
                let sparks = self.data.sparks.load_all().map(|sparks| sparks.len());
                match sparks {
                    Ok(0) => self.tell_today(
                        "Чтобы начать, нужна искра. Запишите мысль — с неё и начнём.",
                        cx,
                    ),
                    Ok(_) => self
                        .today
                        .update(cx, |today, cx| today.offer_sparks(cx)),
                    Err(error) => self.tell_today(format!("Искры не читаются: {error}"), cx),
                }
            }
            Err(error) => {
                log::error!("could not look for an essay in progress: {error}");
                self.tell_today(format!("Эссе не читаются: {error}"), cx);
            }
        }
    }

    fn start_from_spark(&mut self, spark_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        match start_essay_from_spark(&self.data.sparks, &self.data.essays, spark_id) {
            Ok(essay) => {
                // The spark is gone from the box; the list has to agree.
                self.today.update(cx, |today, cx| today.refresh(cx));
                self.open_editor(essay, window, cx);
            }
            // Routing normally makes this impossible; the refusal is still
            // shown rather than swallowed (start-from-spark spec).
            Err(Error::EssayInProgress { slug, .. }) => self.tell_today(
                format!("Сейчас в работе «{slug}» — его нужно закончить, прежде чем начинать новое."),
                cx,
            ),
            Err(error) => {
                log::error!("could not start an essay: {error}");
                self.tell_today(format!("Не получилось начать: {error}"), cx);
            }
        }
    }

    fn tell_today(&mut self, notice: impl Into<String>, cx: &mut Context<Self>) {
        self.today.update(cx, |today, cx| today.say(notice, cx));
    }

    // -- the editor ------------------------------------------------------

    /// Open the editor from Today, in the mode the essay's state calls for: a
    /// Draft is still being written, an essay in Editing is being edited
    /// (start-from-spark spec). The window goes fullscreen and remembers what
    /// to put back (design.md, D2).
    fn open_editor(&mut self, essay: Essay, window: &mut Window, cx: &mut Context<Self>) {
        self.was_fullscreen = window.is_fullscreen();
        if !self.was_fullscreen {
            window.toggle_fullscreen();
        }
        match essay.status() {
            EssayStatus::Editing => self.show_edit(essay, window, cx),
            _ => self.show_write(essay, window, cx),
        }
    }

    /// Put Write mode on screen. Called both from Today and by a switch, and it
    /// touches the window in neither case — entering does that once, and a
    /// switch must not flicker (design.md, D2).
    fn show_write(&mut self, essay: Essay, window: &mut Window, cx: &mut Context<Self>) {
        let write = cx.new(|cx| WriteView::new(self.data.clone(), essay, cx));
        self._editor = Some(cx.subscribe_in(&write, window, Self::on_write));
        window.focus(&write.read(cx).focus_handle(cx), cx);
        self.screen = Screen::Write(write);
        cx.notify();
    }

    fn show_edit(&mut self, essay: Essay, window: &mut Window, cx: &mut Context<Self>) {
        let edit = cx.new(|cx| EditView::new(self.data.clone(), essay, cx));
        self._editor = Some(cx.subscribe_in(&edit, window, Self::on_edit));
        window.focus(&edit.read(cx).focus_handle(cx), cx);
        self.screen = Screen::Edit(edit);
        cx.notify();
    }

    fn on_write(
        &mut self,
        view: &Entity<WriteView>,
        event: &WriteEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            WriteEvent::Left => self.leave_editor(window, cx),
            WriteEvent::Switch => {
                let saved = view.update(cx, |write, cx| write.save(cx));
                let essay = view.read(cx).essay().clone();

                match saved.and_then(|()| self.move_to(essay, EssayStatus::Editing)) {
                    Ok(essay) => {
                        // The stretch of writing ends here, exactly as it would
                        // have on Escape (write-mode spec). Recorded only once
                        // the switch has actually happened, so a refusal leaves
                        // the writer mid-session rather than mid-nothing.
                        view.update(cx, |write, _| write.record_session());
                        self.show_edit(essay, window, cx);
                    }
                    Err(error) => {
                        log::error!("could not switch to Edit mode: {error}");
                        let trouble = format!("Не переключилось: {error}");
                        view.update(cx, |write, cx| write.complain(trouble, cx));
                    }
                }
            }
        }
    }

    fn on_edit(
        &mut self,
        view: &Entity<EditView>,
        event: &EditEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            EditEvent::Left => self.leave_editor(window, cx),
            EditEvent::Switch => {
                let saved = view.update(cx, |edit, cx| edit.save(cx));
                let essay = view.read(cx).essay().clone();

                match saved.and_then(|()| self.move_to(essay, EssayStatus::Draft)) {
                    Ok(essay) => self.show_write(essay, window, cx),
                    Err(error) => {
                        log::error!("could not switch to Write mode: {error}");
                        let trouble = format!("Не переключилось: {error}");
                        view.update(cx, |edit, cx| edit.complain(trouble, cx));
                    }
                }
            }
        }
    }

    /// The state half of a switch, run after the text is already on disk: move
    /// the essay through the lifecycle rules, then save it again. Any refusal
    /// here leaves the mode where it was, with the words intact (design.md, D4).
    fn move_to(&self, mut essay: Essay, to: EssayStatus) -> Result<Essay> {
        essay.transition_to(to)?;
        self.data.essays.save(&essay)?;
        Ok(essay)
    }

    fn leave_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Screen::Today;
        self._editor = None;

        if !self.was_fullscreen && window.is_fullscreen() {
            window.toggle_fullscreen();
        }

        self.today.update(cx, |today, cx| today.refresh(cx));
        window.focus(&self.today.read(cx).focus_handle(cx), cx);
        cx.notify();
    }
}

impl Focusable for RootView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        match &self.screen {
            Screen::Write(write) => write.read(cx).focus_handle(cx),
            Screen::Edit(edit) => edit.read(cx).focus_handle(cx),
            Screen::Today => self.today.read(cx).focus_handle(cx),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(match &self.screen {
            Screen::Write(write) => write.clone().into_any_element(),
            Screen::Edit(edit) => edit.clone().into_any_element(),
            Screen::Today => self.today.clone().into_any_element(),
        })
    }
}
