# install-and-summon — Proposal

## Why

esse is a `cargo run` away from a writer's session, which is fine for the
person building it and wrong for the person using it. There is no app to put
in /Applications, nothing that survives a reboot, and no way to reach the
spark line without first finding a window. The five-second promise of the
spark box holds only once you are already in front of esse.

This serves beginner problem #2 ("cannot start"): the cost of beginning drops
to one key pressed from inside whatever you were doing — a browser, a chat, a
terminal — with no hunting for a window and no launcher. It serves problem #1
in the same motion, because a spark is worth capturing only while it is still
in your head.

Stage 6 deliberately ruled a global hotkey out ("the budget is tried first")
and the budget was met: ~0.15 s warm, ~0.5–0.8 s cold. That measurement
answered "is launching fast enough" — it never answered "how do I get there
from another app", which is what this change is about. A resident app makes
the summon instant, which is a different promise from a fast launch.

## What Changes

- **esse ships as a macOS application bundle.** `make install` builds a
  release binary, assembles `Esse.app` around it, and puts it in
  /Applications. A person installs it once and never types `cargo` again.
- **The app stays resident.** Closing the window puts esse away instead of
  quitting it: the essay is saved exactly as it is on any other way out, and
  the app waits. `cmd-q` still quits for real.
- **One key summons it from anywhere** — `ctrl-alt-e` by default, registered
  system-wide. Pressed from another app it brings esse forward, focused and
  ready to type; pressed while esse is in front it puts it away again. The
  combination can be named at install time with `ESSE_HOTKEY`, because a
  system-wide key can collide with anything else on the machine.
- **No Dock icon, no menu bar.** A tool that lives behind one key is a tool
  that stays out of the way; `cmd-tab` and the Dock are not where it is
  reached from.
- **It can start with the machine.** `make login-item` installs a launch agent
  that starts esse at login with no window on screen, so the key works from the
  first minute of the day.

## Capabilities

### New Capabilities

- `summon-from-anywhere`: esse as an installed, resident background app — the
  bundle, the system-wide summon key, hiding instead of quitting, and starting
  without a window.

### Modified Capabilities

- `write-mode`: the autosave requirement covers being put away by the summon
  key, not only leaving, quitting and closing the window.
- `edit-mode`: the same.

## Non-goals

- No in-app settings screen for the key, no recorder UI, no preferences
  window. The combination is a fact of the installation, chosen once — the
  environment variable belongs to the launch agent, not to a settings panel.
- No menu-bar icon, no tray, no status item, no notifications. Nothing that
  turns a writing tool into something that watches you.
- No quick-capture panel separate from the app: the summon key opens esse
  itself, on the screen it was left on, and the spark line is already there.
- No auto-update, no code signing or notarisation, no distribution beyond
  "build it and install it yourself".
- No Linux or Windows packaging: the app is macOS-only today because gpui's
  desktop support here is, and pretending otherwise in the tooling would be a
  lie.

## Impact

- `crates/esse-app/src/main.rs` — the resident lifecycle, the activation
  policy, registering the key, and starting hidden.
- `crates/esse-app/src/summon.rs` (new) — the key: parsing the combination,
  registering it, and what a press does.
- `crates/esse-app/src/root.rs` — closing the window becomes putting the app
  away, after the same save that closing already does.
- `Cargo.toml` — two new dependencies: `global-hotkey` for the system-wide
  key, `objc` for the activation policy gpui sets for us and we have to unset.
- `Makefile`, `scripts/bundle.sh`, `resources/` — the bundle, its `Info.plist`,
  its icon, and the launch agent.
- `README.md` — how to install it and how the key behaves.
