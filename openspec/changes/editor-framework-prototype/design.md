# Design: Editor Framework Prototype

## Context

The concept fixes the platform (native Rust desktop GUI) but leaves the framework
open: gpui / iced / egui. Live markdown rendering is named the most expensive part
of the project and the place to start. This spike builds the same live-markdown
editor prototype on the candidates and produces a recorded framework decision.
The requirements being validated live in `specs/live-markdown-editing/spec.md`.

The personal version targets macOS first (the author's machine); cross-platform
reach is not a selection criterion for the personal tool.

## Goals / Non-Goals

**Goals:**
- Prove the "Typora feel" (in-place markdown-lite rendering, typewriter mode) is
  achievable on a concrete framework, with Cyrillic/IME input working.
- Produce a framework decision recorded in this document (Decision Record below).
- Produce a reusable markdown-lite span parser that survives into production.

**Non-Goals:**
- Production editor features: persistence, autosave, Write/Edit switch, session
  timer, any of the three screens.
- Full CommonMark support or any syntax beyond `#`, `*italic*`, `**bold**`.
- Prototype code quality: throwaway by declaration, kept only for reference.

## Decisions

### D1: Sequential bake-off with early exit, gpui first
Build the gpui prototype first — it has the highest ceiling for text work (it is
the engine Zed is built on) and best maturity on macOS. If it meets every
requirement in the spec and the development experience is tolerable, gpui wins
and the iced prototype is skipped. Only if gpui fails a requirement or proves
unworkable is the iced prototype built. Each prototype is timeboxed to one week.

*Alternative considered:* building both regardless, for comparative data.
Rejected: for a solo part-time project a saved week outweighs comparison value;
the spec-based criteria make a single passing candidate sufficient.

### D2: egui is not prototyped
The concept already judges egui's typography too weak for the final "Typora
feel". Prototyping it would spend a week to confirm a known conclusion. It
remains only a fallback signal: if both gpui and iced fail, the approach itself
(hand-built rich text editing) must be reconsidered, not just the framework.

### D3: Hand-written markdown-lite parser as a shared crate
A line-based parser (~200 lines) in its own crate inside the `prototypes/`
workspace, returning styled spans with byte ranges per line: heading level,
bold, italic, and marker ranges. Unit-tested independently of any GUI.

*Alternative considered:* pulldown-cmark. Rejected for now: it parses full
CommonMark (we must then suppress most of it) and mapping its events back to
in-source byte ranges for in-place rendering is more work than parsing three
constructs directly. If edge cases pile up, revisit.

### D4: Line-scoped marker reveal
Raw markers are revealed for the whole line under the cursor (Typora behavior),
not per-span. Line scope keeps the parser and cursor mapping simple: rendering
is a pure function of (line text, cursor-on-line?).

### D5: Evaluation protocol
Each prototype is checked against every requirement in
`specs/live-markdown-editing/spec.md` scenario by scenario, plus one subjective
gate: a real 20-minute writing session in the prototype ("do I want to write in
this?"). Results go into the Decision Record table below. IME/Cyrillic input is
tested in the first two days — it is framework-plumbing and a hard gate, so it
must fail fast if it fails.

### D6: Workspace layout
`prototypes/` is a separate Cargo workspace (own `Cargo.toml`, own lockfile),
excluded from the future main crate. Members: `markdown-lite` (parser),
`gpui-editor`, and `iced-editor` (if built). gpui is consumed as a pinned git
revision of `zed-industries/zed`; the pin is frozen for the life of the spike.

### Invariant impact (required note)
- **WIP limit = 1:** untouched — no essay lifecycle exists in this change.
- **Write/Edit mode separation:** no mode switch is built, but typewriter mode
  — the backbone of Write mode — is validated here; the chosen framework must
  carry both modes later.

## Risks / Trade-offs

- [gpui is young: unstable API, thin docs] → pin one git revision, read Zed's
  own editor code as documentation, hard one-week timebox.
- [iced has no rich-text editing widget] → if the iced path is taken, build the
  editor widget directly on cosmic-text; accept prototype-level roughness.
- [IME or Cyrillic breaks at the framework level] → test on day 1-2; a failure
  here disqualifies the candidate immediately instead of surfacing in week two.
- [Sunk-cost temptation to polish the prototype into "almost the product"] →
  prototypes are declared throwaway in the proposal; the change is done when the
  Decision Record is filled, not when the prototype is pretty.
- [Early exit picks gpui without seeing iced] → accepted trade-off (D1); the
  spec scenarios, not impressions, are the bar the winner must clear.

## Decision Record

Filled at the end of the spike; empty until then.

| Requirement | gpui | iced |
|---|---|---|
| In-place markdown-lite rendering | — | — |
| Raw markers only at editing point | — | — |
| Markdown-lite is the whole syntax | — | — |
| Typewriter mode | — | — |
| Baseline editing correctness | — | — |
| Cyrillic / IME | — | — |
| Responsive at essay size | — | — |
| Subjective: 20-minute writing session | — | — |

**Chosen framework:** _pending_

**Rationale:** _pending_

## Open Questions

- Exact gpui revision to pin (resolve on day one of the spike).
- Whether the winning prototype's editor code seeds the production crate or is
  rewritten clean — decide after seeing its quality at the end of the spike.
