//! The router: four screens, and the one rule that matters.
//!
//! An editor mode is reachable only with an essay in hand — both editor arms of
//! [`Screen`] carry a live view, and a view cannot be built without an essay.
//! There is no route that opens the editor without one, which is how "the
//! empty-document screen does not exist" is enforced by the shape of the code
//! rather than by a disabled button (design.md, D3, D1).
//!
//! Today and the Shelf both offer sparks to start from, and both go through the
//! same [`Self::start_from_spark`]: the WIP rule is asked once, of the storage
//! layer, whichever screen the spark was clicked on.

use std::rc::Rc;

use esse_core::{publish, shelve, start_essay_from_spark, Error, Essay, EssayStatus, Result};
use gpui::{div, prelude::*, App, Context, Entity, FocusHandle, Focusable, Subscription, Window};

use crate::data::Data;
use crate::edit::{EditEvent, EditView};
use crate::guidance;
use crate::help;
use crate::keymap::Place;
use crate::shelf::{ShelfEvent, ShelfView};
use crate::today::{TodayEvent, TodayView};
use crate::write::{WriteEvent, WriteView};

/// What is on screen. The editor arms carry their view, so the essay travels
/// with the route rather than beside it.
enum Screen {
    Today,
    Shelf(Entity<ShelfView>),
    Write(Entity<WriteView>),
    Edit(Entity<EditView>),
}

/// The two sheets that can be laid over a screen. Both answer a question about
/// the place the writer is standing in — one which keys work here, the other how
/// to work here — and both are gone on the next keypress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sheet {
    Help,
    Guidance,
}

pub struct RootView {
    data: Rc<Data>,
    today: Entity<TodayView>,
    screen: Screen,
    /// The window state to put back when the editor is left for Today. The
    /// Shelf is a plain screen and never touches it (shelf-screen spec).
    was_fullscreen: bool,
    /// Which sheet is up, if either. One field rather than a flag apiece, so
    /// the two cannot stack (writing-guidance spec, design.md D5). A sheet holds
    /// focus while it is up and gives it straight back — neither changes
    /// anything, ever (shortcut-help spec).
    sheet: Option<Sheet>,
    sheet_focus: FocusHandle,
    /// Who had the keys before a sheet borrowed them.
    was_focused: Option<FocusHandle>,
    _today: Subscription,
    _shelf: Option<Subscription>,
    _editor: Option<Subscription>,
}

impl RootView {
    pub fn new(data: Rc<Data>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = cx.new(|cx| TodayView::new(data.clone(), cx));
        let subscription = cx.subscribe_in(&today, window, Self::on_today);

        // Closing the window is putting esse away, not ending it: the text goes
        // to disk, the window stays, and the summon key brings it all back
        // exactly as it was (summon-from-anywhere spec, design.md D4).
        let this = cx.entity().downgrade();
        window.on_window_should_close(cx, move |_, cx| {
            this.update(cx, |this, cx| this.put_away(cx)).ok();
            cx.hide();
            false
        });

        RootView {
            data,
            today,
            screen: Screen::Today,
            was_fullscreen: false,
            sheet: None,
            sheet_focus: cx.focus_handle(),
            was_focused: None,
            _today: subscription,
            _shelf: None,
            _editor: None,
        }
    }

    // -- going away and coming back -------------------------------------------

    /// Everything esse owes the disk before it goes out of sight: the text, and
    /// only the text. Being put away is not leaving — the session keeps
    /// running, the editor keeps its state, and the way back is one keypress —
    /// so the record of the session is not written here. That belongs to
    /// leaving the room, switching modes, and quitting, as it always did.
    pub fn put_away(&mut self, cx: &mut Context<Self>) {
        match &self.screen {
            Screen::Write(write) => {
                let _ = write.clone().update(cx, |write, cx| write.save(cx));
            }
            Screen::Edit(edit) => {
                let _ = edit.clone().update(cx, |edit, cx| edit.save(cx));
            }
            Screen::Today | Screen::Shelf(_) => {}
        }
    }

