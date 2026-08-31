//! The debounced save both editor modes share (design.md, D7).
//!
//! There is no manual save in esse and no unsaved-changes state: the essay
//! writes itself a second after the last keystroke, and again on every way out
//! — leaving, switching modes, closing the window, quitting. Keeping that in
//! one place is what stops "autosave on pause" from meaning two different
//! things in Write mode and in Edit mode.
//!
//! It holds the essay because saving is the only thing that touches it: the
//! screen above asks for the essay when it needs one to hand on.

use std::rc::Rc;
use std::time::Duration;

use esse_core::model::now;
use esse_core::{Essay, Result};
use gpui::{Context, Entity, Task};

use crate::data::Data;
use crate::editor::EditorView;

/// How long a pause counts as "stopped typing" (design.md, D6).
const AUTOSAVE_DELAY: Duration = Duration::from_millis(1000);

pub struct Autosave {
    data: Rc<Data>,
    editor: Entity<EditorView>,
    essay: Essay,
    /// Revision of the buffer that is on disk.
    saved: u64,
    trouble: Option<String>,
    /// The pending debounced save. Dropping it cancels it, which is how each
    /// keystroke pushes the save back by another second.
    pending: Option<Task<()>>,
}

impl Autosave {
    pub fn new(data: Rc<Data>, editor: Entity<EditorView>, essay: Essay) -> Self {
        Autosave {
            data,
            editor,
            essay,
            saved: 0,
            trouble: None,
            pending: None,
        }
    }

    /// The essay as it stands — after a [`Self::flush`], as it stands on disk.
    pub fn essay(&self) -> &Essay {
        &self.essay
    }

    /// What the disk had to say, if it refused. Shown quietly in the corner;
    /// the text itself is never held hostage to it.
    pub fn trouble(&self) -> Option<&str> {
        self.trouble.as_deref()
    }

    /// Something the screen above wants said in the same quiet corner — a
    /// switch the router could not carry out, for one.
    pub fn complain<V: 'static>(&mut self, trouble: impl Into<String>, cx: &mut Context<V>) {
        self.trouble = Some(trouble.into());
        cx.notify();
    }

    /// Call from the view's `Edited` handler: the save moves to a pause from
    /// now, replacing the one already pending. `autosave` is how the task finds
    /// its way back here through the view that owns it.
    pub fn edited<V: 'static>(
        &mut self,
        cx: &mut Context<V>,
        autosave: fn(&mut V) -> &mut Autosave,
    ) {
        self.pending = Some(cx.spawn(async move |view, cx| {
            cx.background_executor().timer(AUTOSAVE_DELAY).await;
            view.update(cx, |view, cx| autosave(view).save(cx)).ok();
        }));
    }

    /// Write now, cancelling any pending save. Everything on the way out of an
    /// editor mode goes through here.
    pub fn flush<V: 'static>(&mut self, cx: &mut Context<V>) -> Result<()> {
        self.pending = None;
        self.save(cx)
    }

    fn save<V: 'static>(&mut self, cx: &mut Context<V>) -> Result<()> {
        let revision = self.editor.read(cx).revision();
        if revision == self.saved {
            return Ok(());
        }
        self.essay.body = self.editor.read(cx).text().to_string();
        self.essay.updated_at = now();

        let result = self.data.essays.save(&self.essay);
        match &result {
            Ok(()) => {
                self.saved = revision;
                self.trouble = None;
            }
            Err(error) => {
                log::error!("could not save the essay: {error}");
                self.trouble = Some(format!("Не сохранилось: {error}"));
            }
        }
        cx.notify();
        result
    }
}
