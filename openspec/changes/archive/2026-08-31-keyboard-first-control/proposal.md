# Keyboard-First Control

## Why

Stage 6 made the daily loop keyboard-drivable, but everything off the happy
path still needs the pointer: choosing a spark when writing starts, the whole
completion flow, the Shelf's cards and drawer, and applying markup mid-text.
Reaching for the mouse breaks writing flow (beginner problem #3, "drafting and
editing get mixed" — flow is what Write mode protects) and adds friction to
starting (problem #2, "cannot start"). Dogfooding also disproved the original
"no discoverability" bet in the keyboard-shortcuts spec: shortcuts nobody can
discover don't get used, so the keyboard surface needs one on-demand,
context-dependent help overlay.

## What Changes

- Every operation in the app becomes keyboard-operable, end to end, with no
  pointer required: choosing a spark to start from, navigating the Shelf's
  columns, cards, and shelved drawer, and driving the completion overlay
  (copy, export, link, confirm).
- The editor gains markdown-lite formatting shortcuts: toggle `**bold**` and
  `*italic*` on the selection or the word at the cursor, and toggle `#`
  heading levels on the current line.
- `cmd-h` opens a context-dependent shortcut help overlay listing exactly the
  shortcuts active on the current screen and state; any key dismisses it.
- **BREAKING (spec-level)**: the keyboard-shortcuts requirement "Shortcuts are
  chrome, not a feature" is reversed in part — a discoverability surface now
  exists, but only on demand. No persistent hints, no customization UI: those
  prohibitions stay.

## Capabilities

### New Capabilities

- `shortcut-help`: the `cmd-h` context-dependent help overlay — what it
  lists, when it appears, and how it gets out of the way.

### Modified Capabilities

- `keyboard-shortcuts`: scope widens from "the daily loop" to "every
  operation"; the no-discoverability requirement is rewritten to allow the
  on-demand overlay while still forbidding persistent hints and customization.
- `live-markdown-editing`: adds formatting shortcuts that edit the markers
  (`**`, `*`, `#`) through the same buffer operations as typing, including
  undo as a single step.
- `start-from-spark`: choosing a spark when the Write action finds the slot
  free becomes keyboard-navigable (move the highlight, confirm, cancel).
- `essay-completion`: the finish control gets a shortcut, and the completion
  overlay becomes fully keyboard-operable through all its stages.
- `shelf-screen`: the three columns, the in-progress card, and the shelved
  drawer become keyboard-navigable.

## Non-goals

- No keybinding customization, settings screen, or keymap file — bindings are
  fixed (anti-feature: settings and customization at launch).
- No persistent shortcut hints layered onto any screen; discoverability lives
  only behind `cmd-h`.
- No menus or toolbars (anti-feature: formatting toolbars) — formatting
  shortcuts edit markers in place, they add no visible chrome.
- No vim mode, chords, or leader keys.
- No new markdown syntax: the shortcuts only toggle what markdown-lite
  already renders.

## Impact

- `crates/esse-app/src/main.rs` — new bindings per context (Today, Shelf,
  Edit, editor); `cmd-h` bound in every context.
- `crates/esse-app/src/editor/` — buffer-level toggle operations for bold /
  italic / heading; new editor bindings (`cmd-b`, `cmd-i`, `cmd-1..3`).
- `crates/esse-app/src/today.rs` — keyboard selection in the spark-choice
  list.
- `crates/esse-app/src/shelf.rs` — focus/highlight model over columns, cards,
  and the drawer.
- `crates/esse-app/src/edit.rs` — shortcut to open the finish control;
  keyboard traversal of the completion overlay's actions and link field.
- New module for the help overlay, rendered above whichever screen is active.
- No storage, lifecycle, or dependency changes; `markdown-lite` is untouched
  (toggles are buffer edits, the parser already renders the result).
