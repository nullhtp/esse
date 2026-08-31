//! The one place shortcuts are visible.
//!
//! Summoned with `cmd-h`, gone on the next keypress, and never on screen
//! otherwise: a shortcut nobody can discover does not get used, but a hint
//! layered onto the writing does not get ignored either (shortcut-help spec).
//!
//! The sheet lists the keys of the innermost thing the eyes are on and nothing
//! else, straight out of the keymap table — so it says what is true rather than
//! what was true when it was written (design.md, D1, D3).

use gpui::{
    actions, div, prelude::*, px, rgb, rgba, App, FocusHandle, KeyDownEvent, SharedString, Window,
};

use crate::fonts;
use crate::keymap::{self, Place, Shortcut};
use crate::theme;

actions!(help, [Toggle]);

/// The sheet, ready to be laid over whichever screen is up. `dismiss` is called
/// on any keypress: while help is on screen its context binds nothing, so a key
/// that would have published, switched rooms or typed a letter only closes it
/// (design.md, D2).
pub fn overlay(
    place: Place,
    focus: &FocusHandle,
    dismiss: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .key_context(keymap::HELP)
        .track_focus(focus)
        // Nothing behind the sheet answers the pointer either: help is a glance,
        // not a place to work from.
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
                .w(px(400.))
                .max_w_full()
                .px(px(theme::SPACE_XL - 4.))
                .py(px(theme::SPACE_XL - 6.))
                .rounded(px(theme::RADIUS_PANEL))
                .bg(rgb(theme::BACKGROUND))
                .border_1()
                .border_color(rgb(theme::RULE))
                .text_color(rgb(theme::INK))
                .child(heading(place.title()))
                .children(rows(keymap::shortcuts(place)))
                .child(div().pt(px(theme::SPACE_L)).child(heading("Everywhere")))
                .children(rows(keymap::everywhere())),
        )
}

/// A section's name, in the same quiet register as every other label in the
/// app.
fn heading(text: &'static str) -> impl IntoElement {
    div()
        .pb(px(theme::SPACE_S))
        .text_size(px(theme::LABEL_SIZE))
        .font_weight(fonts::MEDIUM)
        .text_color(rgb(theme::MUTED))
        .child(theme::label(text))
}

/// The rows, with the keys that do the same thing gathered onto one line —
/// Escape and `cmd-l` both leave the Shelf, and saying so twice would read like
/// a mistake.
fn rows<'a>(shortcuts: impl Iterator<Item = &'a Shortcut>) -> Vec<impl IntoElement> {
    let mut lines: Vec<(String, &'static str)> = Vec::new();
    for shortcut in shortcuts {
        match lines.last_mut() {
            Some((keys, label)) if *label == shortcut.label => {
                keys.push(' ');
                keys.push_str(&pretty(shortcut.keys));
            }
            _ => lines.push((pretty(shortcut.keys), shortcut.label)),
        }
    }

    lines
        .into_iter()
        .map(|(keys, label)| {
            div()
                .flex()
                .items_baseline()
                .gap(px(theme::SPACE_M))
                .py(px(theme::SPACE_XS - 1.))
                .text_size(px(theme::BODY_SIZE))
                .child(
                    div()
                        .flex_none()
                        .w(px(96.))
                        .text_color(rgb(theme::MUTED))
                        .child(SharedString::from(keys)),
                )
                .child(div().child(label))
        })
        .collect()
}

/// A keystroke as the keyboard shows it: `cmd-enter` is a picture of two keys,
/// not a word about them.
fn pretty(keys: &str) -> String {
    keys.split('-')
        .map(|part| match part {
            "cmd" => "⌘".to_string(),
            "shift" => "⇧".to_string(),
            "alt" => "⌥".to_string(),
            "ctrl" => "⌃".to_string(),
            "enter" => "⏎".to_string(),
            "escape" => "esc".to_string(),
            "tab" => "⇥".to_string(),
            "up" => "↑".to_string(),
            "down" => "↓".to_string(),
            "left" => "←".to_string(),
            "right" => "→".to_string(),
            other => other.to_uppercase(),
        })
        .collect()
}

/// Thinner than the completion overlay's veil: help is laid over work in
/// progress, and the work should stay legible under it.
const VEIL: u32 = 0x15171baa;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keystrokes_read_as_keys() {
        assert_eq!(pretty("cmd-enter"), "⌘⏎");
        assert_eq!(pretty("escape"), "esc");
        assert_eq!(pretty("cmd-b"), "⌘B");
        assert_eq!(pretty("up"), "↑");
        assert_eq!(pretty("cmd-1"), "⌘1");
    }
}
