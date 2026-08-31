# Tasks — Rhythm and Showcase

## 1. Session calendar

- [x] 1.1 Add the day-set fold in `esse-app`: a pure function turning
      `SessionStore::load_all()` output into the set of local calendar days
      (`started_at` → `Local` → `date_naive`), with unit tests covering an
      empty history, two sessions on one day, and a session started at 23:50
      marking only its start day.
- [x] 1.2 Render the dotted calendar on Today: a single row of 28 dots,
      oldest left and today rightmost, filled from the day set; dim, inert
      (no hover, no click), no labels or numbers; the whole strip absent when
      no session falls within the 28-day window.

## 2. Published row

- [x] 2.1 Move the `title()` helper out of `shelf.rs` into a shared module in
      `esse-app` and switch the Shelf to it, so Today and the Shelf derive
      card titles by one rule.
- [x] 2.2 Render the published row on Today from `EssayStore::published()`:
      cards newest first showing the title and the publication link when
      recorded; no counts, no state-changing actions, no editor navigation;
      the row absent while nothing is published.

## 3. Layout and verification

- [x] 3.1 Compose the extended Today layout — "Write" button, spark input and
      list, published row, session dots at the quiet bottom — and verify the
      hierarchy: the button stays the most prominent element and the spark
      input still takes keyboard focus on launch with both sections present.
- [x] 3.2 Run the scenario pass: finish a ≥1-minute session and see today's
      dot fill; publish the in-progress essay and see the row appear with its
      card and link; confirm a fresh data directory shows neither section.
      `cargo fmt`, `cargo clippy`, `cargo test` clean.
