//! The one place the method is written down.
//!
//! `cmd-h` answers "what can I press here"; `cmd-shift-h` answers "what do I
//! actually do here" — and answers it as an instruction, in order, from the
//! first move to the one that ends the place. The mechanics already enforce
//! the method; nothing said it out loud (writing-guidance spec, design.md D1).
//!
//! Same contract as the shortcut sheet: summoned only, any key dismisses it
//! without acting, and nothing behind it changes. The steps are fixed prose out
//! of CONCEPT.md — they never read the essay (design.md, D2, D3).

use gpui::{
    actions, div, prelude::*, px, rgb, rgba, App, FocusHandle, KeyDownEvent, SharedString, Window,
};

use crate::keymap::{self, Place};
use crate::theme;

actions!(guidance, [Toggle]);

/// What to say about a place: what it is for, then what to do in it, in order.
struct Method {
    /// The place in one line — the point of standing here at all.
    lead: &'static str,
    /// The steps, first move first, ending with what takes the writer out of
    /// this place. Each carries its own reason: a step nobody believes is a
    /// step nobody follows.
    steps: &'static [&'static str],
}

/// The sheet for the place the writer is standing in. `dismiss` is called on
/// any keypress: the guidance context binds nothing, so a key that would have
/// switched modes or typed a letter only closes it (design.md, D2).
pub fn overlay(
    place: Place,
    focus: &FocusHandle,
    dismiss: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let method = method(place);

    div()
        .key_context(keymap::GUIDANCE)
        .track_focus(focus)
        .occlude()
        .on_key_down(dismiss)
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(VEIL))
        .child(
            div()
                .w(px(CARD_WIDTH))
                .max_w_full()
                .p(px(26.))
                .rounded(px(10.))
                .bg(rgb(theme::BACKGROUND))
                .border_1()
                .border_color(rgb(theme::RULE))
                .text_color(rgb(theme::INK))
                .text_size(px(theme::BODY_SIZE))
                .line_height(px(theme::BODY_SIZE * theme::LINE_SPACING))
                .child(
                    div()
                        .w_full()
                        .pb(px(6.))
                        .text_size(px(theme::SMALL_SIZE))
                        .text_color(rgb(theme::MUTED))
                        .child(place.title()),
                )
                .child(div().w_full().pb(px(12.)).child(method.lead))
                .children(steps(method.steps)),
        )
}

/// The steps, numbered. Each is one paragraph of the card's own width: a
/// definite width is what makes the text wrap at all — laid out as a flex row
/// with the number in a gutter, the paragraph's minimum size is its whole
/// unwrapped length, and the sheet runs off the screen instead.
fn steps(steps: &'static [&'static str]) -> Vec<impl IntoElement> {
    steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            div()
                .w_full()
                .py(px(5.))
                .child(SharedString::from(format!("{}. {step}", index + 1)))
        })
        .collect()
}

/// What to say in each place. An exhaustive match, so a new [`Place`] cannot be
/// added without deciding what the method is there (design.md, D3).
fn method(place: Place) -> &'static Method {
    match place {
        Place::Today => &TODAY,
        Place::ChoosingSpark => &CHOOSING_SPARK,
        Place::Write => &WRITE,
        Place::Edit => &EDIT,
        Place::Finishing => &FINISHING,
        Place::Shelf => &SHELF,
    }
}

/// The daily loop: catch sparks between sessions, spend one session writing
/// (CONCEPT.md, mechanics 1 and 4).
static TODAY: Method = Method {
    lead: "Everything begins again here. One spark, one session — the writing habit grows out of that.",
    steps: &[
        "A thought went by — do not let it leave: one line in the spark box and it is yours. A seed, not a vow.",
        "Ready — press Write. The app remembers where you stopped, or asks which spark to start from.",
        "Ahead of you are twenty minutes. Not “write the essay” — just be with the text, and that is enough.",
        "Time is up — leave with a clear head. The session happened, even if the end is still far off.",
        "The day gets a dot below. Missed one — never mind: what counts is not the chain but that you come back.",
    ],
};

/// Choosing is the cheap part; deliberating over it is the expensive one
/// (CONCEPT.md, mechanic 2).
static CHOOSING_SPARK: Method = Method {
    lead: "Your thoughts are already here — one of them has to be picked. The choice starts the work, so keep it light.",
    steps: &[
        "Run down the list and take the one your eye stumbles on. That is the one still alive.",
        "Do not hunt for the most important. A spark becomes important while you write it, not while you pick it.",
        "Do not fear knowing only part of the topic. Nobody knows it — the text is written in order to understand.",
        "The chosen spark becomes an essay, the rest wait their turn. One choice does not empty the box.",
        "Changed your mind — leave without starting. There is no counter of failed attempts here, and never will be.",
    ],
};

/// The mode that exists to make bad text (CONCEPT.md, mechanic 3).
static WRITE: Method = Method {
    lead: "Here you may write badly — and should. The task is not to write well but to write it whole; well comes later.",
    steps: &[
        "Start with any first sentence. It does not have to be good — it will almost certainly be rewritten anyway.",
        "Go forward and do not reread. The text above is dimmed on purpose: what is written must not pull you back.",
        "Stuck — write that down: “stuck, because…”. An honest line always beats an empty one.",
        "No word or fact at hand — leave a gap and run on. Editing is where that gets sorted out, not here.",
        "Do not think about saving: the text goes to disk by itself the moment you stop typing.",
        "The session is over — stop mid-sentence if you like. Draft written whole — you are due in Editing.",
    ],
};

