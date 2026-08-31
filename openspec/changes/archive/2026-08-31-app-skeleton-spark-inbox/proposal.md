# App Skeleton, Domain Model, Spark Inbox

## Why

Stage 0 settled the framework (gpui, pinned `v1.17.2`) and proved the editor
risk is manageable. Nothing usable exists yet: no production crate, no data
model, no way to capture an idea. This change builds the smallest app that is
useful every day — a spark inbox — so dogfooding starts now and the essay
pipeline has a foundation (domain model, storage, app shell) for every later
stage.

Of the five beginner problems, this change directly serves **#1 "nothing to
write about"**: sparks get captured the moment they occur instead of being
lost. It also lays the ground for **#3 "drafting and editing get mixed" / #4
"gives up after two weeks"** by enforcing the WIP = 1 invariant in the storage
layer from day one — before any UI could grow around a looser rule.

## What Changes

- New production Rust crate (the real app, separate from the `prototypes/`
  workspace) built on gpui pinned at `zed-industries/zed` `v1.17.2`, macOS
  first.
- Domain model: `Spark`, `Essay` (states Draft → Editing → Published /
  Shelved), `Session`. The **WIP = 1 invariant lives in the storage layer**,
  not in the UI: the store refuses to create a second in-progress essay.
- File-based local storage in the platform data directory: essays as
  `essays/<slug>.md` with TOML front matter (status, dates, publication link);
  sparks and sessions as JSONL. Human-readable, sync-agnostic, no database.
- Minimal "Today" screen: a quick spark input line plus the list of captured
  sparks. This is the whole UI of this stage.

## Capabilities

### New Capabilities

- `essay-lifecycle`: essay states (Draft, Editing, Published, Shelved), legal
  transitions, and the WIP = 1 invariant enforced at the storage layer.
- `local-storage`: on-disk layout and formats — data directory location,
  `essays/<slug>.md` with TOML front matter, sparks and sessions as JSONL;
  durability and human-readability rules.
- `spark-capture`: capturing a spark as a single line in seconds and listing
  captured sparks, newest first.
- `today-screen`: the app's main (and for now only) screen — spark input plus
  spark list; the frame later stages extend with the "Write" button, published
  row, and session dots.

### Modified Capabilities

None. `live-markdown-editing` is untouched — the production editor is the next
stage.

## Impact

- **Code:** new top-level app crate (workspace root); `prototypes/` remains a
  separate excluded workspace. The `markdown-lite` parser stays in
  `prototypes/` for now — it moves when the production editor is built.
- **Dependencies:** gpui (git pin `v1.17.2`), a TOML parser, serde/serde_json
  for JSONL. Toolchain pin (Rust 1.97.1) and macOS Metal Toolchain caveats
  from stage 0 apply to the production crate too.
- **Data:** creates the user-visible data directory; formats defined here are
  load-bearing for every later stage, so they get their own spec.

## Non-goals

- No editor, no Write/Edit modes, no session timer or session recording —
  stage 2. (`Session` exists as a type and a storage format only.)
- No "Write" button flow, no essay creation UI — starting an essay from a
  spark is stage 2; this stage only makes the invariant that will guard it.
- No Shelf screen, no publishing/export — stages 3–4.
- No SQLite, no accounts, no sync — plain local files by design.
- Per the anti-features list: no tags, folders, or ordering options for
  sparks (a flat newest-first list), no statistics, no settings, no
  onboarding.
