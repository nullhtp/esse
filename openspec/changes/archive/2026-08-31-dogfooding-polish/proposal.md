# dogfooding-polish — Proposal

## Why

Stages 1–5 closed the conveyor: spark → draft → editing → published, with the
rhythm and the showcase on Today. The app now works, but the "first week" from
the concept still has friction at its edges: a brand-new user (or a fresh data
directory) lands on an empty screen with no nudge to seed the spark box, core
actions need the mouse, and nothing guarantees the "spark in under five
seconds" promise survives a cold start. Stage 6 of PLAN.md is the polish pass
driven by dogfooding: make the entry effortless, the daily loop keyboard-fast,
and re-examine the concept's open questions against live experience.

This serves beginner problems #1 ("nothing to write about" — the onboarding
seeds the spark box, and the cold-start budget keeps capture at five seconds)
and #2 ("cannot start" — hotkeys and launch speed cut the cost of beginning a
session). Typography polish serves no new problem; it is upkeep of the
already-specified Typora feel that makes Write/Edit mode (problem #3) worth
living in.

## What Changes

- **First-run onboarding**: when the app opens with an empty spark box and no
  essays, Today asks the user to record 3–5 sparks — a quiet prompt around the
  existing spark input, not a wizard. It disappears once sparks exist and never
  returns. This is the only onboarding, per the concept.
- **Keyboard shortcuts**: the daily loop becomes fully keyboard-drivable —
  start/continue writing from Today, open and leave the Shelf, focus the spark
  input — joining the existing editor bindings (`cmd-e` mode switch, `escape`
  to leave). Shortcuts stay unadvertised chrome: no shortcut cheat-sheet
  screen, no customization.
- **Cold-start budget**: the existing "capture-ready on launch" requirement
  gains a time budget — launch to a focused, typeable spark input fast enough
  that the five-second capture promise includes opening the app.
- **Typography polish**: spacing, sizes, and rhythm across the three screens
  and the editor are tuned by eye against real essays; no new spec behavior.
- **Open-questions review**: the concept's open questions (loosen WIP? tighten
  Write-mode backtracking? export forms) are re-answered from dogfooding
  experience and the decisions recorded in CONCEPT.md / PLAN.md. Any resulting
  behavior change becomes its own future change.

## Capabilities

### New Capabilities

- `first-run-onboarding`: the single onboarding moment — an empty app asks for
  3–5 sparks on Today, then gets out of the way forever.
- `keyboard-shortcuts`: keyboard access to the core loop outside the editor —
  write, shelf, capture — without a discoverability UI.

### Modified Capabilities

- `today-screen`: the "Capture-ready on launch" requirement gains a cold-start
  time budget, so capture-readiness is a performance guarantee, not just a
  focus rule.

## Non-goals

- No changes to the five mechanics or the WIP = 1 invariant in this change —
  the open-questions review only records decisions; loosening or tightening
  anything ships as its own change.
- No OS-level global hotkey or background quick-capture window. PLAN.md lists
  it as a fallback if instant launch proves unreachable; the budget is tried
  first.
- No settings, no shortcut customization, no font pickers — typography is
  tuned once by the author, per the anti-features list.
- No word counts, streaks, or statistics anywhere the polish touches.
- No expansion of markdown-lite.

## Impact

- `crates/esse-app/src/today.rs` — onboarding prompt state and rendering.
- `crates/esse-app/src/main.rs` — new key bindings; startup path is the place
  cold-start time is won or lost.
- `crates/esse-app/src/theme.rs` and screen modules — typography tuning.
- `crates/esse-core` — likely untouched (the "is the app empty?" check reads
  existing stores); no storage format changes.
- `CONCEPT.md` / `PLAN.md` — open-questions decisions recorded (Russian docs).
- No new dependencies expected.