/// The mode where the goal flips (CONCEPT.md, mechanic 3).
static EDIT: Method = Method {
    lead: "Now you can be strict. The whole text is in front of you, and everything superfluous is finally visible.",
    steps: &[
        "First read it all the way through without touching a word. Simply notice where you got bored.",
        "Cut boldly, and cut big. A paragraph that does not work is cured by deletion, not by repair.",
        "Move what is left: the strong parts to the beginning and the end. The middle is read with half an eye.",
        "Close the gaps left in the draft, and only then work the sentences: shorter, sharper, more alive.",
        "Tempted to write a whole new piece — go back to Writing. Do not mix them: these are two different jobs.",
        "The title comes last, once you see what it became. Then finish it: publishing is the next step.",
    ],
};

/// The only two endings, and why the shelved one is not a failure
/// (CONCEPT.md, mechanic 5).
static FINISHING: Method = Method {
    lead: "The most frightening step, and the most important. A text lives only once read — otherwise it stays a draft.",
    steps: &[
        "“Not ready yet” is about fear, not about the text. It will not pass by itself: decide, and hand it over.",
        "Copy it as markdown or save it to a file — only your words go out, without the service lines.",
        "Publish it for real: a blog, a newsletter, a channel. Somewhere the text meets a living person.",
        "Come back and paste the link. Optional — but later it is what shows you all of this was in earnest.",
        "Confirm. The essay joins the row of published ones, and your hands are free for the next.",
        "The text really is wrong — shelve it. A grown-up decision, not a defeat; but it does not come back.",
    ],
};

/// The conveyor seen whole, and the rule that keeps it moving (CONCEPT.md,
/// "The conveyor").
static SHELF: Method = Method {
    lead: "From here you see it all at once: what waits its turn, what is in the works, what reached people.",
    steps: &[
        "On the left the spark box, freshest on top. While it has something in it, “nothing to write about” is over.",
        "In the middle, the essay in progress. Always one: one essay reaches the end more often than five do.",
        "On the right the published ones, with dates and links. That is your progress, not a word count.",
        "Below, the shelved — set aside deliberately. Rereading is allowed, coming back is not, and that is honest.",
        "Drawn to another one — finish the current one first. Free hands are earned here, not handed out.",
    ],
};

/// Wider than the shortcut sheet, because this is prose and prose wants a
/// measure. Narrow enough to still fit the Today window, which is the smallest
/// the app ever gets.
const CARD_WIDTH: f32 = 560.;

/// The same veil as the shortcut sheet: the two are one family, and the work
/// stays legible under both.
const VEIL: u32 = 0x15171baa;

#[cfg(test)]
mod tests {
    use super::*;

    const PLACES: [Place; 6] = [
        Place::Today,
        Place::ChoosingSpark,
        Place::Write,
        Place::Edit,
        Place::Finishing,
        Place::Shelf,
    ];

    /// Every place the writer can stand in gets a whole instruction: what it is
    /// for, and enough steps to actually work through it (writing-guidance
    /// spec).
    #[test]
    fn every_place_is_explained_step_by_step() {
        for place in PLACES {
            let method = method(place);
            assert!(!method.lead.trim().is_empty(), "{place:?} has no lead");
            assert!(
                method.steps.len() >= 3,
                "{place:?} has {} step(s) — too few to be an instruction",
                method.steps.len()
            );
            for step in method.steps {
                assert!(!step.trim().is_empty(), "{place:?} has a blank step");
            }
        }
    }

    /// The sheet has to fit on screen whole — there is no scrolling in it,
    /// because any key dismisses it (design.md, D2). Both dimensions are
    /// capped: how many steps, and how long each one wraps to. The budget is
    /// in characters, counted as characters: what wraps a line is glyphs, and
    /// a step is free to carry a dash or a quote that is more than one byte.
    #[test]
    fn no_place_outgrows_the_sheet() {
        // Two wrapped lines at the card's measure, with the number in front.
        const LONGEST_STEP: usize = 120;

        for place in PLACES {
            let method = method(place);
            assert!(
                method.steps.len() <= 6,
                "{place:?} has {} steps — more than the sheet can show at once",
                method.steps.len()
            );
            assert!(
                method.lead.chars().count() <= LONGEST_STEP,
                "{place:?} opens with a paragraph, not a line"
            );
            for step in method.steps {
                let length = step.chars().count();
                assert!(
                    length <= LONGEST_STEP,
                    "a step in {place:?} is {length} characters — too long to stay on two lines: {step}"
                );
            }
        }
    }

    /// Each place says its own thing: two identical texts would mean the sheet
    /// stopped following where the writer is standing.
    #[test]
    fn every_place_says_its_own_thing() {
        for (index, place) in PLACES.iter().enumerate() {
            for other in &PLACES[index + 1..] {
                assert_ne!(
                    method(*place).lead,
                    method(*other).lead,
                    "{place:?} and {other:?} open the same way"
                );
                assert_ne!(
                    method(*place).steps,
                    method(*other).steps,
                    "{place:?} and {other:?} give the same steps"
                );
            }
        }
    }
}
