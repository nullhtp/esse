//! What the screen leaves usable.
//!
//! The rooms take the whole display, borderlessly (fullscreen-rooms design.md,
//! D1), so a room's top edge is the screen's own — and on a laptop with a
//! camera housing the top of the screen is not a place to draw. macOS reports
//! the height of that band as the screen's top safe-area inset: 32pt on the
//! 14-inch MacBook Pro this is written on, measured with a probe rather than
//! taken from the documentation, and 0 on a machine without a notch.
//!
//! gpui fills a safe area on iOS and on Android and leaves it empty on macOS,
//! so this is one more thing asked of AppKit directly (summon.rs).

use gpui::{px, Pixels};
use objc2_app_kit::NSScreen;
use objc2_foundation::MainThreadMarker;

/// The strip along the top of the screen that nothing may be drawn in.
///
/// `mainScreen` is the screen the key window is on rather than the built-in
/// one, so a room dragged to a second display is told that display's answer.
/// The rooms are the only callers, and a room is only ever entered fullscreen:
/// the window itself cannot be asked, because it answers a frame late (the
/// same lag `root.rs` keeps its own flag for).
pub fn top_strip() -> Pixels {
    // Off the main thread there is no screen to ask; nothing draws from there
    // either, so the honest answer is no strip at all.
    let Some(main_thread) = MainThreadMarker::new() else {
        return px(0.);
    };
    NSScreen::mainScreen(main_thread)
        .map(|screen| px(screen.safeAreaInsets().top as f32))
        .unwrap_or(px(0.))
}
