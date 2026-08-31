# Edit Mode and Mode Switching (Stage 3)

## Why

The pipeline currently ends at the draft: Write mode produces text, but there
is no way to see the whole essay, rearrange it, or cut it — and no way to move
an essay into its Editing state from the UI. This change serves beginner
problem #3 ("drafting and editing get mixed"): it gives editing its own room,
visibly different from the writing room, so the two processes stay physically
separate. It is Stage 3 of PLAN.md and the second half of the app's central
mechanic — the two modes.

## What Changes

- A new Edit mode screen: the full text at full strength, calm light
  treatment, free scrolling, normal navigation and editing — the visual and
  behavioral opposite of Write mode's dark centred flow.
- Edit mode reuses the same editor widget (markdown-lite rendering, wrap,
  undo, IME) with typewriter centring and above-cursor dimming off; the
  editor gains a scrolled viewport for when the cursor does not own the view.
- Switching Write ↔ Edit becomes an explicit action available inside both
  modes. The switch persists the essay's Draft ↔ Editing transition (already
  modeled and enforced in `esse-core`) and, out of Write mode, records the
  writing session as leaving always does.
- Routing respects the essay's state: continuing an in-progress essay opens
  Write mode for a Draft and Edit mode for an essay in Editing.
- Edit mode autosaves on pause and on exit, exactly like Write mode; Escape
  leaves to Today with the essay staying in Editing.

## Capabilities

### New Capabilities

- `edit-mode`: the Edit screen — whole text visible and scrollable, calm
  light treatment visibly distinct from Write mode, normal editing, autosave,
  the explicit switch back to Write, and explicit exit to Today.

### Modified Capabilities

- `write-mode`: gains the explicit "switch to Edit" action — it saves the
  essay, records the session, transitions the essay to Editing, and lands in
  Edit mode instead of Today.
- `start-from-spark`: the "Write" button's continue-routing becomes
  state-aware — a Draft opens in Write mode, an essay in Editing opens in
  Edit mode (previously everything opened in Write mode).

No delta for `essay-lifecycle` (Draft ↔ Editing transitions are already
specified and implemented) or `writing-sessions` (the session is recorded on
leaving Write mode, however it is left — switching included).

## Non-goals

- Publishing and shelving (Stage 4) — Edit mode has no "finish" action yet.
- The Shelf screen, published row, session dots (Stages 4–5).
- Sessions in Edit mode — the writing session stays a Write-mode concept; no
  timer or session record for editing time.
- Any editing aids beyond the existing markdown-lite editor: no formatting
  toolbars, style pickers, outline panels, or word counts (anti-features).
- Loosening WIP = 1 or adding routes into the editor besides "Write".

## Impact

- `crates/esse-app`: new `edit.rs` screen view; `root.rs` routing grows the
  Edit arm and the switch transitions; `editor/` gains a scrolled-viewport
  mode (scroll position, wheel handling, cursor-follows-into-view) and an
  `EditorStyle::edit()` palette; `theme.rs` gains the light Edit palette.
- `crates/esse-core`: no API changes expected — `transition_to`, `save`, and
  the WIP invariant already exist; the app starts calling `transition_to`.
- Specs: new `openspec/specs/edit-mode/spec.md`; deltas to `write-mode` and
  `start-from-spark`.
