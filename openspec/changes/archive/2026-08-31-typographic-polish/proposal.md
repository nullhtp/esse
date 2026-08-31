# typographic-polish — Proposal

## Why

The conveyor is closed and in daily use, and it is set in the macOS system UI
font — gpui's default, never chosen. That is the right default for a tool with
panels and toolbars, and the wrong one for this app: esse has no chrome to
speak of, so its screens are almost entirely text, and the face that text is
set in *is* the interface. A spark list, an essay in progress and a published
essay all currently look like rows in a settings window.

The same gap shows in the details. Colour is doing work that space and weight
should do; the three rooms borrow values from one another, so the completion
panel in Edit mode is built out of Today's ink; gaps are a hundred separate
numbers rather than a rhythm; and the corner controls that name a room ("Shelf",
"Editing", "Finish") sit in the same register as the sentences beside them.

This serves no new beginner problem, and does not pretend to. It is craft
upkeep on two that are already specified: the Write/Edit separation of problem
#3, which is enforced by how unmistakably different the two rooms look, and
problem #5 — a writer is likelier to publish an essay that looked, while it was
being written, like it was worth finishing.

## What Changes

- **The app is set in its own face.** Literata, embedded in the binary at three
  weights plus italics, rather than the system UI sans. It is a serif drawn for
  long-form reading on screen, and — the constraint that decided it — it covers
  Cyrillic, which essays here need and which the first face considered did not.
- **One type scale, one spacing rhythm.** Six sizes and five gaps in
  `theme.rs`, used everywhere, replacing the ad-hoc numbers each screen grew.
- **Each room gets a complete palette.** Today and the Shelf are paper, Write is
  that paper at night, Edit is daylight on it — each with its own ink, muted
  ink, rules and buttons. No room borrows a colour from another, which is what
  stops the rooms slowly converging.
- **One accent, and it is the caret.** The interface blue is gone; the only
  saturated colour in the app marks where the writing is happening.
- **A label register.** The corner controls and column headings become small
  letterspaced capitals — they name a place, so they stop competing with the
  sentences that say something.
- **The completion panel asks its question out loud** at reading size, instead
  of captioning it in 13px grey.

## Capabilities

### New Capabilities

- `visual-design`: the appearance rules the app is held to — one embedded
  family, one scale, a complete palette per room, one accent, and the label
  register. Written down because these are the decisions that erode quietly:
  the reason to record them is so the next change cannot borrow Today's ink for
  an Edit-mode panel without noticing.

### Modified Capabilities

None. Every screen behaves exactly as it did; this change is what they look
like, not what they do.

## Non-goals

- **No settings, no font picker, no themes.** Anti-features list, unchanged:
  the appearance is chosen once, by the author, in code.
- No change to any screen's behaviour, routing, keyboard map, or copy — beyond
  one word in a Today notice ("in the list" → "from the list").
- No change to the WIP = 1 invariant, the five mechanics, or the pipeline.
- No dark mode, no system-appearance following. Write mode is dark because
  drafting is dark, not because the OS is.
- No markdown-lite extensions; the editor renders exactly what it did.
- No second optical size, no display cut for headings. One optical size for the
  whole app until dogfooding says otherwise.

## Impact

- `resources/fonts/` — five embedded faces (~1.3 MB) plus the OFL licence and a
  note on provenance; `scripts/fonts.py` cuts them from upstream again.
- `crates/esse-app/src/fonts.rs` — new: the faces, registered at startup, and
  the tests that hold the family naming to what gpui's font matching needs.
- `crates/esse-app/src/theme.rs` — the scale, the rhythm, the three palettes,
  and the `label` helper.
- Every screen module, `line_input.rs` and `editor/element.rs` — the values
  applied. No logic touched.
- `Cargo.toml` — `font-kit` as a dev-dependency only, at gpui's own pin, for
  the font tests. No new runtime dependency.
- Binary grows from ~10.6 MB to ~12 MB; startup is unchanged (201 ms to first
  frame, debug).
