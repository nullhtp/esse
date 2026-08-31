# Rhythm and Showcase

## Why

The conveyor is closed — spark → draft → editing → published all work — but the
Today screen still shows none of the results. Session history is recorded and
never displayed; published essays are visible only on the Shelf. Stage 5 of the
plan puts both onto Today: the dotted calendar of recent sessions and the row of
published essay cards. This serves beginner problem #4 ("gives up after two
weeks") — the habit and the pile of finished work become visible on the screen
the app opens into — and reinforces #5 ("ashamed to publish") by making
published essays the main progress display, exactly as mechanics #4 and #5 of
the concept prescribe.

## What Changes

- Today gains a **session dots** strip: one dot per day for the last few weeks,
  filled when at least one session was recorded that day, empty otherwise. No
  streak counters, no numbers, no colors of shame — a missed day is simply an
  empty dot.
- Today gains a **published row**: published essays as cards, newest first,
  showing the title and, when recorded, the publication link. This is the main
  showcase of progress.
- Both elements are visually subordinate to the "Write" button and the spark
  input: Today remains a launchpad for writing, not a dashboard. Capture-ready
  focus on launch is unchanged.
- The writing-sessions spec drops its "nothing displays the history yet"
  wording — the calendar now reads `sessions.jsonl`.

## Capabilities

### New Capabilities

- `session-calendar`: the unobtrusive dotted display of recent writing sessions
  on the Today screen — which days get a dot, how far back it reaches, and what
  it deliberately never shows (streaks, counts, statistics).
- `published-row`: the row of published essay cards on the Today screen —
  ordering, card content, the publication link, and the empty state before the
  first published essay.

### Modified Capabilities

- `today-screen`: the "later stages extend it" wording becomes concrete — the
  screen now contains the session dots and the published row, and the layout
  hierarchy requirement extends to keep both subordinate to the "Write" button
  and the spark input.
- `writing-sessions`: the "Sessions are recorded" requirement no longer states
  that nothing displays the history — the session calendar reads it now.

## Non-goals

- No streaks, no counts, no word statistics, no graphs — the dots show presence,
  never quantity (anti-features list).
- No new data: the calendar reads the existing `sessions.jsonl`, the row reads
  existing published essays. No schema changes.
- No interaction with the dots — they are not a navigation element and open
  nothing.
- No changes to the Shelf, the editor, or the lifecycle; no settings for the
  calendar depth or the row length.
- No onboarding or empty-state ceremony beyond quietly hiding what has no data
  yet (that is stage 6).

Checked against the anti-features list: no AI, no toolbars, no folders/tags, no
social features, no statistics or streaks, no settings.

## Impact

- `crates/esse-app/src/today.rs` — the Today screen gains the two new sections.
- `crates/esse-app/src/data.rs` — Today's data loading extends to session
  history and published essays.
- `crates/esse-core/src/store/` — read-side queries only (sessions by day,
  published essays newest first); on-disk formats are untouched.
- Specs: two new (`session-calendar`, `published-row`), two deltas
  (`today-screen`, `writing-sessions`).
- No new dependencies.
