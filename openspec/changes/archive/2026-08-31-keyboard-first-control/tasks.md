# Keyboard-First Control — Tasks

## 1. Keymap foundation

- [x] 1.1 Create `keymap.rs`: one declaration per app-level shortcut — keystroke, action, key context, human label (Russian) — covering the existing bindings (Write/Shelf/escape/cmd-e/cmd-q) plus the new ones this change adds
- [x] 1.2 Derive `main.rs`'s `cx.bind_keys(...)` list from the keymap table, leaving `editor/mod.rs`'s baseline text-editing set where it is
- [x] 1.3 Add a unit test asserting every context's non-baseline binding appears in the table with a non-empty label (the drift guard from design D1)

## 2. Formatting toggles in the editor

- [x] 2.1 Implement `toggle_inline(Bold | Italic)` in `editor/buffer.rs` using the markdown-lite parse for span boundaries: wrap/unwrap selection within one source line, word-at-cursor with no selection, no-op on empty line or multi-line selection, single undo step, selection preserved over the same text
- [x] 2.2 Unit-test `toggle_inline`: wrap, unwrap, word-at-cursor, ambiguous-resolves-to-wrap, multi-line no-op, one `undo` restores exactly
- [x] 2.3 Implement `set_heading(1..=3)` in `editor/buffer.rs`: replace any existing marker, strip on matching level, cursor keeps its place in the text, single undo step; unit-test the three scenarios plus undo
- [x] 2.4 Bind `cmd-b`, `cmd-i`, `cmd-1`/`cmd-2`/`cmd-3` in the editor context via the keymap table and verify the toggles render live in both Write and Edit modes

## 3. Spark choice by keyboard (Today)

- [x] 3.1 Add a highlight model and focus context to the choosing state in `today.rs`: entering moves focus off the capture line, highlight starts at the newest spark
- [x] 3.2 Bind `up`/`down` to move the highlight, `enter` to start from the highlighted spark through the same event as a click, `escape` to cancel and return focus to the capture line

## 4. Shelf navigation by keyboard

- [x] 4.1 Add the Shelf highlight model in `shelf.rs`: `left`/`right` across columns with content, `up`/`down` within a column, visible highlight starting on the first actionable card
- [x] 4.2 Bind `enter` to activate the highlighted card through the same events as clicks (spark start only when the slot is free; in-progress continue; published/shelved inert) and `cmd-d` to toggle the shelved drawer

## 5. Completion overlay by keyboard (Edit mode)

- [x] 5.1 Bind `cmd-enter` in the Edit context to open the completion overlay through the same path as the finish control, and confirm it does not exist in Write mode
- [x] 5.2 Add highlight traversal inside the overlay: `left`/`right`/`tab` move between the stage's actions, `enter` activates the highlighted one, no action pre-highlighted when a stage opens
- [x] 5.3 Wire the publishing stage's `tab` cycle through the link field and the actions, and verify the full copy → export → link → confirm flow works without a pointer

## 6. Help overlay

- [x] 6.1 Create the help overlay module: renders the keymap table filtered to the innermost active context plus the global section, styled to sit quietly over any screen
- [x] 6.2 Bind `cmd-h` in every context to toggle the overlay, and give the open overlay focus so any keypress dismisses it without performing its action or reaching an input
- [x] 6.3 Verify the overlay is presentation-only (text, cursor, session, focus all unchanged after open/close) and shows the right list in all six states, including choosing and the completion overlay

## 7. Close out

- [x] 7.1 Run the full conveyor — capture, start from spark, write, format, switch, publish — keyboard only, on both the Today and Shelf starting points
- [x] 7.2 Update README.md (Russian) with the keyboard story and `cmd-h`, and run `openspec validate keyboard-first-control`
