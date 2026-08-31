//! The key that reaches esse from anywhere.
//!
//! esse stays running with nothing on screen — no Dock icon, no menu bar, no
//! status item — and one system-wide combination brings it forward and puts it
//! away again. That is the whole of the app's presence in the machine between
//! sessions (summon-from-anywhere spec).
//!
//! The key is Carbon's `RegisterEventHotKey` underneath: no accessibility
//! permission, nothing readable but the one combination, and it fires on the
//! physical key whatever the keyboard layout — which matters when half the
//! writing is done in a Russian one (design.md, D1).

use std::env;
use std::str::FromStr;

use futures::channel::mpsc;
use futures::StreamExt;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use esse_core::Setup;
use gpui::{App, WindowHandle};
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_foundation::MainThreadMarker;

use crate::root::RootView;

/// The combination esse holds unless the installation names another one.
/// Free on macOS, free in the editors esse sits next to, and one hand
/// (design.md, D2).
pub const DEFAULT_KEYS: &str = "ctrl-alt-e";

/// Where an installation names a different combination. A collision is
/// personal, so the launch agent gets to settle it — this is not a setting the
/// app owns, and nothing in the app ever writes it.
///
/// When the environment says nothing, the answer given to `esse-setup` does:
/// a combination named there is answered however esse was started, not only on
/// the launches the launch agent makes (guided-install design.md, D5).
const KEYS: &str = "ESSE_HOTKEY";

/// Set by the launch agent, so a login can start esse with nothing on screen.
const START_HIDDEN: &str = "ESSE_START_HIDDEN";

/// Whether this launch should show no window (design.md, D6).
pub fn start_hidden() -> bool {
    match env::var(START_HIDDEN) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// Take esse out of the Dock and out of the menu bar.
///
/// gpui hard-codes `NSApplicationActivationPolicyRegular` in
/// `applicationDidFinishLaunching` and offers no way to ask for anything else;
/// this runs from the callback that same method makes afterwards, which is the
/// one moment the policy can be set back (design.md, D5).
pub fn become_background_app() {
    // On screen when it is wanted, and nowhere else.
    application().setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

/// Undo `App::hide`. gpui can hide an app but has no word for the way back, and
/// activating a hidden application does not unhide it.
fn unhide() {
    application().unhide(None);
}

/// The one `NSApplication`, from the thread gpui runs everything on. Every
/// caller here is already on it — a hotkey press arrives through gpui's own
/// foreground task — so the marker is a fact rather than a hope.
fn application() -> objc2::rc::Retained<NSApplication> {
    let main_thread = MainThreadMarker::new()
        .expect("esse touches AppKit only from the thread gpui draws on");
    NSApplication::sharedApplication(main_thread)
}

/// Register the combination and answer it for the life of the app.
///
/// A refusal — some other application holds the keys — is logged and nothing
/// more: an app that will not start because a shortcut is taken is worse than
/// an app you have to click on (summon-from-anywhere spec).
pub fn listen(window: WindowHandle<RootView>, cx: &mut App) {
    let spelling = combination();
    let hotkey = match parse(&spelling) {
        Ok(hotkey) => hotkey,
        Err(trouble) => {
            log::error!("{KEYS}: {trouble}; esse has no summon key this run");
            return;
        }
    };

    let manager = match GlobalHotKeyManager::new().and_then(|manager| {
        manager.register(hotkey)?;
        Ok(manager)
    }) {
        Ok(manager) => manager,
        Err(error) => {
            log::error!("could not hold {spelling}: {error}; esse has no summon key this run");
            return;
        }
    };

    // The crate's handler runs inside the Carbon event handler, on this very
    // thread: it sends and returns, and the app answers from its own task,
    // rather than gpui being re-entered from a callback (design.md, D1).
    let (presses, mut pressed) = mpsc::unbounded();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.state == HotKeyState::Pressed {
            presses.unbounded_send(()).ok();
        }
    }));

    cx.spawn(async move |cx| {
        // The manager lives as long as this task, which is as long as the app:
        // dropping it unregisters the key.
        let _manager = manager;
        while pressed.next().await.is_some() {
            cx.update(|cx| toggle(window, cx));
        }
    })
    .detach();

    log::info!("summon key: {spelling}");
}

/// The combination this launch holds: the environment first, then whatever was
/// answered at setup, then the one esse was born with.
fn combination() -> String {
    env::var(KEYS)
        .ok()
        .filter(|spelling| !spelling.trim().is_empty())
        .or_else(|| Setup::read().hotkey)
        .unwrap_or_else(|| DEFAULT_KEYS.to_string())
}

/// Whether esse can read a combination, asked from outside the app.
///
/// `esse-setup` has to refuse `cmd-shift-spacebar` before it records it, and
/// the vocabulary of key names is here rather than in a shell script that would
/// drift away from it. This is not a command line for esse; it is the parser,
/// asked out loud (guided-install design.md, D11).
pub fn check_spelling(spelling: &str) -> Result<(), String> {
    parse(spelling).map(|_| ())
}

/// One key, both directions: what is in front of you goes away, and what is
/// away comes forward ready to type (design.md, D3).
fn toggle(window: WindowHandle<RootView>, cx: &mut App) {
    let in_front = window
        .update(cx, |_, window, _| window.is_window_active())
        .unwrap_or(false);

    if in_front {
        put_away(window, cx);
        return;
    }

    unhide();
    cx.activate(true);
    window
        .update(cx, |root, window, cx| {
            window.activate_window();
            root.take_the_keys(window, cx);
        })
        .ok();
}

