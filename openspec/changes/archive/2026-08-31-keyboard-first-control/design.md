# Keyboard-First Control — Design

## Context

Bindings today live in two places: `main.rs` registers per-context
`KeyBinding`s (LineInput, Write, Edit, Today, Shelf) and `editor/mod.rs`
registers the editor's cursor/selection/clipboard set. Everything bound is
keyboard-only chrome for the daily loop; everything else is `on_click`:
spark choice on Today (`today.rs`), the Shelf's spark rows, in-progress card
and drawer toggle (`shelf.rs`), and the completion overlay's actions
(`edit.rs`). The editor buffer (`editor/buffer.rs`) exposes selection, word
movement, and undo-grouped edits — the raw material for formatting toggles.
`markdown-lite` parses `#{1..6} ` headings and `*`/`**` emphasis per line and
is not touched by this change.

Neither the WIP-limit-of-1 invariant nor the Write/Edit mode separation is
affected: every keyboard path triggers the same actions and lifecycle
transitions the pointer paths already trigger, through the same single
enforcement points.

## Goals / Non-Goals

**Goals:**

- One declaration per shortcut: binding, context, and human-readable label
  defined together, consumed by both `bind_keys` and the help overlay, so
  the overlay can never drift from reality.
- Full keyboard traversal of the three screens and both overlays.
- Formatting toggles as ordinary buffer edits: one undo step, cursor kept in
  a sensible place, rendering picked up by the existing parser.

**Non-Goals:**

- Customizable keymaps, keymap files, or a settings surface.
- Persistent hints, menu bars, tooltips showing shortcuts.
- Focus-ring traversal of every pixel (no generic Tab-cycles-everything
  framework); each screen gets a purpose-built highlight model instead.

## Decisions

### D1. A single binding table with labels

A new `keymap.rs` module declares every app-level shortcut as
`(keystroke, action, context, label)`. `main.rs` derives its
`cx.bind_keys(...)` list from the table; the help overlay renders the same
table filtered by the active context. The editor's text-editing bindings
(arrows, clipboard, undo) stay in `editor/mod.rs` and are *not* listed in
help — they are platform conventions, not app vocabulary; the editor's
*formatting* shortcuts and mode/exit keys are in the table.

*Alternative — introspect gpui's keymap at render time*: rejected; gpui has
no stable public enumeration keyed the way the overlay needs, and labels
(Russian, human phrasing) have to live somewhere anyway.

### D2. `cmd-h` everywhere, overlay closes on any key

`cmd-h` is bound in every context and toggles the help overlay for the
current context. esse sets no macOS application menu, so `cmd-h` is not
claimed by the system Hide item; the binding is the user's explicit choice.
While the overlay is up it holds focus; *any* keypress (including `cmd-h`
and Escape) only dismisses it — the dismissed key never also performs its
action, so glancing at help can never publish, switch modes, or type into
the text. The overlay is presentation only: no state changes on open or
close.

*Alternative — keys pass through and act*: rejected; "look up the key, press
it, and it fires while the sheet closes" is nice in theory but makes a wrong
guess destructive.

### D3. Context is what the eyes see

The overlay lists the shortcuts of the innermost active context only: Today
(resting), Today (choosing a spark), Write, Edit, completion overlay, Shelf.
Global keys (`cmd-q`, `cmd-h` itself) render as a short trailing section on
every variant. Six small hand-curated lists beat one clever computed one.

### D4. Formatting toggles are buffer operations

`buffer.rs` gains two operations, each a single undo group:

- `toggle_inline(Bold | Italic)` — with a selection inside one source line,
  wrap it in `**`/`*`, or unwrap if the selection (or the span it sits in)
  is already styled that way, using `markdown-lite`'s parse of that line to
  find span boundaries; with no selection, apply to the word at the cursor
  (word boundaries as in `move_word_*`); selection spanning multiple source
  lines or an empty line: no-op. Cursor/selection land on the same text
  after toggling, markers excluded.
- `set_heading(level 1..=3)` — replace the current line's heading marker
  with `#{level} `, or strip the marker when the line already has exactly
  that level. Cursor keeps its position in the text, clamped to the line.

Bound as `cmd-b`, `cmd-i`, `cmd-1`/`cmd-2`/`cmd-3` in the editor context, so
they work identically in Write and Edit modes. Levels 4–6 remain type-only:
the app's essays do not use them, and three keys is the whole vocabulary.

*Alternative — implement toggling in the view layer*: rejected; buffer-level
operations get undo, autosave, and revision tracking for free and are
testable without a window.

### D5. Spark choice is a modal focus, not a second input mode

Entering the choosing state (Write with a free slot and sparks present)
moves focus from the capture line to the choice list: `up`/`down` move the
highlight (starting at the newest spark), `enter` starts from the
highlighted spark, `escape` returns to resting Today with focus back on the
capture line. Unmodified keys therefore operate the choice while it is open
— this does not break the "typing is capture" rule, because the state is
entered only by an explicit action and left by one key. The
keyboard-shortcuts delta re-words the modifier rule to apply to Today's
resting state.

### D6. Shelf gets a highlight model

The Shelf holds a highlighted position: `left`/`right` move between the
columns that have content, `up`/`down` move within a column, `enter`
activates the highlighted card exactly as a click does (spark → start when
the slot is free; in-progress → continue; published and shelved cards have
no click action, so `enter` does nothing on them), `cmd-d` toggles the
shelved drawer. No text input exists on the Shelf, so unmodified arrows are
safe. The highlight is visible and starts on the first actionable card.

### D7. The completion overlay is a small key world of its own

In Edit mode `cmd-enter` opens the finish control (the same action as
clicking it). Inside the overlay: `left`/`right` (and `tab`) move the
highlight between the visible actions, `enter` activates the highlighted
action, `escape` dismisses (already true). In the publishing stage the link
field is a LineInput; `tab` moves between the field and the actions so the
whole flow — copy, export, paste link, confirm — needs no pointer. Publish
confirmation stays behind an explicit highlighted-enter, never a bare
default-on-open, so a stray `enter` cannot publish.

## Risks / Trade-offs

- [Help table drifts from real bindings] → the table *is* the source
  `bind_keys` consumes; a unit test asserts every context's bindings appear
  in the table with a non-empty label.
- [`cmd-h` habit from other macOS apps (Hide)] → accepted knowingly; esse is
  fullscreen or single-window and hiding it has no workflow. Documented in
  the help overlay itself (the `cmd-h` row).
- [Unwrap logic guesses wrong on nested/adjacent emphasis] → toggling uses
  the parser's span map rather than string scanning; ambiguous cases (cursor
  on a marker, mixed styles inside the selection) resolve to "wrap", which
  is always visible and always one `cmd-z` from undone.
- [Arrow keys in the editor already mean cursor movement, so Shelf/choice
  arrow conventions can't leak there] → the highlight models live in
  `today.rs`/`shelf.rs` view state, keyed to their own focus contexts;
  nothing is added to the editor context beyond formatting keys.
- [Overlay-eats-the-key surprises a user who expects pass-through] → the
  overlay closes on the first key; the second press acts. One extra
  keystroke, zero accidents.

## Open Questions

None blocking. If dogfooding shows heading levels 4–6 are ever wanted from
the keyboard, `cmd-4..6` extend D4 without design changes.
