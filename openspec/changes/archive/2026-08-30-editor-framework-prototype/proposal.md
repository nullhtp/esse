# Editor Framework Prototype

## Why

The GUI framework (gpui / iced / egui) is the last unresolved foundational decision,
and live markdown rendering is the most expensive part of the project. Until a
prototype proves the "Typora feel" is achievable on a concrete framework, building
anything else risks rework. This change de-risks the project by building the same
live-markdown-editor prototype on the two strongest candidates and recording a
framework decision.

**Beginner problems served:** #2 "cannot start" and #3 "drafting and editing get
mixed" — both depend on the editor: Write mode's fullscreen typewriter flow is the
mechanic against them, and this prototype proves that editor is buildable. The
change delivers no end-user feature itself; it is the prerequisite spike the
concept explicitly calls for ("resolve it with a live-markdown-editor prototype
before main development").

## What Changes

- Add a `prototypes/` directory with two throwaway prototypes of a live
  markdown-lite editor: `prototypes/gpui-editor/` and `prototypes/iced-editor/`.
- Both prototypes implement the same behavior: in-place rendering of `#`,
  `*italic*`, `**bold**` as you type (raw markers visible only on the current
  line), typewriter mode with the cursor vertically centred, basic editing
  (selection, copy/paste, undo), Cyrillic/IME input.
- Evaluate both against fixed success criteria; record the framework decision
  and evaluation results in `design.md` of this change.
- egui is deliberately not prototyped: it is a fallback to reconsider the
  approach only if both candidates fail.

## Capabilities

### New Capabilities

- `live-markdown-editing`: the behavioral requirements for the editor core —
  in-place markdown-lite rendering (Typora-like, no split preview), typewriter
  mode, and baseline text-editing correctness. The prototype validates these
  requirements are achievable; the same spec later governs the production editor.

### Modified Capabilities

None — this is the first change; no specs exist yet.

## Impact

- New code: `prototypes/gpui-editor/`, `prototypes/iced-editor/` (throwaway,
  excluded from the future main crate; kept for reference).
- New dependencies (prototype-local only): gpui, iced, pulldown-cmark or a
  hand-written markdown-lite span parser.
- Decision impact: the chosen framework becomes the foundation for all
  subsequent UI work (plan stages 1-6); the losing prototype is kept but frozen.
- No user data, no storage, no existing code affected.

## Non-goals

- Not the production editor: no autosave, no persistence, no Write/Edit mode
  switch, no session timer, no screens ("Today", "Shelf").
- No full CommonMark: headings, italic, bold only (markdown-lite is the whole
  scope, per the concept).
- No polish beyond what the success criteria demand; prototype code quality is
  explicitly allowed to be poor.
- Nothing from the anti-features list is touched: no formatting toolbars, no
  font settings, no AI, no statistics (checked — the prototype adds none of these).
