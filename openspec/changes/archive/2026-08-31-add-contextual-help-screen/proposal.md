## Why

The app's mechanics embody a writing method — capture sparks as one-line
seeds, pour out a bad draft without looking back, then cut and rearrange, then
publish — but the method itself is written down nowhere the writer can see it.
The mechanics enforce the method; nothing *teaches* it. A beginner standing in
Write mode does not know that going back is discouraged on purpose, and a
beginner in Edit mode does not know the goal has changed from volume to
quality.

This serves beginner problem #2 ("cannot start" — knowing that the draft is
allowed to be bad is what unblocks the first sentence) and problem #3
("drafting and editing get mixed" — the guidance states the rule the two
modes enforce). The spark and finishing texts also touch problems #1 and #5.

## What Changes

- A new on-demand guidance overlay, summoned with `cmd-shift-h` from any
  screen, showing a step-by-step instruction for the place the writer is
  standing: a line on what the place is for, then the steps in order — Today
  (catch a spark, start a session, stop when the stretch is done), choosing a
  spark (scan, take what itches, don't deliberate), Write mode (any first
  sentence, don't reread, leave gaps, stop at the indicator), Edit mode (read
  whole, cut largest-first, rearrange, sentences last, title at the end), the
  completion overlay (copy, publish for real, paste the link, confirm), the
  Shelf (what each column is, the WIP rule, how to free the slot).
- The overlay follows the same rules as the existing shortcut help: visible
  only while summoned, any keypress dismisses it without acting, presentation
  only — nothing behind it changes.
- The instructions are static, written by hand from CONCEPT.md and the
  capability specs, in Russian like the rest of the UI. No AI, no per-essay
  advice.
- `cmd-shift-h` joins the global keys listed in the shortcut-help overlay's
  trailing "everywhere" section.

## Capabilities

### New Capabilities

- `writing-guidance`: the on-demand, context-dependent method overlay — what
  it shows in each place, how it is summoned and dismissed, and that it is
  presentation only.

### Modified Capabilities

- `shortcut-help`: the enumerated global-keys section of the help overlay
  gains the guidance shortcut (`cmd-h`, `cmd-shift-h`, `cmd-q`).

## Non-goals

- Not a writing course, not a tour, not first-run onboarding — the
  first-run-onboarding spec's "this is the only onboarding" requirement is
  untouched; the guidance appears only when summoned.
- No persistent hints layered onto any screen (anti-features list); the
  overlay exists only between two keypresses, like shortcut help.
- No AI-generated or text-aware advice — the instructions are fixed and never
  read the essay (anti-features list).
- No settings, no way to customize or hide the instructions.
- Not a replacement for the shortcut overlay: `cmd-h` keeps listing keys,
  `cmd-shift-h` explains method. The two stay separate surfaces.

## Impact

- `crates/esse-app`: a new guidance module mirroring `help.rs` (overlay,
  veil, dismissal), a `Toggle` action and `cmd-shift-h` binding in
  `keymap.rs` with `Scope::Everywhere`, wiring in `root.rs` alongside the
  existing help overlay, instructions keyed by `keymap::Place`.
- Specs: new `writing-guidance` spec; delta to `shortcut-help` for the
  global-keys enumeration.
- No changes to `esse-core`, storage, or any persisted state.
