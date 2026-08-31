# Design: Write Mode

## Context

Stage 1 delivered the two-crate workspace (`esse-core` GUI-free, `esse-app` on
gpui pinned `v1.17.2`), the domain model with WIP = 1 enforced in
`EssayStore::create`, the on-disk formats (essays with TOML front matter,
sparks and sessions as JSONL), and a Today screen that captures sparks. The
stage-0 prototype proved live markdown-lite rendering and typewriter centring
in gpui and left a reusable `markdown-lite` parser; its view layer was
declared non-carrying because it lacks word wrap, which cuts too deep into
layout, hit-testing, and centring to retrofit.

This change builds the editor for real and the routing around it: spark →
Draft, written in fullscreen Write mode, in recorded sessions. `SessionStore`
and the `spark` front-matter field exist and are waiting to be used.

## Goals / Non-Goals

**Goals:**
- A production editor widget: everything the prototype proved, plus soft word
  wrap and autosave, responsive at ~20k characters.
- `markdown-lite` promoted into the production workspace unchanged.
- Writing starts only through Today's "Write" button: continue the
  in-progress essay or turn a spark into a Draft. No empty-document path.
- Write mode as the concept defines it: fullscreen flow, only the current
  fragment in focus, going back inconvenient but never mechanically blocked.
- Sessions recorded to `sessions.jsonl` with a quiet timer and a soft finish.

**Non-Goals:**
- Edit mode, the Write/Edit switch, and any UI use of `Draft ↔ Editing` —
  stage 3. The essay stays in Draft for this whole change.
- Publishing, shelving, freeing the WIP slot, the Shelf screen — stage 4.
- Rendering session history (dotted calendar) — stage 5; we only write records.
- A hard cursor lock in Write mode — friction only, per the plan's
  recommendation; revisit after dogfooding.

## Decisions

### D1: Promote `markdown-lite` as a crate; adapt the buffer; rewrite the view
`prototypes/markdown-lite` moves to `crates/markdown-lite` (workspace member,
tests included, no code changes beyond the move). The prototype's `Buffer`
(text, selection, undo/redo — 400 framework-free lines with tests) is adapted
into `esse-app` as the editor's document model. The display/view layer is
written fresh around wrapped layout.

*Rationale:* the parser and buffer are framework-free and proven; the view
layer is exactly the part stage 0 declared disposable. Copying the buffer in
rather than making it a third crate keeps the workspace flat — it can be
extracted later if the Edit-mode editor wants to share it.

*Alternative:* rewrite the buffer too. Rejected — nothing about wrap changes
byte-offset text storage; only the offset↔point mapping layer is new.

### D2: Word wrap via gpui's shaped-line layout, mapped to visual lines
Each source line (paragraph) is shaped with a wrap width; the result is a list
of *visual lines*. Cursor up/down, mouse hit-testing, and typewriter centring
all operate on visual lines; horizontal scrolling does not exist. Wrapping is
presentation only — the document text never gains newlines from wrap. The
marker-hiding rule ("raw markers on the cursor line") stays keyed to the
*source* line, so a wrapped paragraph reveals its markers as one unit.

*Rationale:* gpui shapes and wraps text natively (this is Zed's own text
stack); hand-rolling wrap would duplicate it badly. Keying marker visibility
to source lines keeps the parser and the stage-1 spec semantics unchanged.

### D3: A two-screen router in the root view
The root view holds an enum: `Today` or `Write { essay }`. Entering Write mode
requests native fullscreen for the window; leaving restores it and returns to
Today. There is no route that opens the editor without an essay, which is how
"the empty-document screen does not exist" is enforced structurally rather
than by a disabled button.

*Rationale:* two screens do not need a navigation framework. Making the
editor's route carry a loaded essay makes the invariant unrepresentable.

### D4: Starting from a spark is one `esse-core` operation
`esse-core` gains `start_essay_from_spark(&SparkStore, &EssayStore, spark_id)`:
derive a slug from the spark text, `EssayStore::create(slug, Some(text))`
(the WIP = 1 check runs here as always), then remove the spark from
`sparks.jsonl` by atomic rewrite (`SparkStore::remove`, promised in stage 1's
design). Order matters: the essay is created *before* the spark is removed, so
a failure at any point can leave a duplicate seed but can never lose the spark.

