//! Where the document sits in the window.
//!
//! Layout always runs from an *anchor* — one paragraph, and the y its first row
//! starts at — outwards until the viewport is covered. What differs between the
//! two modes is only where that anchor comes from (design.md, D5):
//!
//! * [`Viewport::Typewriter`] derives it from the caret, every frame. The view
//!   has no position of its own, which is why the wheel has nothing to move.
//! * [`Viewport::Scrolled`] keeps it in a [`Scroll`] the wheel moves, and only
//!   chases the caret when the caret itself moved.
//!
//! The three rules that keep a scrolled view honest — keep a margin at each
//! end, pull the caret back in, and do not let the document float off either
//! end — are the pure functions below, so they can be reasoned about without a
//! window.

use gpui::{px, Pixels};

/// Which of the two mode's viewports an editor runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewport {
    /// Write mode: the caret's visual row is centred and everything is laid out
    /// from there.
    Typewriter,
    /// Edit mode: the view has a position of its own, and the whole document is
    /// reachable by scrolling.
    Scrolled,
}

/// A scrolled view's position: the paragraph at the top of the page, and how
/// much of it has already gone past that edge.
///
/// A position expressed in paragraphs rather than in pixels from the document's
/// start is what lets scrolling stay as cheap as typing: reaching the anchor
/// never means measuring everything above it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scroll {
    pub line: usize,
    pub offset: Pixels,
}

impl Default for Scroll {
    fn default() -> Self {
        Scroll {
            line: 0,
            offset: px(0.),
        }
    }
}

/// The band of the window the document is allowed to come to rest in: the
/// window, less a page margin at the top and at the bottom.
///
/// Edit mode has the whole display and no title bar (fullscreen-rooms
/// design.md, D1), so the window's top edge is the screen's own. Without a
/// margin the first line of the essay rests hard against it — under the corner
/// controls, and behind the notch strip on the machines that have one.
///
/// Only the resting positions are inset. Text still paints to the window's
/// edges, so scrolling carries a paragraph off the screen rather than cutting
/// it off part way down the page; what the margin buys is the two ends.
///
/// A window too short for two margins keeps a quarter of itself at each end
/// rather than folding inside out.
pub fn page(top: Pixels, bottom: Pixels, margin: Pixels) -> (Pixels, Pixels) {
    let margin = margin.min((bottom - top) / 4.).max(px(0.));
    (top + margin, bottom - margin)
}

/// The smallest shift that brings a row fully into view — positive moves the
/// document down the screen, zero means it is already visible.
///
/// Minimal on purpose: a caret one row past the bottom edge comes to rest *on*
/// the bottom edge, so typing at the end of the screen scrolls a line at a time
/// instead of jumping (edit-mode spec).
pub fn shift_into_view(
    top: Pixels,
    bottom: Pixels,
    view_top: Pixels,
    view_bottom: Pixels,
) -> Pixels {
    if top < view_top {
        view_top - top
    } else if bottom > view_bottom {
        // A row taller than the window cannot fit either way; showing its top
        // beats scrolling its start off to reveal its end.
        (view_bottom - bottom).max(view_top - top)
    } else {
        px(0.)
    }
}

/// The shift that pulls the document back when the scroll ran past an end: the
/// last row never floats above the bottom edge, and the first never sinks below
/// the top one — so a document shorter than the window sits at the top rather
/// than drifting.
///
/// `at_start` and `at_end` say whether the frame reaches the document's first
/// and last paragraph; when it reaches neither there is nothing to clamp to.
pub fn clamp_shift(
    at_start: bool,
    first_top: Pixels,
    at_end: bool,
    last_bottom: Pixels,
    view_top: Pixels,
    view_bottom: Pixels,
) -> Pixels {
    let mut shift = px(0.);
    if at_end && last_bottom < view_bottom {
        shift = view_bottom - last_bottom;
    }
    // The start wins: in a short document both ends want a say, and text
    // belongs at the top of the page.
    if at_start && first_top + shift > view_top {
        shift = view_top - first_top;
    }
    shift
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A window 500px tall, starting 100px down — offset on purpose, so a rule
    /// that quietly assumes the viewport starts at zero fails here.
    const TOP: Pixels = px(100.);
    const BOTTOM: Pixels = px(600.);

    #[test]
    fn the_page_keeps_a_margin_at_each_end() {
        assert_eq!(page(px(0.), px(1000.), px(44.)), (px(44.), px(956.)));
        assert_eq!(page(TOP, BOTTOM, px(44.)), (px(144.), px(556.)));
    }

    #[test]
    fn a_window_too_short_for_two_margins_gives_what_it_has() {
        // The full margin would leave the text a sliver; half the window is
        // where it stops instead — and never less than nothing.
        assert_eq!(page(px(0.), px(100.), px(44.)), (px(25.), px(75.)));
        assert_eq!(page(px(0.), px(0.), px(44.)), (px(0.), px(0.)));
    }

    fn into_view(top: f32, height: f32) -> Pixels {
        shift_into_view(px(top), px(top + height), TOP, BOTTOM)
    }

    #[test]
    fn a_visible_row_is_left_alone() {
        assert_eq!(into_view(100., 30.), px(0.), "flush with the top edge");
        assert_eq!(into_view(300., 30.), px(0.), "in the middle");
        assert_eq!(into_view(570., 30.), px(0.), "flush with the bottom edge");
    }

    #[test]
    fn a_row_past_the_top_comes_down_to_the_edge() {
        assert_eq!(into_view(90., 30.), px(10.), "ten pixels above");
        assert_eq!(into_view(-400., 30.), px(500.), "a screen above");
    }

    #[test]
    fn a_row_past_the_bottom_comes_up_to_the_edge() {
        assert_eq!(into_view(580., 30.), px(-10.), "ten pixels below");
        assert_eq!(into_view(900., 30.), px(-330.), "well below");
    }

    #[test]
    fn a_row_taller_than_the_window_shows_its_top() {
        assert_eq!(into_view(100., 900.), px(0.), "already at the top edge");
        assert_eq!(into_view(50., 900.), px(50.), "brought down to it");
    }

    fn clamp(at_start: bool, top: f32, at_end: bool, bottom: f32) -> Pixels {
        clamp_shift(at_start, px(top), at_end, px(bottom), TOP, BOTTOM)
    }

    #[test]
    fn a_document_that_fills_the_window_scrolls_freely() {
        assert_eq!(clamp(false, -900., false, 1500.), px(0.), "both ends away");
        assert_eq!(clamp(true, 100., false, 1500.), px(0.), "at the top");
        assert_eq!(clamp(false, -900., true, 600.), px(0.), "at the bottom");
    }

    #[test]
    fn scrolling_past_the_end_pulls_the_last_row_back_to_the_bottom() {
        assert_eq!(clamp(false, -900., true, 400.), px(200.));
    }

    #[test]
    fn scrolling_above_the_start_pulls_the_first_row_back_to_the_top() {
        assert_eq!(clamp(true, 250., false, 1500.), px(-150.));
    }

    #[test]
    fn a_document_shorter_than_the_window_sits_at_the_top() {
        // Both ends are on screen: pulling the end down to the bottom edge
        // would leave the text floating, so the start wins.
        assert_eq!(clamp(true, 100., true, 300.), px(0.), "already at the top");
        assert_eq!(clamp(true, 40., true, 240.), px(60.), "pushed down to it");
        assert_eq!(clamp(true, 260., true, 460.), px(-160.), "and back up");
    }
}
