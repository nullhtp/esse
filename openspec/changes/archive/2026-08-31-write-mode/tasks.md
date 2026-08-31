# Tasks: Write Mode

## 1. Core foundations (esse-core, no GUI)

- [x] 1.1 Move `markdown-lite` from `prototypes/` to `crates/markdown-lite`, add it to the workspace, keep its tests green; leave a pointer in `prototypes/README.md`
- [x] 1.2 `SparkStore::remove(id)` — atomic rewrite of `sparks.jsonl` without the removed line; tests for remove, remove-missing, and survival across reload
- [x] 1.3 Slug derivation in `esse-core`: Cyrillic transliteration table, lowercase/hyphenate/collapse, word-boundary truncation (~48 chars), `essay-<date>` fallback; unit tests incl. «почему эссе» → `pochemu-esse`
- [x] 1.4 `start_essay_from_spark(&SparkStore, &EssayStore, spark_id)`: create essay (spark text as seed) then remove spark; `-2`/`-3` suffix retry on `SlugTaken`; tests — happy path, failure keeps the spark, WIP refusal passes through

## 2. Editor widget (esse-app)

- [x] 2.1 Port the prototype `Buffer` (text, selection, undo/redo, movement) into `esse-app` with its tests
- [x] 2.2 Visual-line layout: wrap each source line via gpui shaping at viewport width; offset ↔ (visual line, column) mapping type with unit tests
- [x] 2.3 Editor element: render visual lines with markdown-lite styling; marker reveal/hide keyed to the cursor's source line
- [x] 2.4 Text input: `EntityInputHandler` for typing and IME composition (Cyrillic gate), backspace/delete, Enter
- [x] 2.5 Selection and clipboard: shift-movement, mouse drag selection, copy/cut/paste; undo/redo keybindings
- [x] 2.6 Mouse hit-testing and vertical movement on visual lines (up/down with sticky column, home/end on the visual line)
- [x] 2.7 Typewriter centring on the cursor's visual line (typing, Enter, and wrap all keep it centred)
- [x] 2.8 Performance pass: shape/render only viewport-near lines; verify no perceptible keystroke lag at ~20k characters

## 3. Routing and Today

- [x] 3.1 Root router (`Today` | `Write { essay }`): screen switch, native fullscreen on enter, restore on leave
- [x] 3.2 "Write" button on Today as the primary action; when an essay is in progress it opens that essay in Write mode
- [x] 3.3 Spark picker for the free-slot path: choose a spark → `start_essay_from_spark` → Write mode; empty-box hint ("a spark is needed"); surface the WIP refusal defensively
- [x] 3.4 Spark list shows remaining sparks only — refreshes after a spark is consumed

## 4. Write-mode behaviors

- [x] 4.1 Write-mode visual treatment: dark theme variant, dimming of paragraphs above the cursor's
- [x] 4.2 Cursor-following viewport: ignore wheel/trackpad scrolling in Write mode
- [x] 4.3 Autosave: ~1 s debounce after the last edit, plus save on leaving Write mode, window close, and quit; `updated_at` follows
- [x] 4.4 Session timer: starts on entering Write mode, quiet corner indicator, soft "session complete" state at the 20-minute constant
- [x] 4.5 Leaving Write mode (Escape): save, append the session record (skip < 1 min), return to Today

## 5. Verification

- [ ] 5.1 Walk every spec scenario manually in the running app (wrap, markers, typewriter, dimming, autosave-by-inspecting-the-file, session record in `sessions.jsonl`); tune the dimming constant
- [x] 5.2 Update README.md (Russian): how writing works now — Write button, spark consumption, where sessions land on disk