*Invariant impact (required note):* **WIP = 1 is untouched and remains in the
storage layer** — this operation goes through `EssayStore::create` and
surfaces its refusal; the UI adds routing on top (the Write button continues
an in-progress essay instead of attempting creation) but has no creation path
of its own. **Write/Edit separation:** this change builds only Write mode;
the essay stays in `Draft`, the `Draft ↔ Editing` transition stays unexercised
by the UI, and the Write screen never offers a whole-text view — the
separation is preserved by not building the second half yet.

### D5: Slugs — transliterate, truncate, suffix
`validate_slug` is ASCII-only and sparks are mostly Russian, so slug
derivation transliterates Cyrillic (small hand-rolled table in `esse-core`,
GOST-style), lowercases, maps everything else non-alphanumeric to `-`,
collapses and trims, and truncates to ~48 chars at a word boundary. If nothing
usable survives, fall back to `essay-<YYYY-MM-DD>`. On `SlugTaken`, retry with
`-2`, `-3`, … This resolves stage 1's open question.

*Alternative:* date-based slugs always. Rejected — `essays/pochemu-esse.md`
beats `essays/essay-2026-08-31.md` in a directory a human reads and syncs.

### D6: Autosave on pause, plus every hard edge
The editor debounces: ~1 s after the last edit, the body is saved through the
existing atomic `EssayStore::save`. Saves also fire on leaving Write mode and
on quit/window close. `updated_at` moves with each save. There is no dirty
flag surfaced to the user and no manual save.

*Rationale:* stage 1's atomic write-temp-then-rename makes frequent saves
safe; 1 s of debounce keeps writes off the keystroke path while making the
window where a crash loses text negligible.

### D7: Write-mode friction — dimming and cursor-following scroll
The current paragraph (source line) renders at full strength; text above it
fades toward the background. Free scrolling is disabled in Write mode — the
viewport follows the cursor (typewriter centring) and wheel input is ignored.
The cursor itself moves anywhere: arrows, clicks, and selection work in the
whole document. Inconvenience comes from the dimming and the missing
overview, not from a lock.

*Rationale:* the plan fixes this explicitly ("only by inconvenience"); a
mechanical block is the post-dogfooding experiment, not the default.

### D8: Sessions — fixed 20-minute target, recorded on exit
Entering Write mode starts a session; a quiet elapsed-time indicator sits in a
corner of the screen. At 20 minutes (a code constant — settings are an
anti-feature; the concept's 15–25 band is a rhythm guide, not a knob) the
indicator changes state gently: no modal, no sound, typing is never
interrupted. Leaving Write mode records `{essay_slug, started_at,
duration_min}` via the existing `SessionStore::append`; sessions under one
minute are not recorded (an accidental peek is not a session).

## Risks / Trade-offs

- [Wrapped layout is the hardest part: offset↔point mapping across shaped
  lines, hit-testing, centring] → it is the *first* editor task, built with
  unit-testable mapping types (the prototype's `display_offset`/
  `source_offset` pattern, extended per visual line), not bolted on last.
- [Autosave from a second running instance could clobber the essay file] →
  single instance still assumed (stage 1's open question stands); atomic
  writes mean the file is never torn, last writer wins. A data-dir lock file
  remains the known fix if dogfooding ever hits this.
- [Native fullscreen transitions on macOS are animated and might make
  entering Write mode feel slow] → fullscreen is requested on entry but the
  screen is usable immediately in the existing window while the transition
  runs; if it grates in practice, dropping to "fills the window" is a
  one-line retreat that does not touch the spec's flow requirements.
- [Dimming tuned wrong: too dark reads as a lock, too light as nothing] →
  it is a theme constant exercised daily by dogfooding; adjusting it is
  cheap and expected.
- [20k-character responsiveness with per-frame shaping] → shape only the
  visual lines in and near the viewport; the prototype already proved
  keystroke latency at this size without wrap, and wrap adds work only
  proportional to visible lines.

## Open Questions

- Does `Buffer` need paragraph-aware movement (option-arrow word jumps are
  enough for Write mode?) — decide while wiring keybindings; not spec-level.
- Whether the session's soft finish should also gently suggest *stopping the
  app* or just show "session done" — start with the indicator change only;
  wording can move after a week of dogfooding.
