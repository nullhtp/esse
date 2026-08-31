# Design — publish-and-shelf

## Context

The pipeline runs spark → Draft → Editing, and stops. `essay-lifecycle`
already defines the exits (Draft/Editing → Published, Draft/Editing →
Shelved) and the slot-freeing rule (only Draft and Editing count as in
progress); `local-storage` already reserves `published_at` and
`publication_url` in the essay front matter. Nothing in the UI reaches those
exits. This change adds the two missing rooms: the completion flow in the
editor and the Shelf screen.

## Goals / Non-Goals

**Goals:**

- An essay can end — Published (with the text handed out as markdown) or
  deliberately Shelved — and the WIP slot frees itself as a consequence.
- The third and final screen, the Shelf, shows the whole system at a glance:
  Sparks | In progress | Published, shelved essays in a collapsed drawer.
- Starting from a spark works from the Shelf too, under the same rules.

**Non-Goals:**

- No reviving shelved essays (the lifecycle forbids leaving Shelved).
- No editing of `publication_url` after publishing — it is captured in the
  completion flow or stays empty; revisit after dogfooding.
- No published row on Today, no session dots (stage 5).
- No publishing integrations; the clipboard and a `.md` file are the whole
  export surface.

## Decisions

### 1. Completion lives in Edit mode only

The finish control is an overlay in Edit mode; Write mode gets nothing. The
decision to finish is a quality-side judgement — you make it looking at the
whole text, which is exactly the room Edit mode is. Adding it to Write mode
would puncture the flow room with a completion decision, weakening the
Write/Edit separation this app is built around.

*Alternative considered:* completion actions on the Shelf's in-progress card.
Rejected: two entry points to maintain, and finishing an essay you are not
looking at invites accidental shelving. One path, in the room where the text
is visible whole.

### 2. Completion flow: one overlay, two exits

The finish control (quiet corner control, mirroring the existing mode-switch
control) opens a small completion overlay with the two outcomes:

- **Publish**: "Copy as markdown" and "Export to file…" buttons, an optional
  publication-link field, and a confirming "Published" action. Copy/export
  are usable *before* confirming — the realistic sequence is: copy → paste
  into the blog → paste the resulting URL back → confirm.
- **Shelve**: a single action behind one explicit confirmation step, so it is
  deliberate («осознанно в стол»), never a slip.

Cancelling the overlay changes nothing. A failed save or transition keeps
Edit mode on screen with the error surfaced quietly — same pattern as the
existing mode switch. After a successful completion the editor closes to
Today (fullscreen restored), because there is no essay to edit anymore.

### 3. Export and copy carry the body only

Both the clipboard copy and the file export contain the markdown body without
TOML front matter — the front matter is esse's bookkeeping, not part of the
essay a blog receives. Export uses the native save dialog with `<slug>.md`
pre-filled (gpui: `prompt_for_new_path`; clipboard via `write_to_clipboard`).

### 4. Publication link is optional

Publishing often completes outside the app after the copy; requiring a URL
would block the confirm at the worst moment. Empty stays empty in front
matter (`publication_url` simply absent), per the existing storage contract.

### 5. Shelf is a plain window screen

Today ↔ Shelf navigation is a quiet affordance on each screen (plus Escape on
the Shelf); no fullscreen — fullscreen belongs to the editor rooms. Columns
read straight from the existing stores: sparks newest-first (reused list),
the 0-or-1 in-progress essay, published essays newest-first by
`published_at`. The shelved drawer is collapsed by default and only expands
to be looked at. Clicking the in-progress card continues it exactly like the
Write button (same routing); clicking a spark starts from it when the slot is
free — both paths are the existing `start-from-spark` routing, not new logic.

### 6. WIP invariant untouched by construction

No new storage API. Completion calls the existing lifecycle transition; the
slot frees itself because Published/Shelved do not count as in progress. The
storage layer remains the sole enforcer of WIP = 1; the Shelf only *reflects*
slot state (startable sparks vs. an occupied slot), and the defensive
refusal path from `start-from-spark` still applies if routing is ever wrong.

## Risks / Trade-offs

- [Accidental shelving buries an essay irreversibly (no revival this stage)]
  → the confirmation step, and the drawer keeps the file visible; the `.md`
  file on disk is always recoverable by hand.
- [gpui save-dialog/clipboard APIs behave differently across platforms] →
  personal tool, macOS is the only target for now; both APIs are already
  used by Zed on macOS.
- [User publishes, forgets the link, and cannot add it later] → accepted for
  this stage (non-goal); the front-matter field is hand-editable, revisit
  after dogfooding.

## Open Questions

None blocking. Deferred by decision: shelved-essay revival, later
link editing — both listed in Non-Goals with their revisit points.
