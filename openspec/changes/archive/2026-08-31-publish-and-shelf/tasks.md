# Tasks — publish-and-shelf

## 1. Core: completion operations

- [x] 1.1 Add `publish` and `shelve` operations to `esse-core` (over the
      existing lifecycle transitions): `publish` sets `published_at` and the
      optional `publication_url` and transitions to Published; `shelve`
      transitions to Shelved; unit tests for both, including the
      failed-transition paths and the absent-`publication_url` round trip
- [x] 1.2 Add a `body_markdown` accessor (essay body without front matter)
      for copy/export, plus queries the Shelf needs: published essays newest
      first by `published_at`, shelved essays; tests over the store

## 2. Editor: finish control and completion overlay

- [x] 2.1 Add the quiet finish control overlay to Edit mode only, opening a
      completion overlay with Publish and Shelve; dismissing it changes
      nothing (spec: essay-completion "Completion is offered from Edit mode",
      edit-mode delta)
- [x] 2.2 Build the publish panel: "Copy as markdown" (clipboard, body
      only), "Export to file…" (native save dialog pre-filled `<slug>.md`,
      body only, cancelled dialog writes nothing), optional link field
- [x] 2.3 Wire the confirming "Published" action: save → publish op → return
      to Today with window state restored; failure stays in Edit mode with
      the quiet error (same pattern as the mode switch)
- [x] 2.4 Wire Shelve behind one confirmation step: confirm → save → shelve
      op → Today; decline returns to the overlay unchanged; failure stays put

## 3. Shelf screen

- [x] 3.1 Build the Shelf screen skeleton with the three columns fed from
      the stores — Sparks (newest first), In progress (0 or 1 card),
      Published (newest first, link shown when recorded) — no counts, stats,
      or ordering options
- [x] 3.2 Add the collapsed shelved drawer: expands/collapses on explicit
      action, display only, no state-changing actions on shelved essays
- [x] 3.3 Add navigation: quiet Shelf affordance on Today (visually
      subordinate, never steals launch focus from the spark input), return
      via affordance or Escape, window state untouched both ways
- [x] 3.4 Wire Shelf actions through the existing start-from-spark routing:
      spark click starts an essay only when the slot is free (no start
      action offered while occupied); the In progress card continues the
      essay in the mode matching its state

## 4. Verification

- [ ] 4.1 Full-pipeline check on a scratch `ESSE_DATA_DIR`: spark → write →
      edit → publish (copy + export + link) frees the slot, next spark can
      start; repeat ending with shelve; `cargo test` and `cargo clippy` clean
