//! Edit mode: the room the essay is read whole and cut about in.
//!
//! The visual and behavioural opposite of Write mode, and deliberately so: light
//! where that is dark, the whole text at full strength where that fades what is
//! already written, free to scroll where that pins the caret to the middle of
//! the screen. Same editor underneath, same autosave; what differs is the two
//! things that make editing a different job from drafting.
//!
//! No session runs here. Sessions are a Write-mode idea, and time spent editing
//! is not time spent drafting (edit-mode spec).

use std::rc::Rc;

use esse_core::{Essay, Result};
use gpui::{
    actions, div, prelude::*, px, rgb, App, Context, Entity, EventEmitter, FocusHandle, Focusable,
    Subscription, Window,
};

use crate::autosave::Autosave;
use crate::data::Data;
use crate::editor::{Edited, EditorStyle, EditorView, Viewport};
use crate::theme;

actions!(edit, [Leave, Switch]);

/// What Edit mode asks the router for. The text is saved either way.
pub enum EditEvent {
    /// Back to Today. The essay stays in Editing — leaving is not finishing.
    Left,
    /// Across to Write mode. The Editing → Draft transition is the router's,
    /// because it can fail and a failed switch stays put.
    Switch,
}

pub struct EditView {
    autosave: Autosave,
    editor: Entity<EditorView>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<EditEvent> for EditView {}

impl EditView {
    pub fn new(data: Rc<Data>, essay: Essay, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            EditorView::new(
                essay.body.clone(),
                EditorStyle::edit(),
                Viewport::Scrolled,
                cx,
            )
        });

        let mut subscriptions = vec![cx.subscribe(&editor, Self::on_edited)];
        // Quitting with cmd-Q is leaving too, and leaving never loses text.
        subscriptions.push(cx.on_app_quit(|this: &mut Self, cx| {
            this.finish(cx);
            async {}
        }));

        EditView {
            autosave: Autosave::new(data, editor.clone(), essay),
            editor,
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

    /// All Edit mode owes on the way out is the text — there is no session to
    /// record.
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        let _ = self.save(cx);
    }

    /// A switch the router could not carry out, said in the same quiet corner
    /// as save trouble (edit-mode spec).
    pub fn complain(&mut self, trouble: impl Into<String>, cx: &mut Context<Self>) {
        self.autosave.complain(trouble, cx);
    }

    fn on_edited(&mut self, _: Entity<EditorView>, _: &Edited, cx: &mut Context<Self>) {
        self.autosave.edited(cx, |this| &mut this.autosave);
    }

    fn leave(&mut self, _: &Leave, _: &mut Window, cx: &mut Context<Self>) {
        // Mid-composition, Escape belongs to the IME.
        if self.editor.read(cx).is_composing() {
            return;
        }
        self.finish(cx);
        cx.emit(EditEvent::Left);
    }

    fn switch(&mut self, _: &Switch, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(EditEvent::Switch);
    }
}

impl Focusable for EditView {
    /// There is one thing to focus in Edit mode too, and it is the text.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.read(cx).focus_handle(cx)
    }
}

impl Render for EditView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("Edit")
            .relative()
            .size_full()
            .bg(rgb(theme::edit::BACKGROUND))
            .on_action(cx.listener(Self::leave))
            .on_action(cx.listener(Self::switch))
            .child(self.editor.clone())
            .child(
                // The way back, named for the room it leads to (design.md, D3).
                div()
                    .id("switch")
                    .absolute()
                    .top(px(18.))
                    .right(px(22.))
                    .text_size(px(theme::SMALL_SIZE))
                    .text_color(rgb(theme::edit::SWITCH))
                    .cursor_pointer()
                    .hover(|style| style.text_color(rgb(theme::edit::SWITCH_HOVER)))
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(EditEvent::Switch)))
                    .child("Пишу"),
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
