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
//!
//! This is also the only room an essay can end in. The finish control opens the
//! completion overlay, because deciding an essay is done is a judgement about
//! the whole text — you make it looking at the text, not at a card on a screen
//! somewhere else (design.md, D1).

use std::path::PathBuf;
use std::rc::Rc;

use esse_core::{Essay, Result};
use gpui::{
    actions, div, prelude::*, px, rgb, rgba, App, ClipboardItem, Context, Entity, EventEmitter,
    FocusHandle, Focusable, SharedString, Subscription, Window,
};

use crate::autosave::Autosave;
use crate::data::Data;
use crate::editor::{Edited, EditorStyle, EditorView, Viewport};
use crate::keymap;
use crate::line_input::LineInput;
use crate::theme;

actions!(
    edit,
    [Leave, Switch, Finish, NextAction, PreviousAction, Activate]
);

/// What Edit mode asks the router for. The text is saved either way.
pub enum EditEvent {
    /// Back to Today. The essay stays in Editing — leaving is not finishing.
    Left,
    /// Across to Write mode. The Editing → Draft transition is the router's,
    /// because it can fail and a failed switch stays put.
    Switch,
    /// End the essay: published, with the link the writer gave if there was
    /// one. Saving and the transition are the router's, for the same reason.
    Publish(Option<String>),
    /// End it the other way. The confirmation has already been given.
    Shelve,
}

/// How far the completion flow has got. Absent — no overlay at all, which is
/// what Edit mode looks like nearly all the time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// The two outcomes, offered.
    Choosing,
    /// Publishing: copy, export, the optional link, and the confirming action.
    Publishing,
    /// Shelving, waiting for the one explicit confirmation (design.md, D2).
    Shelving,
}

/// A place the highlight can stand in a stage, in the order Tab walks them —
/// which is the order the panel reads down the page, and the order the evening
/// actually goes in (design.md, D2, D7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Publish,
    Shelve,
    NotYet,
    Copy,
    Export,
    /// The link field, which is typed into rather than pressed.
    Link,
    Published,
    Back,
    Confirm,
    Decline,
}

impl Stage {
    fn steps(self) -> &'static [Step] {
        match self {
            Stage::Choosing => &[Step::Publish, Step::Shelve, Step::NotYet],
            Stage::Publishing => &[
                Step::Copy,
                Step::Export,
                Step::Link,
                Step::Published,
                Step::Back,
            ],
            Stage::Shelving => &[Step::Confirm, Step::Decline],
        }
    }
}

