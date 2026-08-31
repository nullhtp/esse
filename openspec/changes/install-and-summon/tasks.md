# install-and-summon — Tasks

## 1. The key

- [x] 1.1 Add `global-hotkey` and the AppKit bindings (`objc2`,
      `objc2-app-kit`, `objc2-foundation`) to the workspace manifest and to
      `esse-app`.
- [x] 1.2 `summon.rs`: parse a combination in the app's own spelling
      (`ctrl-alt-e`, `cmd-shift-space`) into modifiers and a code, defaulting to
      `ctrl-alt-e` and reading `ESSE_HOTKEY` when it is set (D2).
- [x] 1.3 Tests for the parser: the default, every modifier, letters and digits
      and the named keys, and the refusals — no key, an unknown key, modifiers
      only.
- [x] 1.4 Register the combination on startup, keep the manager alive for the
      life of the app, and log a refusal instead of dying on it (D1).
- [x] 1.5 Deliver presses into gpui: the crate's handler pushes onto a channel
      the app awaits, so nothing re-enters gpui from a Carbon callback.

## 2. Residency

- [x] 2.1 Set the activation policy back to Accessory at the top of the `run`
      callback, where it sticks (D5).
- [x] 2.2 A press toggles: hide when esse is in front, otherwise unhide,
      activate and focus the window (D3).
- [x] 2.3 Flush the essay before hiding, through the same path the window close
      already uses (D4, write-mode and edit-mode specs).
- [x] 2.4 `on_window_should_close` saves, hides and refuses the close; stop
      quitting on the last window closing (D4).
- [x] 2.5 `ESSE_START_HIDDEN=1` builds the window and hides it immediately, so
      login starts nothing on screen (D6).

## 3. The bundle

- [x] 3.1 `resources/Info.plist`: identifier, name, executable, icon,
      `LSUIElement`, minimum system version, high-resolution capable.
- [x] 3.2 `resources/esse.icns` — the icon, and the script that draws it, so it
      can be redrawn rather than only replaced.
- [x] 3.3 `scripts/bundle.sh`: release build, then the bundle assembled around
      it in `target/Esse.app` (D8).
- [x] 3.4 `Makefile`: `app`, `install`, `login-item`, `uninstall`, `run`,
      `test`.
- [x] 3.5 `resources/com.nullhtp.esse.plist`: the launch agent — run at login,
      no keep-alive, `ESSE_START_HIDDEN=1`.

## 4. The docs

- [x] 4.1 README: an Install section — `make install`, `make login-item`, the
      summon key, and how to name another one.
- [x] 4.2 README: the app is a background app with no Dock icon, and `cmd-q` is
      how it ends.
- [x] 4.3 A scenario list for the checks that only a person at the machine can
      make: summon from another app, summon while in front, close the window,
      quit, and a login start.

## Checks by hand

The key and the window are the parts no test can reach. After `make install`,
open Esse.app once and walk these:

1. **Summon.** From a browser or a terminal, press `ctrl-alt-e`: esse is in
   front, the spark line has the keys, typing lands in it.
2. **Away.** Press `ctrl-alt-e` again: esse is gone and the application you
   came from is in front. Nothing flashes, nothing is left behind.
3. **Mid-sentence.** Start writing, type half a line, press the key away and
   back: the text is there, the session minutes have kept running, and
   `~/Documents/Esse/essays/<slug>.md` holds what was typed.
4. **The close button.** Close the window with the red button: esse stays
   running, the key still brings it back with the essay open where it was.
5. **Quitting.** `cmd-q` ends it; the key does nothing until esse is opened
   again.
6. **Login.** `make login-item`, then log out and in: nothing on screen, and
   the first `ctrl-alt-e` is a window ready to type.
7. **No Dock, no menu bar.** While esse is in front, the Dock has no esse icon
   and the menu bar belongs to nobody but the system.