/// Out of sight, with the text on disk first — being put away is one more way
/// out, and no way out loses a paragraph (write-mode and edit-mode specs).
fn put_away(window: WindowHandle<RootView>, cx: &mut App) {
    window.update(cx, |root, _, cx| root.put_away(cx)).ok();
    cx.hide();
}

/// The app's own spelling of a combination — `ctrl-alt-e`, `cmd-shift-space` —
/// read into the key to register. The same words the help sheet uses, so the
/// machine and the writer are talking about the same thing.
fn parse(spelling: &str) -> Result<HotKey, String> {
    let mut modifiers = Modifiers::empty();
    let mut key: Option<Code> = None;

    for part in spelling.split('-').filter(|part| !part.trim().is_empty()) {
        let part = part.trim().to_ascii_lowercase();
        let modifier = match part.as_str() {
            "cmd" | "command" | "super" => Some(Modifiers::META),
            "ctrl" | "control" => Some(Modifiers::CONTROL),
            "alt" | "opt" | "option" => Some(Modifiers::ALT),
            "shift" => Some(Modifiers::SHIFT),
            _ => None,
        };
        match modifier {
            Some(modifier) => modifiers |= modifier,
            None if key.is_some() => {
                return Err(format!("'{spelling}' names more than one key"))
            }
            None => key = Some(code(&part).ok_or_else(|| format!("'{part}' is not a key"))?),
        }
    }

    match key {
        Some(key) => Ok(HotKey::new(Some(modifiers), key)),
        None => Err(format!("'{spelling}' is modifiers and no key")),
    }
}

/// A key by the name a person writes it: a letter, a digit, or one of the few
/// keys with words instead of glyphs.
fn code(name: &str) -> Option<Code> {
    let spelled = match name {
        "space" => "Space".to_string(),
        "enter" | "return" => "Enter".to_string(),
        "tab" => "Tab".to_string(),
        "escape" | "esc" => "Escape".to_string(),
        letter if letter.len() == 1 && letter.starts_with(|c: char| c.is_ascii_alphabetic()) => {
            format!("Key{}", letter.to_ascii_uppercase())
        }
        digit if digit.len() == 1 && digit.starts_with(|c: char| c.is_ascii_digit()) => {
            format!("Digit{digit}")
        }
        function
            if function.starts_with('f')
                && function[1..].parse::<u8>().is_ok_and(|n| (1..=12).contains(&n)) =>
        {
            format!("F{}", &function[1..])
        }
        _ => return None,
    };
    Code::from_str(&spelled).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(spelling: &str) -> HotKey {
        parse(spelling).unwrap_or_else(|trouble| panic!("{spelling}: {trouble}"))
    }

    /// The combination the app holds when nobody has said otherwise.
    #[test]
    fn the_default_is_a_key_the_system_will_take() {
        let hotkey = keys(DEFAULT_KEYS);

        assert_eq!(hotkey.key, Code::KeyE);
        assert!(hotkey.mods.contains(Modifiers::CONTROL));
        assert!(hotkey.mods.contains(Modifiers::ALT));
        assert!(!hotkey.mods.contains(Modifiers::SHIFT));
    }

    /// The spelling is the app's own, down to the alternative words a person
    /// might reach for.
    #[test]
    fn every_modifier_is_spelled_the_way_the_app_spells_it() {
        // cmd is META on the way in, which the crate stores as SUPER.
        assert_eq!(keys("cmd-space"), keys("command-space"));
        assert_eq!(keys("ctrl-e"), keys("control-e"));
        assert_eq!(keys("alt-e"), keys("option-e"));
        assert_eq!(keys("alt-e"), keys("opt-e"));

        let all = keys("cmd-ctrl-alt-shift-e");
        assert!(all.mods.contains(Modifiers::CONTROL));
        assert!(all.mods.contains(Modifiers::ALT));
        assert!(all.mods.contains(Modifiers::SHIFT));
    }

    #[test]
    fn letters_digits_and_the_named_keys_are_all_reachable() {
        assert_eq!(keys("cmd-e").key, Code::KeyE);
        assert_eq!(keys("cmd-E").key, Code::KeyE);
        assert_eq!(keys("cmd-7").key, Code::Digit7);
        assert_eq!(keys("cmd-space").key, Code::Space);
        assert_eq!(keys("cmd-enter").key, Code::Enter);
        assert_eq!(keys("cmd-return").key, Code::Enter);
        assert_eq!(keys("cmd-escape").key, Code::Escape);
        assert_eq!(keys("cmd-tab").key, Code::Tab);
        assert_eq!(keys("cmd-f5").key, Code::F5);
    }

    /// A combination that cannot be held is worth saying out loud rather than
    /// quietly holding nothing.
    #[test]
    fn a_combination_that_is_not_one_is_refused() {
        assert!(parse("").is_err(), "nothing at all");
        assert!(parse("cmd-shift").is_err(), "modifiers and no key");
        assert!(parse("cmd-квадрат").is_err(), "not a key on any keyboard");
        assert!(parse("cmd-f13").is_err(), "past the function keys");
        assert!(parse("cmd-e-f").is_err(), "two keys");
    }

    /// A key on its own is allowed to be a key on its own — unwise, but the
    /// parser is not the place to argue with the installation.
    #[test]
    fn a_bare_key_is_still_a_key() {
        assert_eq!(keys("f9").key, Code::F9);
        assert!(keys("f9").mods.is_empty());
    }
}
