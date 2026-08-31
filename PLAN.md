# esse — implementation plan

From an empty repository to the full spark → published conveyor.
The ground is [CONCEPT.md](CONCEPT.md); the process is OpenSpec (see
[README.md](README.md)).

---

## Principles

1. **Kill the main risk first.** Live markdown rendering is the most expensive
   part of the project. Until the "Typora feeling" is proven reachable on the
   chosen framework, building anything else is premature.
2. **Every stage ends with an app you can actually use.** A tool "for myself"
   means dogfooding from the first weeks: the spark box first (cheap, useful
   immediately), then writing, then the whole conveyor.
3. **One stage = one OpenSpec change** (`/opsx:propose` → `/opsx:apply` →
   `/opsx:archive`), with tasks of at most two hours each.

---

## Stage 0 — Editor prototype and framework choice ✅ done

**gpui was chosen** (the Zed engine), pinned at `v1.17.2`. The prototype met
every criterion on the first attempt, so by the early-exit rule the iced
prototype was never built — a week saved. The decision and its grounds are in
the Decision Record `openspec/changes/editor-framework-prototype/design.md`.

What survived the stage: the `markdown-lite` parser moves into production as
is, and the editor is rewritten from scratch — the prototype has no line
wrapping, and wrapping reaches too deep into layout, mouse hit-testing and
typewriter centring to be bolted on afterwards.

The original plan was this:

- **gpui** first — the highest ceiling for text (the Zed engine); if it works,
  the "Typora feeling" is guaranteed.
- **iced** second — the fallback with a stable API; the editor widget would
  have to be written on top of cosmic-text.
- **egui** not to be prototyped at all — only if both fail, and then the whole
  approach gets reconsidered.

The prototype's success criteria (which are also the choice criteria):

- typing with no lag, Cyrillic and IME working;
- `#`, `*italic*`, `**bold**` rendering in place, with markers hidden outside
  the current line (Typora behaviour);
- typewriter mode: the cursor stays centred on the screen;
- selection, clipboard and undo not falling apart;
- the subjective test: "I want to write in this for twenty minutes".

Markdown-lite simplifies the job a lot — only headings and two inline styles
are needed, no full CommonMark rendering (the span parser can be written by
hand or pulldown-cmark used for parsing only). Prototype code lives in
`prototypes/`; the decision is recorded in the design document of the first
"real" change.

## Stage 1 — App skeleton, data model, spark box (~1 week)

- The domain model: `Spark`, `Essay` (states Draft → Editing → Published /
  Shelved), `Session`. **The WIP = 1 invariant lives in the storage layer**,
  not in the UI.
- Storage — plain files in the data directory: `essays/<slug>.md` with TOML
  front matter (status, dates, publication link); sparks and sessions as JSONL.
  Human-readable, survives any refactoring, syncs with anything; SQLite only if
  files ever stop being enough.
- A minimal Today screen: the spark input line and the list of sparks. From
  this moment the app is already useful every day — the box fills up with
  material for dogfooding.

## Stage 2 — Writing mode: the heart of the app (~2–4 weeks)

- The stage 0 prototype becomes the production editor widget: autosave on every
  pause, robustness on large texts.
- An essay starts only from a spark (the Write button → pick a spark; if the
  slot is taken, the essay continues). No "empty document" screen exists — that
  is enforced at the routing level.
- Typewriter mode, fullscreen, a quiet session timer in the corner with a
  gentle ending. Sessions are written into history right away — the dotted
  calendar comes later.
- Going back to edit is limited **by awkwardness only** (only the current
  fragment is visible), with no mechanical ban on the cursor: it is simpler,
  and tightening is possible after living with it.

After this stage the conveyor works halfway: spark → draft, and real writing is
possible.

## Stage 3 — Editing mode and the switch (~1–2 weeks)

- The whole text, a calm layout, ordinary navigation and editing.
- The Writing / Editing switch is an explicit action; the modes are visually
  contrasted (a dark flow against a light page, say) so the separation of the
  two processes is felt physically.

## Stage 4 — Publishing and the Shelf: the conveyor closes (~1–2 weeks)

- Finishing an essay: **published** (copy as markdown + export to a file — both
  trivial, so both are done; plus the "publication link" field) or deliberately
  **shelved**.
- The Shelf screen: Sparks | In progress | Published, plus the collapsed
  drawer. An essay can be started from a spark only while the slot is free.
- The WIP slot is freed on publishing or shelving.

**From this point the app fully does its job** — the concept's "first week" can
be lived through for real.

## Stage 5 — Rhythm and the showcase (~1 week)

- The dotted line of the last weeks' sessions on Today — no streaks, no guilt.
- The row of published essays as cards — the main display of progress.

## Stage 6 — Polish from dogfooding (as needed)

- First-run onboarding: the request for 3–5 sparks, and that is the only
  onboarding there is.
- Typography polish, shortcuts, launch speed (for "a spark in five seconds" the
  app has to open instantly). Measurement closed the question: ~0.5–0.8 s cold
  and ~0.15 s warm to being ready to type — the fast companion window for
  sparks was not needed.
- A revision of the concept's open questions against live experience: should
  the WIP limit be loosened, should Writing mode be tightened. The result is in
  [CONCEPT.md](CONCEPT.md#open-questions).

---

## Recommendations on the open questions

They were recorded here so they would not be decided in code; at stage 6 they
were revisited against live experience and became decisions. The details are in
[CONCEPT.md](CONCEPT.md#open-questions).

| Question | Decision after dogfooding |
|---|---|
| Framework | **gpui** — decided at stage 0, every criterion met |
| How hard the WIP limit is | Stays hard. If it starts to get in the way — a second slot with friction (the current essay explicitly shelved), as its own change |
| Export | Both copying and a file — both built, question closed |
| Editing backwards in Writing mode | Stays merely awkward; no mechanical ban needed |
| Launch speed | ~0.5–0.8 s cold, ~0.15 s warm — inside the one-second budget, no companion window for sparks needed |

---

## In total

The full conveyor is roughly two to three months part-time. The app starts
being useful at stage 1, and the main technical risk is retired in the first
two or three weeks.
