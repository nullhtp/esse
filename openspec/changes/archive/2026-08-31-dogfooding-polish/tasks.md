# dogfooding-polish — Tasks

## 1. First-run onboarding

- [x] 1.1 Add the empty-app condition to `TodayView` (no sparks and no essays
      in any state; prompt persists below three sparks while no essay
      exists), computed inside the existing `reload` from data already read —
      no new storage, no flags (design D1)
- [x] 1.2 Render the prompt: one or two quiet lines with the spark input,
      visually subordinate to the Write button, never taking focus; wording
      per the concept ("ideas arrive daily and get lost — write down 3–5
      now"), Russian on screen like the rest of the UI
- [x] 1.3 Verify the recede rules by hand against a fresh data directory
      (`ESSE_DATA_DIR` to a temp dir): first launch shows the ask, first
      spark keeps it, third spark ends it, starting an essay ends it

## 2. Keyboard shortcuts

- [x] 2.1 Add a `"Today"` key context to `TodayView`'s root element and a
      `Write` action wired to the same path as the Write button; bind
      `cmd-enter` in `main.rs` next to the existing bindings
- [x] 2.2 Add the Shelf shortcut both ways: `cmd-l` on Today opens the Shelf,
      `cmd-l` on the Shelf returns to Today (reusing `shelf::Leave`)
- [x] 2.3 Verify shortcuts yield to typing: unmodified characters (Cyrillic
      and IME included) land in the spark input, and the whole loop runs
      pointer-free: type → Enter → `cmd-enter` → `escape` → `cmd-l` → `cmd-l`

## 3. Cold-start budget

- [x] 3.1 Instrument startup: measure `main()` entry to first-frame-ready and
      report it behind `ESSE_STARTUP_TIMING=1` via the existing `env_logger`
      route (design D3)
- [x] 3.2 Measure cold and warm starts on the target machine several times;
      record the numbers in the change (or design.md addendum)
- [x] 3.3 Only if over one second: move data loading off the critical path
      (open the window first, fill lists on first refresh) and re-measure; if
      the floor is gpui's own init, record that fact for task 5.1 instead of
      forcing a fix
      — not triggered: cold start is 0.5–0.8 s, warm 0.15 s (design.md D3
      addendum), so nothing was moved off the critical path

## 4. Typography pass

- [x] 4.1 One deliberate pass over `theme.rs` and per-screen sizes against
      real dogfooding essays: type scale, line heights, column measures,
      spacing rhythm on Today and the Shelf (design D4, timeboxed)
      — walked through with real essays on screen; nothing wanted changing,
      so `theme.rs` is untouched
- [x] 4.2 The same pass over the editor's two modes, checking the Write/Edit
      visual contrast still reads clearly after tuning
      — both modes checked; the contrast still reads at a glance

## 5. Open-questions review (documentation only)

- [x] 5.1 Revise CONCEPT.md «Открытые вопросы» and PLAN.md's recommendations
      table from lived experience: WIP strictness, Write-mode backtracking,
      export forms, the measured cold-start number, and anything else
      dogfooding surfaced — each answered "keep as is" or sketched as a
      future change; no behavior changes in this change (design D5)
