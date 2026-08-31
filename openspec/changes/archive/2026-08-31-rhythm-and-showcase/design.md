# Design — Rhythm and Showcase

## Context

The conveyor is complete and both data sources for stage 5 already exist on
disk and in code: `SessionStore::load_all()` reads `sessions.jsonl` (recorded
by Write mode since stage 2) and `EssayStore::published()` returns published
essays newest first (used by the Shelf since stage 4). The Today screen
(`crates/esse-app/src/today.rs`) currently shows the "Write" button, the spark
input, and the spark list; the stores are re-read on every render
(`data.rs` deliberately caches nothing), so freshness comes for free.

This change is display-only: two new read-only sections on Today, no writes,
no schema changes, no new store mutations.

## Goals / Non-Goals

**Goals:**

- The dotted session calendar on Today: presence of writing, not quantity.
- The published row on Today: the pile of finished essays as the main
  progress display.
- Both quiet — Today stays a launchpad whose loudest element is "Write".

**Non-Goals:**

- Streaks, counts, statistics, graphs of any kind (anti-features list).
- Interactivity in the new sections beyond what the Shelf already offers for
  the same data (the dots are inert; cards display, they do not navigate).
- Any change to the editor, the Shelf, the lifecycle, or the on-disk formats.
- Settings for the calendar depth or row length — both are fixed constants,
  like the 20-minute session target.

## Decisions

### 1. The calendar is one row of 28 day-dots, today rightmost

A single horizontal dotted line — the last 28 days, oldest left, today
rightmost. A dot is filled when at least one session record falls on that
local calendar day, empty otherwise. No axis labels, no weekday letters, no
numbers, no tooltips.

- *Why one row and not a GitHub-style week grid*: a grid reads as a
  statistics widget and invites streak-counting; the concept's word is
  "пунктир" — a dotted line. 28 dots is "the last weeks" from the plan,
  small enough to stay a texture rather than a chart.
- *Why 28 is a constant*: same reasoning as the 20-minute session target in
  writing-sessions — settings are an anti-feature; tune by editing the
  constant during dogfooding.

### 2. A session belongs to the local day of `started_at`

`Timestamp` is `chrono::DateTime<FixedOffset>`; the dot for a day is filled
when `started_at.with_timezone(&Local).date_naive()` equals that day. The
day the writing *started* is the day that gets the dot — a session crossing
midnight still marks one day, matching how a person remembers "I wrote on
Tuesday".

### 3. Aggregation lives in the app layer, not the store

`today.rs` folds `SessionStore::load_all()` into a `HashSet<NaiveDate>` at
render time. No new store query.

- *Why*: which days get dots is a presentation question; `esse-core` stays
  about storage and lifecycle. `load_all()` already exists and the file is a
  personal writing history — rescanning it per render is well within budget,
  and consistent with the "nothing is cached" rule in `data.rs`.
- *Alternative considered*: `SessionStore::days_with_sessions(range)` —
  rejected as a second API for the same bytes.

### 4. The published row reuses the Shelf's data path and title rule

The row calls `EssayStore::published()` (already sorted newest first with the
`updated_at` fallback) and derives card titles exactly as the Shelf does.
The `title()` helper currently private to `shelf.rs` moves to a shared spot
in `esse-app` so Today and the Shelf cannot drift apart. A card shows the
title and, when recorded, the publication link — the same fields the Shelf's
Published column shows, in card form.

### 5. Sections with no data disappear, they do not placeholder

No sessions in the last 28 days → no calendar strip. No published essays →
no row. An all-empty dotted line on first launch would read as reproach —
the opposite of "no guilt for a missed day" — and empty-state ceremony is
stage 6's onboarding concern, not this change's.

### 6. Layout keeps the existing hierarchy

Order on Today: "Write" button, spark input + spark list (unchanged,
capture-ready focus untouched), then the published row, then the session
dots at the quiet bottom edge. Both new sections are visually dimmer than
the spark list; neither can take launch focus.

## Invariants

- **WIP = 1**: untouched. Both sections are read-only; starting an essay
  still goes only through the start-from-spark rules. Cards do not open
  essays, so no new path into the editor exists.
- **Write/Edit separation**: untouched. Nothing here enters or alters the
  editor; the session calendar only reads what leaving Write mode recorded.

## Risks / Trade-offs

- [28 hard-coded days may feel wrong in dogfooding] → it is one constant next
  to the session-target constant; adjust by edit, not by setting.
- [Re-reading `sessions.jsonl` every render grows linearly with history] →
  years of daily writing is still a few thousand short lines; if it ever
  shows up in profiling, cache behind a file-mtime check without changing the
  store API.
- [Published-row cards displaying raw links can look busy] → same trade-off
  the Shelf already accepted; revisit both together in stage 6 typography
  polish.
