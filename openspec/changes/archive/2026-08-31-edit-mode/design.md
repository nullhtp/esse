# Design — Edit Mode and Mode Switching

## Context

Stage 2 built the Write half of the editor screen: `WriteView` (dark,
fullscreen, autosave, session recording) over the shared `EditorView`, routed
by `RootView` which holds `write: Option<Entity<WriteView>>` so the editor is
unreachable without an essay. The editor deliberately has no scroll position —
the viewport is a pure function of the cursor (typewriter centring), which is
also why the wheel does nothing.

`esse-core` is already Stage-3-ready: `EssayStatus::Editing` exists,
`Essay::transition_to` enforces the legal transitions, `is_in_progress()`
covers Draft and Editing, and the WIP = 1 check counts both. Nothing in core
needs to change; the app simply has not called `transition_to` yet.

## Goals / Non-Goals

**Goals:**

- An Edit screen: whole text, full strength, light calm layout, free
  scrolling, normal editing, autosave.
- An explicit Write ↔ Edit switch that persists the Draft ↔ Editing
  transition and keeps the two modes visibly and behaviorally contrasting.
- Continue-routing that opens the mode matching the essay's state.

**Non-Goals:**

- Publishing, shelving, the Shelf screen (Stage 4).
- Sessions or timers in Edit mode — the session stays a Write-mode concept.
- Scrollbars, minimaps, outlines, or any editing chrome beyond the text.
- Editor-core feature growth beyond the scrolled viewport (markdown-lite
  stays frozen).

## Decisions

### D1 — Routing: a three-arm screen enum, essay still carried by the arm

`RootView`'s `Option<WriteView>` becomes a screen enum:
`Today | Write(Entity<WriteView>) | Edit(Entity<EditView>)`. Both editor arms
carry a live view constructed with an essay, so the Stage-2 property — no
route reaches an editor without an essay — survives by construction.
Alternative considered: a single `EditorScreen` view owning a mode flag;
rejected because Write and Edit share almost nothing above the `EditorView`
widget (session timer vs none, dark vs light, centring vs scrolling), and two
small views are simpler than one view of ifs.

### D2 — Mode switching swaps views without touching fullscreen

The editor screen is fullscreen in both modes (the concept's Editor screen is
one fullscreen room; the contrast between modes is the treatment, not the
window). Entering either mode from Today requests fullscreen; switching
Write ↔ Edit swaps the view and does not toggle the window, so there is no
flicker; leaving either mode to Today restores the pre-editor window state,
which `RootView` already tracks in `was_fullscreen`.

### D3 — The switch is a quiet corner control plus `cmd-e`

Both modes show a small dim label in the top-right corner naming the other
mode («Правлю» in Write, «Пишу» in Edit); clicking it or pressing `cmd-e`
switches. One shortcut for both directions keeps it one gesture in muscle
memory. The control is text, not a toolbar — consistent with the
anti-feature list and the quiet session indicator already in the corner.

### D4 — Switching goes through the same "finish" path as leaving

Write → Edit: `WriteView::finish` runs exactly as on Escape (save the text,
record the session), then the essay transitions Draft → Editing and is saved,
then `EditView` takes over. Edit → Write: save, transition Editing → Draft,
save, `WriteView` takes over — which starts a new session, because entering
Write mode is what starts sessions (writing-sessions spec holds unchanged).
The transition uses `Essay::transition_to`, so an illegal state is a core
error surfaced in the corner, not a silent overwrite.

### D5 — The editor gains a viewport policy, not a second widget

`EditorView` grows a viewport mode next to `EditorStyle`:

- **Typewriter** (Write): today's behavior — the cursor's visual row is
  centred, the wheel is inert. Unchanged.
- **Scrolled** (Edit): the view owns a scroll offset in pixels; wheel and
  trackpad move it, clamped to the document; any edit or cursor movement
  scrolls the minimum needed to keep the caret visible (no centring).

The element already lays the document out from an anchor row; the scrolled
mode replaces "anchor = cursor row centred" with "anchor = scroll offset".
Alternative considered: separate Edit editor widget — rejected, it would fork
the buffer/display/wrap stack that took Stage 2 to harden.

### D6 — Light Edit palette in `theme.rs`, `EditorStyle::edit()`

`theme::edit` mirrors `theme::write` with an off-white ground, dark ink, and
`dim_above: false`. The two palettes are deliberately far apart — the mode
change must be felt at a glance (spec: visibly distinct).

### D7 — Autosave logic is extracted and shared

The debounced save + save-on-quit + revision tracking in `WriteView` moves to
a small shared helper (`autosave.rs`) used by both views, so "autosave on
pause" cannot drift apart between modes. The session recording stays in
`WriteView` — it is Write-only.

## Risks / Trade-offs

- [Scrolled viewport touches element geometry that assumed centring] → the
  anchor abstraction in D5 keeps one layout path; the typewriter mode becomes
  a special case of it, and the existing editor tests plus the performance
  test must stay green before the Edit screen builds on it.
- [Switch-with-transition has three fallible steps (save, transition, save)]
  → run them in that order; on failure stay in the current mode and show the
  error in the corner like save trouble today. Text is saved first, so no
  outcome loses words.
- [Two corner controls in Write mode (timer + switch) creep toward chrome] →
  both are small dim text; if it feels like furniture during dogfooding, the
  switch control can become hover-only without a spec change.

## Invariants check

- **WIP = 1**: untouched — no new essay-creation path; switching operates on
  the one in-progress essay, and Draft ↔ Editing both count as in progress.
- **Write/Edit separation**: strengthened — the modes get contrasting
  palettes, contrasting viewport behavior, and an explicit persisted state
  transition between them; there is still no route into either without an
  essay.
