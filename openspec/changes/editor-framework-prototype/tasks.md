# Tasks: Editor Framework Prototype

Groups 5-6 run only if the gpui prototype fails the evaluation (design.md D1,
early exit).

## 1. Workspace and markdown-lite parser

- [ ] 1.1 Create `prototypes/` Cargo workspace with a `markdown-lite` library crate; `cargo test` runs green on a placeholder test
- [ ] 1.2 Implement line classification (heading level from leading `#`) and inline span parsing for `**bold**` and `*italic*` with byte ranges (text spans + marker spans)
- [ ] 1.3 Unit-test the parser: plain lines, headings, bold, italic, unclosed markers stay literal, other markdown (`-`, `[]()`, backticks) stays plain, Cyrillic byte offsets correct
- [ ] 1.4 Add a render-plan helper: for (line, cursor_on_line) return either styled spans with markers hidden or raw text — the single entry point both prototypes call

## 2. gpui prototype: plumbing and hard gates

- [ ] 2.1 Pin a gpui git revision (record it in design.md open questions), get an empty gpui window building and running
- [ ] 2.2 Wire keyboard input to a text buffer and draw it as plain multi-line text
- [ ] 2.3 Hard gate: verify Cyrillic typing and IME composition insert correctly; record the result immediately

## 3. gpui prototype: editor behavior

- [ ] 3.1 Cursor: arrow-key movement, home/end, click-to-position
- [ ] 3.2 Selection with shift+arrows and mouse drag; render selection highlight
- [ ] 3.3 Clipboard: copy, cut, paste through the system clipboard
- [ ] 3.4 Undo/redo stack over buffer edits
- [ ] 3.5 Styled rendering: draw each line through the markdown-lite render plan (heading size, bold, italic; markers hidden)
- [ ] 3.6 Line-scoped marker reveal: the cursor line renders raw, re-renders styled on leave
- [ ] 3.7 Typewriter mode: keep the cursor line vertically centred while typing and on Enter
- [ ] 3.8 Paste a ~20,000-character sample essay; check typing latency and scroll smoothness

## 4. gpui evaluation

- [ ] 4.1 Walk every scenario in `specs/live-markdown-editing/spec.md` against the prototype; fill the gpui column of the Decision Record in design.md
- [ ] 4.2 Subjective gate: one real 20-minute writing session in the prototype; record the verdict
- [ ] 4.3 Early-exit decision: if all requirements pass, record gpui as chosen in design.md and skip groups 5-6; otherwise proceed

## 5. iced prototype (only on gpui failure)

- [ ] 5.1 Add `iced-editor` crate to the workspace; empty iced window builds and runs
- [ ] 5.2 Build a minimal custom editor widget on cosmic-text: buffer, keyboard input, plain-text drawing
- [ ] 5.3 Hard gate: verify Cyrillic and IME input; record the result immediately
- [ ] 5.4 Cursor movement, click-to-position, selection with highlight
- [ ] 5.5 Clipboard and undo/redo
- [ ] 5.6 Styled rendering through the markdown-lite render plan
- [ ] 5.7 Line-scoped marker reveal
- [ ] 5.8 Typewriter mode
- [ ] 5.9 ~20,000-character latency and scroll check

## 6. iced evaluation (only on gpui failure)

- [ ] 6.1 Walk every spec scenario; fill the iced column of the Decision Record
- [ ] 6.2 Subjective gate: 20-minute writing session; record the verdict

## 7. Wrap-up

- [ ] 7.1 Complete the Decision Record in design.md: chosen framework and rationale (or, if both failed, the reconsideration note per D2)
- [ ] 7.2 Resolve the remaining open question: whether the winning prototype seeds the production crate or gets a clean rewrite; note it in design.md
- [ ] 7.3 Update README.md ("Статус") and PLAN.md: framework decided, stage 0 complete