    /// Back from being away: whatever the writer was typing into gets the keys
    /// again, so a summon lands ready to type rather than merely visible.
    pub fn take_the_keys(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle(cx), cx);
    }

    // -- the sheets ----------------------------------------------------------

    /// `cmd-h`: which keys work here.
    fn toggle_help(&mut self, _: &help::Toggle, window: &mut Window, cx: &mut Context<Self>) {
        self.toggle(Sheet::Help, window, cx);
    }

    /// `cmd-shift-h`: how to work here — the method the mechanics enforce, said
    /// out loud (writing-guidance spec).
    fn toggle_guidance(
        &mut self,
        _: &guidance::Toggle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.toggle(Sheet::Guidance, window, cx);
    }

    /// Put a sheet up, or take down the one that is. Opening takes the keys so
    /// that the next press only puts the sheet away, and closing hands them back
    /// to exactly whoever had them — the screen behind is untouched
    /// (shortcut-help spec, writing-guidance spec, design.md D2).
    fn toggle(&mut self, sheet: Sheet, window: &mut Window, cx: &mut Context<Self>) {
        if self.sheet == Some(sheet) {
            self.close_sheet(window, cx);
            return;
        }
        // Whatever the writer was typing into keeps the focus it had, unless a
        // sheet is already holding it on their behalf.
        if self.sheet.is_none() {
            self.was_focused = window.focused(cx);
        }
        self.sheet = Some(sheet);
        window.focus(&self.sheet_focus, cx);
        cx.notify();
    }

    fn close_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sheet.take().is_none() {
            return;
        }
        if let Some(focus) = self.was_focused.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }

    /// Where the writer is standing, as the sheets understand it: the innermost
    /// state, not the screen it happens to be drawn on (design.md, D3).
    fn place(&self, cx: &App) -> Place {
        match &self.screen {
            Screen::Today if self.today.read(cx).is_choosing() => Place::ChoosingSpark,
            Screen::Today => Place::Today,
            Screen::Shelf(_) => Place::Shelf,
            Screen::Write(_) => Place::Write,
            Screen::Edit(edit) if edit.read(cx).is_finishing() => Place::Finishing,
            Screen::Edit(_) => Place::Edit,
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
            TodayEvent::Shelf => self.show_shelf(window, cx),
        }
    }

    // -- the shelf -------------------------------------------------------

    /// Open the Shelf. Built fresh each time, so it reads the disk rather than
    /// remembering it, and the window is left exactly as it is — the Shelf is
    /// a plain screen, not a room (shelf-screen spec).
    fn show_shelf(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let shelf = cx.new(|cx| ShelfView::new(self.data.clone(), cx));
        self._shelf = Some(cx.subscribe_in(&shelf, window, Self::on_shelf));
        window.focus(&shelf.read(cx).focus_handle(cx), cx);
        self.screen = Screen::Shelf(shelf);
        cx.notify();
    }

    fn on_shelf(
        &mut self,
        view: &Entity<ShelfView>,
        event: &ShelfEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            ShelfEvent::Left => self.leave_shelf(window, cx),
            ShelfEvent::Start(spark_id) => self.start_from_spark(spark_id, window, cx),
            // The card is only on screen while an essay is in progress; if it
            // has ended since, the shelf was looking at a stale disk and is
            // told to look again.
            ShelfEvent::Continue => match self.data.essays.in_progress() {
                Ok(Some(essay)) => self.open_editor(essay, window, cx),
                Ok(None) => view.update(cx, |shelf, cx| shelf.refresh(cx)),
                Err(error) => {
                    log::error!("could not look for an essay in progress: {error}");
                    let notice = format!("Cannot read the essays: {error}");
                    view.update(cx, |shelf, cx| shelf.say(notice, cx));
                }
            },
        }
    }

    fn leave_shelf(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Screen::Today;
        self._shelf = None;
        self.today.update(cx, |today, cx| today.refresh(window, cx));
        window.focus(&self.today.read(cx).focus_handle(cx), cx);
        cx.notify();
    }

    /// The Write button: continue the essay in progress, or ask for a spark to
    /// start one (start-from-spark spec).
    fn write_pressed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.data.essays.in_progress() {
            Ok(Some(essay)) => self.open_editor(essay, window, cx),
            Ok(None) => {
                let sparks = self.data.sparks.load_all().map(|sparks| sparks.len());
                match sparks {
                    Ok(0) => self.tell(
                        "A spark comes first. Write a thought down, and we start from it.",
                        window,
                        cx,
                    ),
                    Ok(_) => self
                        .today
                        .update(cx, |today, cx| today.offer_sparks(window, cx)),
                    Err(error) => self.tell(format!("Cannot read the sparks: {error}"), window, cx),
                }
            }
            Err(error) => {
                log::error!("could not look for an essay in progress: {error}");
                self.tell(format!("Cannot read the essays: {error}"), window, cx);
            }
        }
    }

    fn start_from_spark(&mut self, spark_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        match start_essay_from_spark(&self.data.sparks, &self.data.essays, spark_id) {
            Ok(essay) => {
                // The spark is gone from the box; the list has to agree.
                self.today.update(cx, |today, cx| today.refresh(window, cx));
                self.open_editor(essay, window, cx);
            }
            // Routing normally makes this impossible — neither screen offers a
            // start while the slot is taken; the refusal is still shown rather
            // than swallowed (start-from-spark spec).
            Err(Error::EssayInProgress { slug, .. }) => self.tell(
                format!("“{slug}” is in progress — finish it before starting another one."),
                window,
                cx,
            ),
            Err(error) => {
                log::error!("could not start an essay: {error}");
                self.tell(format!("Could not start: {error}"), window, cx);
            }
        }
    }

    /// Something the writer has to know, said on the screen they are actually
    /// looking at.
    fn tell(&mut self, notice: impl Into<String>, window: &mut Window, cx: &mut Context<Self>) {
        match &self.screen {
            Screen::Shelf(shelf) => shelf.clone().update(cx, |shelf, cx| shelf.say(notice, cx)),
            _ => self
                .today
                .update(cx, |today, cx| today.say(notice, window, cx)),
        }
    }

    // -- the editor ------------------------------------------------------

    /// Open the editor from Today, in the mode the essay's state calls for: a
    /// Draft is still being written, an essay in Editing is being edited
    /// (start-from-spark spec). The window goes fullscreen and remembers what
    /// to put back (design.md, D2).
    fn open_editor(&mut self, essay: Essay, window: &mut Window, cx: &mut Context<Self>) {
        // Whichever screen the essay was opened from, it is behind us now.
        self._shelf = None;
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
                        let trouble = format!("Did not switch: {error}");
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
                        let trouble = format!("Did not switch: {error}");
                        view.update(cx, |edit, cx| edit.complain(trouble, cx));
                    }
                }
            }
            // The two endings, in the same order as a switch: the words first,
            // then the state. Neither frees the WIP slot by hand — the slot is
            // free because the essay's state stopped counting (design.md, D6).
            EditEvent::Publish(link) => {
                let saved = view.update(cx, |edit, cx| edit.save(cx));
                let essay = view.read(cx).essay().clone();
                let ended = saved.and_then(|()| publish(&self.data.essays, essay, link.as_deref()));
                self.ended(ended, "Did not publish", view, window, cx);
            }
            EditEvent::Shelve => {
                let saved = view.update(cx, |edit, cx| edit.save(cx));
                let essay = view.read(cx).essay().clone();
                let ended = saved.and_then(|()| shelve(&self.data.essays, essay));
                self.ended(ended, "Did not shelve", view, window, cx);
            }
        }
    }

    /// An essay that ended has no room left to be in: the editor closes to
    /// Today with the window put back. One that could not end keeps its room,
    /// its text and its overlay, and is told why (essay-completion spec).
    fn ended(
        &mut self,
        ended: Result<Essay>,
        trouble: &str,
        view: &Entity<EditView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match ended {
            Ok(_) => self.leave_editor(window, cx),
            Err(error) => {
                log::error!("could not finish the essay: {error}");
                let trouble = format!("{trouble}: {error}");
                view.update(cx, |edit, cx| edit.complain(trouble, cx));
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

        self.today.update(cx, |today, cx| today.refresh(window, cx));
        window.focus(&self.today.read(cx).focus_handle(cx), cx);
        cx.notify();
    }
}

impl Focusable for RootView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        if self.sheet.is_some() {
            return self.sheet_focus.clone();
        }
        match &self.screen {
            Screen::Write(write) => write.read(cx).focus_handle(cx),
            Screen::Edit(edit) => edit.read(cx).focus_handle(cx),
            Screen::Shelf(shelf) => shelf.read(cx).focus_handle(cx),
            Screen::Today => self.today.read(cx).focus_handle(cx),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The sheet is a sibling of the screen, not a child of it: that is what
        // leaves it standing in a key context of its own, where none of the
        // screen's shortcuts are bound and any key can only dismiss it
        // (design.md, D2).
        let sheet = self.sheet.map(|sheet| {
            let place = self.place(cx);
            let dismiss = cx.listener(|this, _, window, cx| this.close_sheet(window, cx));
            match sheet {
                Sheet::Help => help::overlay(place, &self.sheet_focus, dismiss).into_any_element(),
                Sheet::Guidance => {
                    guidance::overlay(place, &self.sheet_focus, dismiss).into_any_element()
                }
            }
        });

        div()
            .relative()
            .size_full()
            .on_action(cx.listener(Self::toggle_help))
            .on_action(cx.listener(Self::toggle_guidance))
            .child(match &self.screen {
                Screen::Write(write) => write.clone().into_any_element(),
                Screen::Edit(edit) => edit.clone().into_any_element(),
                Screen::Shelf(shelf) => shelf.clone().into_any_element(),
                Screen::Today => self.today.clone().into_any_element(),
            })
            .children(sheet)
    }
}
