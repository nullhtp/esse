//! Write mode: the screen an essay is actually written on.
//!
//! Fullscreen, dark, no chrome — the editor and two quiet corners: how long the
//! session has run, and the way across to Edit mode. The essay saves itself a
//! second after the last keystroke and again on the way out; the session is
//! recorded when the writer leaves, however they leave.

use std::rc::Rc;
use std::time::{Duration, Instant};

use esse_core::model::{now, Timestamp};
use esse_core::{Essay, Result, Session};
use gpui::{
    actions, div, prelude::*, px, rgb, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    SharedString, Subscription, Window,
};

use crate::autosave::Autosave;
use crate::data::Data;
use crate::editor::{EditorStyle, EditorView, Edited, Viewport};
use crate::theme;

actions!(write, [Leave, Switch]);

/// The session target: a code constant, because settings are an anti-feature
/// and the concept's 15–25 minutes is a rhythm, not a knob (design.md, D8).
const SESSION_TARGET: Duration = Duration::from_secs(20 * 60);
/// How often the corner indicator refreshes. It shows whole minutes, so this
/// only has to be finer than a minute.
const TICK: Duration = Duration::from_secs(5);
/// Anything shorter is a peek, not a session (writing-sessions spec).
const SHORTEST_SESSION: Duration = Duration::from_secs(60);

/// What Write mode asks the router for. Both ways out are explicit, and neither
/// leaves the text unsaved (write-mode spec).
pub enum WriteEvent {
    /// Back to Today. The essay is saved and the session recorded; the essay
    /// stays a Draft.
    Left,
    /// Across to Edit mode. The text is saved; the Draft → Editing transition
    /// and the session are the router's to finish, because either can fail and
    /// a failed switch stays put.
    Switch,
}

pub struct WriteView {
    autosave: Autosave,
    editor: Entity<EditorView>,
    /// When the session began — the record's `started_at`, kept in the
    /// writer's own offset, and `Instant` for measuring across a sleeping Mac.
    started_at: Timestamp,
    started: Instant,
    /// Sessions are recorded once, whether the writer leaves, switches or
    /// quits.
    recorded: bool,
    data: Rc<Data>,
    _tick: gpui::Task<()>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<WriteEvent> for WriteView {}

impl WriteView {
    pub fn new(data: Rc<Data>, essay: Essay, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            EditorView::new(
                essay.body.clone(),
                EditorStyle::write(),
                Viewport::Typewriter,
                cx,
            )
        });

        let mut subscriptions = vec![cx.subscribe(&editor, Self::on_edited)];
        // Quitting with cmd-Q is leaving too: the text is saved and the
        // session counts.
        subscriptions.push(cx.on_app_quit(|this: &mut Self, cx| {
            this.finish(cx);
            async {}
        }));

        let tick = cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(TICK).await;
            if this.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
        });

        WriteView {
            autosave: Autosave::new(data.clone(), editor.clone(), essay),
            editor,
            started_at: now(),
            started: Instant::now(),
            recorded: false,
            data,
            _tick: tick,
            _subscriptions: subscriptions,
        }
    }

    /// The essay as it stands on disk, for the router to move on.
    pub fn essay(&self) -> &Essay {
        self.autosave.essay()
    }

    /// Write the text out now. The router calls this before a switch, so the
    /// words are safe before the state moves (design.md, D4).
    pub fn save(&mut self, cx: &mut Context<Self>) -> Result<()> {
        self.autosave.flush(cx)
    }

    /// Everything leaving Write mode owes the disk: the text, then the record.
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        let _ = self.save(cx);
        self.record_session();
    }

    /// The stretch of writing ends here. Called once — on the way to Today, on
    /// a switch the router carried out, or on quit.
    pub fn record_session(&mut self) {
        if self.recorded {
            return;
        }
        self.recorded = true;

        let elapsed = self.started.elapsed();
        if elapsed < SHORTEST_SESSION {
            return;
        }
        let session = Session {
            essay_slug: self.essay().slug.clone(),
            started_at: self.started_at,
            duration_min: (elapsed.as_secs() / 60) as u32,
        };
        if let Err(error) = self.data.sessions.append(&session) {
            log::error!("could not record the session: {error}");
        }
    }

    /// A switch the router could not carry out, said in the same quiet corner
    /// as save trouble (write-mode spec).
    pub fn complain(&mut self, trouble: impl Into<String>, cx: &mut Context<Self>) {
        self.autosave.complain(trouble, cx);
    }

    fn on_edited(&mut self, _: Entity<EditorView>, _: &Edited, cx: &mut Context<Self>) {
        self.autosave.edited(cx, |this| &mut this.autosave);
    }

    fn leave(&mut self, _: &Leave, _: &mut Window, cx: &mut Context<Self>) {
        // Mid-composition, Escape belongs to the IME (write-mode spec).
        if self.editor.read(cx).is_composing() {
            return;
        }
        self.finish(cx);
        cx.emit(WriteEvent::Left);
    }

    fn switch(&mut self, _: &Switch, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(WriteEvent::Switch);
    }

    /// The corner indicator: elapsed minutes, and a gentler colour and a word
    /// once the session is done. No modal, no sound, no interruption.
    fn indicator(&self) -> (SharedString, u32) {
        let elapsed = self.started.elapsed();
        let minutes = elapsed.as_secs() / 60;
        if elapsed >= SESSION_TARGET {
            (
                format!("{minutes} min · session done").into(),
                theme::write::INDICATOR_DONE,
            )
        } else {
            (format!("{minutes} min").into(), theme::write::INDICATOR)
        }
    }
}

impl Focusable for WriteView {
    /// There is one thing to focus in Write mode, and it is the text.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.read(cx).focus_handle(cx)
    }
}

impl Render for WriteView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (elapsed, colour) = self.indicator();

        div()
            .key_context(crate::keymap::WRITE)
            .relative()
            .size_full()
            .bg(rgb(theme::write::BACKGROUND))
            .on_action(cx.listener(Self::leave))
            .on_action(cx.listener(Self::switch))
            .child(self.editor.clone())
            .child(
                // The way across, named for the room it leads to (design.md,
                // D3). Text, not a toolbar.
                div()
                    .id("switch")
                    .absolute()
                    .top(px(18.))
                    .right(px(22.))
                    .text_size(px(theme::SMALL_SIZE))
                    .text_color(rgb(theme::write::SWITCH))
                    .cursor_pointer()
                    .hover(|style| style.text_color(rgb(theme::write::SWITCH_HOVER)))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(WriteEvent::Switch)))
                    .child("Editing"),
            )
            .child(
                div()
                    .absolute()
                    .bottom(px(18.))
                    .right(px(22.))
                    .text_size(px(theme::SMALL_SIZE))
                    .text_color(rgb(colour))
                    .child(elapsed),
            )
            .children(self.autosave.trouble().map(|text| {
                div()
                    .absolute()
                    .bottom(px(18.))
                    .left(px(22.))
                    .text_size(px(theme::SMALL_SIZE))
                    .text_color(rgb(theme::ALARM))
                    .child(text.to_string())
            }))
    }
}
