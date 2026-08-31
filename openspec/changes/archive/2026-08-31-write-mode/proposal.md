# Write Mode: the Heart of the App

## Why

Stage 1 left esse as a spark box: ideas accumulate, but nothing can be written.
This change builds stage 2 of the plan — the Write mode — turning the app into
an actual writing tool: spark → draft, written in fullscreen flow, in sessions.

It serves three of the five beginner problems directly:

- **#2 "cannot start"** — an essay starts only from a spark; the empty-page
  screen does not exist, and the routing makes it impossible.
- **#3 "drafting and editing get mixed"** — Write mode shows only the current
  fragment and makes going back inconvenient (by friction, not by a mechanical
  cursor lock), so drafting stays drafting.
- **#4 "gives up after two weeks"** — the session (15–25 minutes, quiet timer,
  soft finish) becomes the recorded unit of habit; every session is a
  completed success regardless of essay progress.

## What Changes

- The stage-0 prototype becomes a production editor widget: in-place
  markdown-lite rendering, typewriter centring, selection/clipboard/undo,
  Cyrillic + IME — now with **word wrap** (absent in the prototype and too
  deep in layout, hit-testing, and centring to retrofit) and **autosave on
  every pause**.
- The `markdown-lite` parser crate moves from `prototypes/` into the
  production workspace as-is; the prototype view layer is rewritten, per the
  stage-0 decision.
- The Today screen gains the big **"Write" button**: it continues the
  in-progress essay, or offers a spark to start from when the slot is free.
- Starting an essay **consumes the chosen spark**: its text seeds the draft's
  front matter, a slug is derived from it, and it leaves the spark box.
- **Write mode screen**: fullscreen, no panels, dark flow-focused look, only
  the current fragment visible, a quiet session timer in the corner.
- Finished sessions are **recorded to `sessions.jsonl`** — the dotted
  calendar that reads them comes in stage 5.

## Capabilities

### New Capabilities

- `write-mode`: the fullscreen Write screen — flow view showing only the
  current fragment, deliberate friction on going back, autosave on pause and
  on exit, leaving back to Today.
- `start-from-spark`: how writing begins — the "Write" button routing
  (continue vs. pick a spark), spark consumption into a Draft essay, slug
  derivation, and surfacing the storage layer's WIP = 1 refusal.
- `writing-sessions`: the session as the unit of habit — quiet corner timer,
  soft finish suggestion, session records persisted.

### Modified Capabilities

- `live-markdown-editing`: new requirement — soft word wrap at the viewport
  width; typewriter centring, mouse hit-testing, and vertical movement operate
  on visual (wrapped) lines.
- `today-screen`: the screen gains the "Write" button as its primary action.
- `spark-capture`: the spark list shows the *remaining* sparks — a spark
  consumed by starting an essay leaves the list.

## Impact

- **Crates**: `markdown-lite` joins the production workspace (moved from
  `prototypes/`, tests included); `esse-app` gains the editor widget, the
  Write screen, and Today ↔ Write routing; `esse-core` gains spark removal
  (atomic rewrite, as designed in stage 1), slug derivation, and starts
  actually writing `sessions.jsonl` through the existing `SessionStore`.
- **Storage formats**: unchanged — everything needed (`spark` field,
  `sessions.jsonl`, atomic writes) was laid down in stage 1.
- **Dependencies**: no new external dependencies expected; gpui stays pinned
  at `v1.17.2`.

## Non-goals

- **Edit mode and mode switching** — stage 3; the `Draft ↔ Editing`
  transition stays unexercised by the UI.
- **Publishing, the Shelf, freeing the WIP slot** — stage 4.
- **Session dotted calendar and published row** — stage 5; this change only
  records sessions.
- **Mechanical cursor lock in Write mode** — the concept's open question is
  resolved for now as friction-only; a hard lock is a post-dogfooding
  experiment.
- **Onboarding, settings, hotkey customization** — stage 6 or never.

Checked against the anti-features list: no AI, no toolbars or style pickers,
no folders/tags, no statistics or streaks (sessions are recorded, not
displayed or counted), no settings.
