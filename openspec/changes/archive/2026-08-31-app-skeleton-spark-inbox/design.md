# Design: App Skeleton, Domain Model, Spark Inbox

## Context

Stage 0 ended with a framework decision (gpui, pinned `zed-industries/zed`
`v1.17.2`) and a reusable `markdown-lite` parser, but no production code: the
repository holds only the `prototypes/` workspace. This change creates the real
application — the crate layout every later stage builds on, the domain model
with its central invariant, file-based storage, and the first useful screen
(spark capture). The stage-0 spike notes (toolchain pin 1.97.1, Metal
Toolchain component, `font-kit` feature) apply verbatim to the production
crate.

## Goals / Non-Goals

**Goals:**
- A production workspace whose core (domain + storage) is testable without
  building gpui.
- Domain model `Spark` / `Essay` / `Session` with the WIP = 1 invariant
  enforced in the storage layer, proven by unit tests.
- Human-readable on-disk formats that are load-bearing for all later stages.
- A minimal Today screen that makes spark capture a daily habit from now on.

**Non-Goals:**
- The editor, Write/Edit modes, session recording, essay creation UI, Shelf,
  publishing — later stages.
- Moving `markdown-lite` out of `prototypes/` — it moves with the editor.
- Any storage beyond plain files; any settings or customization.

## Decisions

### D1: Two-crate workspace, GUI-free core
Root Cargo workspace with `crates/esse-core` (domain model + storage, no gpui
dependency) and `crates/esse-app` (gpui shell and views). `prototypes/`
remains its own excluded workspace.

*Rationale:* gpui builds ~700 dependencies; the invariant and file formats are
the load-bearing logic and must be testable in seconds (`cargo test -p
esse-core`). The split also keeps the framework swappable, same reasoning that
kept `markdown-lite` framework-free in stage 0.

*Alternative:* one crate. Rejected — couples every core test to the gpui
build and invites UI types into the domain.

### D2: Files are the single source of truth; store re-reads on demand
The store holds a data-directory path, not an in-memory database. Reads scan
the files; writes go straight to disk. No cache, no index.

*Rationale:* at personal scale (hundreds of sparks, tens of essays) scanning
is instant, and statelessness means external edits — hand-editing an essay
file, syncing the directory — are picked up without an import step. This is
exactly what "human-readable files, survives any refactoring" in the plan is
for. Revisit only if profiling ever says otherwise.

### D3: WIP = 1 enforced by the essay store, not the UI
`EssayStore::create` determines the set of in-progress essays (status Draft or
Editing) by reading front matter, and refuses to create a new essay if that
set is non-empty. The UI never gets an API that could bypass this; stage 2's
"Write" button will call this same method and surface the refusal.

*Invariant impact (required note):* **WIP limit = 1** is implemented here, at
the storage layer, per the plan. **Write/Edit mode separation** is untouched —
no editor exists in this change; the `Draft ↔ Editing` states that will back
it are defined but only exercised by tests.

### D4: On-disk formats
- Data directory: platform data dir (`~/Library/Application Support/esse` on
  macOS) via the `dirs` crate, created on first launch. `ESSE_DATA_DIR`
  overrides it — a development/test affordance, not a user setting.
- Essays: `essays/<slug>.md`, TOML front matter between `+++` fences (Hugo
  convention — front matter is TOML per the plan, and `+++` unambiguously
  signals TOML where `---` conventionally means YAML). Fields: `status`,
  `created_at`, `updated_at`, optional `published_at`, `publication_url`,
  `spark` (the originating spark's text). Body: markdown-lite.
- Sparks: `sparks.jsonl`, one JSON object per line (`id`, `text`,
  `created_at`), appended on capture. Mutations (consuming a spark in stage 2)
  rewrite the file atomically — at this scale a rewrite is trivial and keeps
  each line a current record rather than an event to replay.
- Sessions: `sessions.jsonl` (`essay_slug`, `started_at`, `duration_min`).
  Format and round-trip code defined now; nothing writes it until stage 2.
- Timestamps: RFC 3339 with local offset, serialized as strings.

*Alternative:* SQLite. Rejected by the plan itself — files are
human-readable, sync-agnostic, and refactor-proof; SQLite only if files fall
short.

### D5: Atomic writes from day one
Every essay write is write-temp-then-rename in the same directory; JSONL
appends flush before returning. Stage 2's autosave-on-pause inherits this for
free, and a crash can never leave a half-written essay.

### D6: Today screen reuses the prototype's input plumbing pattern
The spark input is a hand-built single-line gpui field: `EntityInputHandler`
for text and IME composition (the Cyrillic gate), cursor, backspace, Enter to
submit. It deliberately skips selection/clipboard/undo — a one-line capture
box does not need them, and the real editor (stage 2) owns that complexity.
The prototype's `main.rs` and Zed's `input.rs` example are the references; the
code is written fresh in `esse-app` (the prototype view layer was declared
non-carrying in stage 0).

## Risks / Trade-offs

- [gpui pin ages while stages 1–4 are built] → stay on `v1.17.2` for the whole
  pipeline build; bumping is its own change with the toolchain caveat from
  stage 0 in mind.
- [Front-matter parsing is hand-rolled and quietly wrong] → use a real TOML
  crate for the fence contents; round-trip property: parse(serialize(x)) == x
  in tests; unknown fields are preserved, not dropped.
- [Two stores (sparks, essays) drift apart in style] → both live in
  `esse-core::store` behind one `DataDir` type that owns path resolution and
  atomic-write helpers.
- [Hand-built input field balloons into a mini-editor] → scope is fixed by
  spec: text, cursor, IME, submit. Anything more waits for the real editor.
- [WIP check races with concurrent instances] → single-user desktop app, one
  instance assumed this stage; noted as an open question, not solved.

## Open Questions

- Slug collision policy for essays (`<slug>` from spark text) — decide in
  stage 2 when essay creation is actually wired; store API reserves the right
  to suffix (`-2`).
- Single-instance guard (file lock on the data dir) — likely wanted before
  stage 2 autosave; not needed while the app only appends sparks.
