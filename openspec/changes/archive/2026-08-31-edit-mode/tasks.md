# Tasks — Edit Mode and Mode Switching

## 1. Editor viewport (design D5 — the risky foundation, first)

- [x] 1.1 Refactor `editor/element.rs` layout to run from a viewport anchor,
      with typewriter centring expressed as "anchor = cursor row centred";
      no behavior change, existing editor and performance tests stay green
- [x] 1.2 Add the Scrolled viewport mode: scroll offset owned by
      `EditorView`, wheel/trackpad events move it, clamped to the document;
      cursor stays put while scrolling
- [x] 1.3 Scrolled mode keeps the caret visible: any edit or cursor movement
      scrolls the minimum needed to bring the caret into view; unit tests
      for clamping and caret-follow

## 2. Theme and editor style

- [x] 2.1 Add the light `theme::edit` palette and `EditorStyle::edit()`
      (`dim_above: false`), deliberately far from the Write palette (D6)

## 3. Shared autosave (design D7)

- [x] 3.1 Extract the debounced save / save-on-quit / revision tracking from
      `WriteView` into a shared helper; `WriteView` uses it with no behavior
      change

## 4. Edit view

- [x] 4.1 New `edit.rs`: `EditView` with the light treatment, scrolled
      editor, shared autosave, and Escape emitting a leave event (essay
      stays in Editing); quiet save-trouble notice like Write's
- [x] 4.2 Add the corner mode-switch control («Пишу») and the `cmd-e`
      binding to `EditView`, emitting a switch-to-Write event

## 5. Write view switch

- [x] 5.1 Add the corner mode-switch control («Правлю») and `cmd-e` to
      `WriteView`: runs `finish()` (save + record session), then emits a
      switch-to-Edit event

## 6. Routing and transitions (design D1, D2, D4)

- [x] 6.1 Turn `RootView`'s `write: Option<…>` into the three-arm screen
      enum; entering either editor mode from Today requests fullscreen,
      leaving to Today restores the previous window state, switching swaps
      views without touching the window
- [x] 6.2 Implement the switch transitions in `RootView`: save →
      `transition_to` → save in order; on failure stay in the current mode
      and surface the error quietly
- [x] 6.3 Make continue-routing state-aware: "Write" opens Write mode for a
      Draft and Edit mode for an essay in Editing; new essays still open in
      Write mode

## 7. Verification

- [x] 7.1 Walk every scenario in the `edit-mode`, `write-mode`, and
      `start-from-spark` deltas by hand in the running app (including quit
      mid-edit and the failed-switch path); `cargo test`, `cargo clippy`,
      `cargo fmt --check` clean
