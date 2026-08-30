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

## Spike notes

Facts found while building the gpui prototype. Not verdicts — input for the
Decision Record, since D1 counts "development experience is tolerable" as part
of the bar.

- **Toolchain is pinned twice over.** gpui at the pinned revision uses
  `std::hint::cold_path`, stable only from Rust 1.97. `prototypes/` therefore
  carries a `rust-toolchain.toml` matching Zed's own pin (1.97.1). Bumping the
  gpui revision later may drag the toolchain with it.
- **macOS needs Xcode's Metal Toolchain component.** Xcode 26 no longer ships
  the `metal` compiler by default, and gpui's build script compiles shaders, so
  a build fails on a clean machine until
  `xcodebuild -downloadComponent MetalToolchain` (~700 MB) has been run. Worth
  writing into any future setup instructions.
- **First build is expensive**, roughly 700 dependencies including the whole
  Zed platform layer; incremental rebuilds of prototype code are fast.
- **`gpui_platform` needs `features = ["font-kit"]` or nothing renders.** The
  feature is on by default in `gpui` but *not* in `gpui_platform`, which is
  what actually pulls `gpui_macos`. Without it the app runs, lays text out with
  correct metrics, paints quads — and draws no glyphs at all. The only signal
  is a `log::warn!` ("compiled without the `font-kit` feature, so no text will
  be rendered"), invisible unless the binary installs a logger. Cost us an
  hour; the prototype now calls `env_logger::init()` so gpui's own warnings are
  never silently dropped again. A sharp edge worth weighing under D1's
  "development experience is tolerable".
- **The API held up.** The editor is a custom `Element` (`request_layout` /
  `prepaint` / `paint`) plus `EntityInputHandler` for text and IME. Zed's
  `crates/gpui/examples/input.rs` is the working reference for both; it covers
  marked-text composition, which is what the Cyrillic/IME gate rests on. The
  prototype compiled against the API without fighting it.
- **Heading size is kept on the cursor line.** A strict reading of task 1.4
  would draw the raw line at body size, but then moving onto a heading resizes
  it and the page jumps under the caret. The view keeps the heading's size and
  reveals its markers — Typora's actual behaviour. Emphasis still flattens to
  plain on the cursor line, as specified.

## Decision Record

| Requirement | gpui | iced |
|---|---|---|
| In-place markdown-lite rendering | pass — tests + app | not built |
| Raw markers only at editing point | pass — tests + app | not built |
| Markdown-lite is the whole syntax | pass — tests | not built |
| Typewriter mode | pass — app | not built |
| Baseline editing correctness | pass — tests + app | not built |
| Cyrillic / IME | pass — app | not built |
| Responsive at essay size | pass — 0.27 ms/frame + app | not built |
| Subjective: "do I want to write in this?" | pass — author | not built |

*Basis.* "tests" means the 69 tests in `prototypes/` cover the rule — the parser
and render plan, the buffer's movement/selection/clipboard/undo, the
source↔screen offset mapping, and a per-frame layout budget measured on a
20,000-character document (0.27 ms against a 16 ms frame, debug build). "app"
means confirmed by the author in the running prototype. The scenario walk was
not logged line by line, and the subjective gate was a first sitting rather than
a timed twenty minutes; the author judged the result sufficient to decide.
Recorded this way so a later reader knows which rows rest on tests and which on
one person's judgement.

**Chosen framework:** **gpui**, pinned at `zed-industries/zed` `v1.17.2`
(`c8e44cfa7bda9b2e22c8d6934d78969352e7f61a`).

**Rationale:** gpui met every requirement in the spec on the first prototype, so
D1's early exit applies and the iced prototype is not built — a saved week, as
intended. The API carried the two things that actually decide this project:
per-line styled runs with real glyph metrics (`shape_line` + a custom
`Element`), and IME composition through `EntityInputHandler`, which is the gate
a hand-built editor most often fails. Nothing in the spike had to fight the
framework; the editor compiled against the API essentially first try.

The cost is set-up friction rather than capability: a pinned Rust toolchain, a
~700 MB Xcode component, and a non-default Cargo feature whose absence renders
no text at all with only a `log::warn!` to say so (all three in Spike notes
above). These are one-time and now documented in `prototypes/README.md`. They
are worth weighing again only if gpui's instability starts costing time
repeatedly — the pinned revision is the hedge against that.

**Not chosen:** iced was never built (D1 early exit). egui remains unprototyped
per D2. If gpui later proves untenable, iced is the fallback and the spec plus
`markdown-lite` carry over unchanged — which is the point of keeping the parser
framework-free.

## Open Questions

- ~~Exact gpui revision to pin~~ — **resolved:** `zed-industries/zed` tag
  `v1.17.2`, commit `c8e44cfa7bda9b2e22c8d6934d78969352e7f61a`. A release tag
  rather than `main`, so the pin corresponds to a shipped, known-good Zed build.
- ~~Whether the winning prototype's editor code seeds the production crate or is
  rewritten clean~~ — **resolved: partly seeds, partly rewritten**, split by
  which layers earned it.

  *Carries over as-is:* `markdown-lite`. It was written to survive (D3), is
  framework-free, and its 22 tests are the spec's rules in executable form.

  *Carries over with known debts:* `buffer.rs` and `display.rs` — sound designs
  with 42 tests between them, but three prototype shortcuts must be paid off
  before they are load-bearing: movement steps by `char` rather than grapheme
  cluster (breaks on emoji sequences, fine for Cyrillic); undo snapshots the
  whole document per step; every edit re-indexes all line starts. All three are
  invisible at essay size (0.039 ms per keystroke measured) and all three are
  wrong at book size.

  *Rewritten:* the gpui view layer in `gpui-editor/src/main.rs`. It hand-rolls
  layout, and it has no soft wrapping at all — a long line simply runs past the
  measure. Wrapping changes the shape of line layout, hit-testing and typewriter
  centring deeply enough that the production editor should be built around it
  from the start rather than have it retrofitted.
