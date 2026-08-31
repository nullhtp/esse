# fullscreen-rooms — Proposal

## Why

Both editor rooms are specified as fullscreen, the code asks for fullscreen on
the way in, and the window has never once gone fullscreen. Write mode is
written in a 680×720 window with a title bar over it, on a desktop full of
other windows.

The cause is a collision between two things that were each decided correctly.
esse is an Accessory application — no Dock icon, no menu bar, reached by one
system-wide key (summon-from-anywhere). AppKit refuses `toggleFullScreen:` for
an Accessory application: the call returns, nothing is logged, and the window
keeps the size it had. Measured, not guessed — a probe that opens esse's own
window with esse's own activation policy reports `fullscreen=false` and
680×752 after the toggle, and 1512×982 at the screen's origin after the
borderless one.

This serves beginner problem #3 ("drafting and editing get mixed"), which the
two rooms exist to answer. The rooms are supposed to be felt as places you
enter: a room the size of a chat window, with the desktop visible around it,
is a panel. The mechanic was specified, built, and then quietly not delivered.

## What Changes

- **The rooms take the whole screen, borderlessly.** Entering Write or Edit
  mode puts the window over the entire display — no title bar, and the menu bar
  and the Dock get out of the way — and leaving gives back exactly the window
  that was there before.
- **No second Space.** The rooms are not a macOS fullscreen Space, and not only
  because AppKit will not give this app one: a Space would mean the summon key
  drags the whole desktop somewhere else to show you a paragraph. The writing
  stays where the writer is standing.
- **The spec stops promising native fullscreen.** Both room specs said
  "SHALL request native fullscreen". That is the one thing this app cannot do,
  so it is replaced by what the rooms actually owe the writer: the whole screen.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `write-mode`: the fullscreen requirement says the whole screen rather than
  the platform's fullscreen, and says the room must not become a Space.
- `edit-mode`: the same, for the other room.

## Non-goals

- **No new setting.** Whether the rooms fill the screen is not a preference;
  anti-features list, unchanged.
- No change to Today or the Shelf. They stay plain window screens, and entering
  or leaving them still leaves the window exactly as it is (shelf-screen spec).
- No change to what the rooms contain, their palettes, their keys, or their
  routing — this is the size of the room, not what is in it.
- No layout inset for the notch strip. The corner controls sit 22px down;
  whether that wants a safe area is a question for looking at it, not for this
  change.
- No fullscreen for the summon window itself, and no change to what the summon
  key does.

## Impact

- `crates/esse-app/src/root.rs` — `open_editor` and `leave_editor`: the
  borderless toggle in place of the native one, and the flag that remembers
  whether the editor is the reason the window is that size.
- `openspec/specs/write-mode`, `openspec/specs/edit-mode` — the wording of one
  requirement each.
- Nothing else. No new dependency: gpui's `toggle_simple_fullscreen` is in the
  revision already pinned.
