# Publish and Shelf

## Why

Stages 1–3 built spark → draft → editing, but no essay can end: the pipeline
has no exit, so the WIP slot, once occupied, can never be freed and nothing is
ever finished. This change closes the conveyor. It serves beginner problem #5
("ashamed to publish") — publishing becomes the routine completion step, with
copy-as-markdown and file export built in — and supports #4 ("gives up after
two weeks") by making the Published column the visible pile of finished work.

## What Changes

- An essay can now be completed from Edit mode: **Published** (copy the body
  as markdown to the clipboard and/or export it to a `.md` file, with an
  optional publication link recorded in front matter) or deliberately
  **shelved**. Both end the essay through the existing lifecycle transitions
  and free the WIP slot.
- New **Shelf** screen — the third and final screen of the app: three columns
  Sparks | In progress | Published, plus a collapsed drawer for shelved
  essays. Starting an essay from a spark on the Shelf follows the same
  start-from-spark rules (only when the slot is free).
- The Today screen gains a quiet navigation affordance to the Shelf.

## Capabilities

### New Capabilities

- `essay-completion`: how an essay ends — the publish flow (markdown copy,
  file export, publication link, `published_at`) and the deliberate shelve
  action, both transitioning the essay out of "in progress" and freeing the
  WIP slot.
- `shelf-screen`: the Shelf screen — the three columns, the collapsed shelved
  drawer, navigation to and from Today, and starting an essay from a spark
  under the WIP rules.

### Modified Capabilities

- `edit-mode`: the calm full-text screen gains one more quiet overlay — the
  finish control that opens the completion flow.
- `today-screen`: the screen gains a quiet navigation affordance to the Shelf.

(`essay-lifecycle` and `local-storage` already define the required
transitions, the `published_at` / `publication_url` fields, and the
slot-freeing rule — no requirement changes there.)

## Non-goals

- No reviving shelved essays — the lifecycle forbids leaving Shelved; the
  drawer only shows them. Revisit after dogfooding.
- No published row or session dots on Today — that is stage 5.
- No publishing integrations, in-app sharing, or social features (anti-feature
  list): the user pastes the markdown into their own blog.
- No counts, statistics, or ordering options on the Shelf columns
  (anti-feature list).

## Impact

- New UI: the Shelf screen and the completion flow in the editor; clipboard
  and native save-dialog integration in gpui.
- Storage layer is used as-is: existing transitions, existing front-matter
  fields, no format changes.
- Routing: Today ↔ Shelf navigation; the Shelf's "start from spark" path
  reuses the start-from-spark routing.