pub struct EditView {
    autosave: Autosave,
    editor: Entity<EditorView>,
    /// Where the completion flow is, if it has been opened at all.
    finishing: Option<Stage>,
    /// Which of the stage's steps the keyboard is on. Nothing, when a stage has
    /// just opened: ending an essay always takes one deliberate movement first,
    /// so a stray Enter cannot publish (essay-completion spec).
    highlight: Option<usize>,
    /// Focus belongs to the overlay while it is open, so a keystroke meant for
    /// a decision cannot land in the text behind it.
    overlay_focus: FocusHandle,
    /// Where the essay went, when the writer has the link to hand.
    link: Entity<LineInput>,
    /// Something the overlay has to say for itself — copied, exported. Trouble
    /// goes to the same quiet corner as everything else.
    note: Option<SharedString>,
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
            finishing: None,
            highlight: None,
            overlay_focus: cx.focus_handle(),
            link: cx.new(|cx| LineInput::new("Link to the publication — if you have one", cx)),
            note: None,
            _subscriptions: subscriptions,
        }
    }

    /// The essay as it stands on disk, for the router to move on.
    pub fn essay(&self) -> &Essay {
        self.autosave.essay()
    }

    /// Write the text out now. The router calls this before a switch or an
    /// ending, so the words are safe before the state moves (design.md, D4).
    pub fn save(&mut self, cx: &mut Context<Self>) -> Result<()> {
        self.autosave.flush(cx)
    }

    /// All Edit mode owes on the way out is the text — there is no session to
    /// record.
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        let _ = self.save(cx);
    }

    /// A switch or an ending the router could not carry out, said in the same
    /// quiet corner as save trouble (edit-mode spec).
    pub fn complain(&mut self, trouble: impl Into<String>, cx: &mut Context<Self>) {
        self.autosave.complain(trouble, cx);
    }

    fn on_edited(&mut self, _: Entity<EditorView>, _: &Edited, cx: &mut Context<Self>) {
        self.autosave.edited(cx, |this| &mut this.autosave);
    }

    fn leave(&mut self, _: &Leave, window: &mut Window, cx: &mut Context<Self>) {
        // Mid-composition, Escape belongs to the IME.
        if self.editor.read(cx).is_composing() {
            return;
        }
        // With the overlay open, Escape closes it and changes nothing else:
        // the way out of a decision is not the way out of the room.
        if self.finishing.is_some() {
            self.close(window, cx);
            return;
        }
        self.finish(cx);
        cx.emit(EditEvent::Left);
    }

    fn switch(&mut self, _: &Switch, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(EditEvent::Switch);
    }

    // -- the completion flow ---------------------------------------------

    /// The finish control from the keyboard: the very same opening, so the
    /// shortcut cannot drift into meaning something else.
    fn finish_pressed(&mut self, _: &Finish, window: &mut Window, cx: &mut Context<Self>) {
        if self.finishing.is_none() {
            self.show(Stage::Choosing, window, cx);
        }
    }

    /// Move the overlay to a stage, taking focus off the text while it is up.
    /// A stage always opens with nothing under the highlight.
    fn show(&mut self, stage: Stage, window: &mut Window, cx: &mut Context<Self>) {
        self.finishing = Some(stage);
        self.highlight = None;
        self.note = None;
        window.focus(&self.overlay_focus, cx);
        cx.notify();
    }

    /// Dismiss the overlay. Nothing is saved, nothing is moved, nothing is
    /// undone — the essay is in Editing exactly as it was (essay-completion
    /// spec).
    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.finishing = None;
        self.highlight = None;
        self.note = None;
        window.focus(&self.editor.read(cx).focus_handle(cx), cx);
        cx.notify();
    }

    fn next_action(&mut self, _: &NextAction, window: &mut Window, cx: &mut Context<Self>) {
        self.step(1, window, cx);
    }

    fn previous_action(&mut self, _: &PreviousAction, window: &mut Window, cx: &mut Context<Self>) {
        self.step(-1, window, cx);
    }

    /// One step round the stage's steps. From nowhere, forwards lands on the
    /// first and backwards on the last.
    fn step(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(stage) = self.finishing else {
            return;
        };
        let count = stage.steps().len() as isize;
        let next = match self.highlight {
            Some(at) => (at as isize + delta).rem_euclid(count),
            None if delta > 0 => 0,
            None => count - 1,
        } as usize;
        self.highlight = Some(next);

        // The link is typed into, so standing on it means having the caret in
        // it; everything else is pressed, and the overlay keeps the keys.
        let focus = match stage.steps()[next] {
            Step::Link => self.link.read(cx).focus_handle(cx),
            _ => self.overlay_focus.clone(),
        };
        window.focus(&focus, cx);
        cx.notify();
    }

    fn activate(&mut self, _: &Activate, window: &mut Window, cx: &mut Context<Self>) {
        let Some(step) = self
            .finishing
            .zip(self.highlight)
            .and_then(|(stage, at)| stage.steps().get(at).copied())
        else {
            return;
        };
        self.take(step, window, cx);
    }

    /// What a step does — the one place it is written down, so pressing Enter
    /// on it and clicking it cannot come apart.
    fn take(&mut self, step: Step, window: &mut Window, cx: &mut Context<Self>) {
        match step {
            Step::Publish => self.show(Stage::Publishing, window, cx),
            Step::Shelve => self.show(Stage::Shelving, window, cx),
            Step::NotYet => self.close(window, cx),
            Step::Copy => self.copy(cx),
            Step::Export => self.export(cx),
            // Standing on the link field is already all it offers.
            Step::Link => {}
            Step::Published => self.publish(cx),
            Step::Back | Step::Decline => self.show(Stage::Choosing, window, cx),
            Step::Confirm => cx.emit(EditEvent::Shelve),
        }
    }

    /// Whether the completion overlay is up — a small key world of its own, and
    /// so a place with its own help (shortcut-help spec).
    pub fn is_finishing(&self) -> bool {
        self.finishing.is_some()
    }

    /// Whether the highlight is standing on `step`.
    fn is_on(&self, step: Step) -> bool {
        self.finishing
            .zip(self.highlight)
            .and_then(|(stage, at)| stage.steps().get(at).copied())
            == Some(step)
    }

    /// The body, on the clipboard, ready to be pasted into a blog.
    ///
    /// The text is written out first: what gets handed over is the essay as it
    /// stands on disk, so a copy can never quietly be a copy of something else.
    /// A disk that refuses says so in the corner, and nothing is copied.
    fn copy(&mut self, cx: &mut Context<Self>) {
        if self.save(cx).is_err() {
            return;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(self.essay().body_markdown()));
        self.note = Some("Copied".into());
        cx.notify();
    }

    /// The body, in a file of the writer's choosing — `<slug>.md`, offered in
    /// the native save dialog. Dismissing the dialog writes nothing.
    fn export(&mut self, cx: &mut Context<Self>) {
        if self.save(cx).is_err() {
            return;
        }
        let markdown = self.essay().body_markdown();
        let name = format!("{}.md", self.essay().slug);
        let directory = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let chosen = cx.prompt_for_new_path(&directory, Some(&name));
        cx.spawn(async move |this, cx| {
            let told = match chosen.await {
                Ok(Ok(Some(path))) => match std::fs::write(&path, &markdown) {
                    Ok(()) => Ok(format!("Saved: {}", path.display())),
                    Err(error) => Err(format!("Did not save to the file: {error}")),
                },
                // The dialog was dismissed: no file, and the panel stays open.
                Ok(Ok(None)) => return,
                Ok(Err(error)) => Err(format!("The save dialog did not open: {error}")),
                // The dialog went away with the window; there is nobody to tell.
                Err(_) => return,
            };
            this.update(cx, |this, cx| match told {
                Ok(note) => {
                    this.note = Some(note.into());
                    cx.notify();
                }
                Err(trouble) => {
                    log::error!("{trouble}");
                    this.complain(trouble, cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// Confirmed: publish, with the link if one was typed. The rest is the
    /// router's — saving, the transition, and the way back to Today.
    fn publish(&mut self, cx: &mut Context<Self>) {
        let link = self.link.read(cx).text().trim().to_string();
        cx.emit(EditEvent::Publish((!link.is_empty()).then_some(link)));
    }

    // -- the overlay ------------------------------------------------------

    /// The panel, on its sheet of paper, over the text it is about.
    fn overlay(&self, panel: gpui::AnyElement) -> impl IntoElement {
        div()
            .id("finishing")
            .key_context(keymap::FINISHING)
            .track_focus(&self.overlay_focus)
            // The decision is in front of the text: a click on the veil is not
            // a click into the essay behind it.
            .occlude()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(theme::edit::VEIL))
            .child(
                div()
                    .w(px(420.))
                    .max_w_full()
                    .p(px(26.))
                    .rounded(px(10.))
                    .bg(rgb(theme::edit::PANEL))
                    .border_1()
                    .border_color(rgb(theme::edit::PANEL_BORDER))
                    .text_color(rgb(theme::edit::INK))
                    .child(panel)
                    .children(self.note.clone().map(|note| {
                        div()
                            .pt(px(14.))
                            .text_size(px(theme::SMALL_SIZE))
                            .text_color(rgb(theme::MUTED))
                            .child(note)
                    })),
            )
    }

    fn choosing(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(title("Is the essay finished?"))
            .child(self.action("publish", "Publish", true, Step::Publish, cx))
            .child(self.action("shelve", "Shelve", false, Step::Shelve, cx))
            .child(self.cancel("not-yet", "Not yet", Step::NotYet, cx))
    }

    /// Copy and export come before the confirming action, because that is the
    /// order the evening actually goes in: copy the text out, post it, paste
    /// the link back, and only then say it is published (design.md, D2). Tab
    /// walks them in that order too.
    fn publishing(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(title("Publish"))
            .child(self.action("copy", "Copy as markdown", false, Step::Copy, cx))
            .child(self.action("export", "Save to a file…", false, Step::Export, cx))
            .child(div().pt(px(6.)).child(self.link.clone()))
            .child(self.action("published", "Published", true, Step::Published, cx))
            .child(self.cancel("back", "Back", Step::Back, cx))
    }

    /// The one explicit step in front of shelving. Nothing comes back out of
    /// the drawer, so the writer is told so before the door shuts.
    fn shelving(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(title("Shelve this essay?"))
            .child(
                div()
                    .pb(px(4.))
                    .text_size(px(theme::SMALL_SIZE))
                    .text_color(rgb(theme::MUTED))
                    .child("The essay stays on the shelf, but it cannot come back into work."),
            )
            .child(self.action("shelve-confirm", "Yes, shelve it", true, Step::Confirm, cx))
            .child(self.cancel("shelve-decline", "No", Step::Decline, cx))
    }

    /// A thing the panel does. The confirming action carries the ink;
    /// everything else is a quiet face on paper. Under the pointer or under the
    /// highlight it lights the same way — there is one way to be "here".
    fn action(
        &self,
        id: &'static str,
        label: &'static str,
        confirming: bool,
        step: Step,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let button = div()
            .id(id)
            .flex()
            .items_center()
            .justify_center()
            .w_full()
            .py(px(11.))
            .rounded(px(7.))
            .text_size(px(theme::BODY_SIZE))
            .cursor_pointer()
            .child(label)
            .on_click(cx.listener(move |this, _, window, cx| this.take(step, window, cx)));

        let here = self.is_on(step);
        if confirming {
            button
                .bg(rgb(if here {
                    theme::INK_HOVER
                } else {
                    theme::INK
                }))
                .text_color(rgb(theme::BACKGROUND))
                .hover(|style| style.bg(rgb(theme::INK_HOVER)))
        } else {
            button
                .bg(rgb(if here {
                    theme::edit::ACTION_HOVER
                } else {
                    theme::edit::ACTION
                }))
                .hover(|style| style.bg(rgb(theme::edit::ACTION_HOVER)))
        }
    }

    /// The way back out of a stage: a line of text, not a third button.
    fn cancel(
        &self,
        id: &'static str,
        label: &'static str,
        step: Step,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        div()
            .id(id)
            .pt(px(4.))
            .text_size(px(theme::SMALL_SIZE))
            .text_color(rgb(if self.is_on(step) {
                theme::edit::INK
            } else {
                theme::MUTED
            }))
            .cursor_pointer()
            .hover(|style| style.text_color(rgb(theme::edit::INK)))
            .child(label)
            .on_click(cx.listener(move |this, _, window, cx| this.take(step, window, cx)))
    }
}

fn title(text: &'static str) -> impl IntoElement {
    div()
        .pb(px(6.))
        .text_size(px(theme::SMALL_SIZE))
        .text_color(rgb(theme::MUTED))
        .child(text)
}

impl Focusable for EditView {
    /// There is one thing to focus in Edit mode too, and it is the text.
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.editor.read(cx).focus_handle(cx)
    }
}

impl Render for EditView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let overlay = match self.finishing {
            Some(Stage::Choosing) => Some(self.overlay(self.choosing(cx).into_any_element())),
            Some(Stage::Publishing) => Some(self.overlay(self.publishing(cx).into_any_element())),
            Some(Stage::Shelving) => Some(self.overlay(self.shelving(cx).into_any_element())),
            None => None,
        };

        div()
            .key_context(keymap::EDIT)
            .relative()
            .size_full()
            .bg(rgb(theme::edit::BACKGROUND))
            .on_action(cx.listener(Self::leave))
            .on_action(cx.listener(Self::switch))
            .on_action(cx.listener(Self::finish_pressed))
            .on_action(cx.listener(Self::next_action))
            .on_action(cx.listener(Self::previous_action))
            .on_action(cx.listener(Self::activate))
            .child(self.editor.clone())
            .child(
                // The two ways on, in the same quiet corner: across to the
                // other room, or out of the pipeline altogether (design.md,
                // D3, D2).
                div()
                    .absolute()
                    .top(px(18.))
                    .right(px(22.))
                    .flex()
                    .gap(px(20.))
                    .text_size(px(theme::SMALL_SIZE))
                    .text_color(rgb(theme::edit::SWITCH))
                    .child(
                        div()
                            .id("finish")
                            .cursor_pointer()
                            .hover(|style| style.text_color(rgb(theme::edit::SWITCH_HOVER)))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.show(Stage::Choosing, window, cx)
                            }))
                            .child("Finish"),
                    )
                    .child(
                        div()
                            .id("switch")
                            .cursor_pointer()
                            .hover(|style| style.text_color(rgb(theme::edit::SWITCH_HOVER)))
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(EditEvent::Switch)))
                            .child("Writing"),
                    ),
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
            .children(overlay)
    }
}
