# Tasks: App Skeleton, Domain Model, Spark Inbox

## 1. Workspace and app shell

- [x] 1.1 Create the root Cargo workspace with `crates/esse-core` and
      `crates/esse-app`; pin the Rust toolchain (1.97.1) at the root; keep
      `prototypes/` excluded as its own workspace; verify `cargo test` runs
      green on the empty crates.
- [x] 1.2 Add gpui to `esse-app` pinned at `zed-industries/zed` `v1.17.2`
      with the `font-kit` feature and `env_logger::init()` (stage-0 caveats);
      open a single empty window titled "esse" that closes cleanly.

## 2. Domain model (esse-core)

- [x] 2.1 Define `Spark`, `Session`, `Essay`, and `EssayStatus` (Draft,
      Editing, Published, Shelved) with an `is_in_progress` predicate; unit
      tests for the type surface.
- [x] 2.2 Implement the transition rules (Draft ↔ Editing, in-progress →
      Published / Shelved, everything else rejected) as the only way to
      change status; tests cover each legal and each rejected transition.

## 3. Storage (esse-core)

- [x] 3.1 Implement `DataDir`: platform data-directory resolution via `dirs`,
      `ESSE_DATA_DIR` override, create-on-first-use, and an atomic
      write-temp-then-rename helper; tests use a temp dir via the override.
- [x] 3.2 Implement the spark store on `sparks.jsonl`: append on capture
      (flushed), load-all newest first; tests for append, ordering, and
      reopening an existing file.
- [x] 3.3 Implement essay front-matter serialization: `+++` TOML fences with
      the D4 field set, body passthrough; round-trip tests including
      preservation of unknown fields.
- [x] 3.4 Implement the essay store: `create` (refusing when any essay is
      Draft/Editing, per the WIP = 1 spec), `load`, `save` via the atomic
      helper; tests for refusal with slot occupied and creation with slot
      free.
- [x] 3.5 Implement session record round-trip on `sessions.jsonl`
      (`essay_slug`, `started_at`, `duration_min`); round-trip test. Nothing
      calls it from the app yet.

## 4. Today screen (esse-app)

- [x] 4.1 Build the single-line spark input as a gpui element with
      `EntityInputHandler`: text, cursor, backspace, IME composition
      (Cyrillic verified by hand), Enter submits, whitespace-only submits
      ignored. No selection/clipboard/undo.
- [x] 4.2 Build the spark list view: flat, newest first, scrollable; wire the
      Today view to load sparks from the store on launch.
- [x] 4.3 Wire capture end-to-end: Enter persists via the spark store, clears
      the input, and the new spark appears at the top of the list; input has
      focus on launch (launch → type → Enter works with no click).

## 5. Verification

- [x] 5.1 Walk every spec scenario against the running app and the test
      suite; fix gaps; `cargo test` green in both crates.
- [x] 5.2 Update README with production-crate build/run steps (toolchain,
      Metal Toolchain note) and start daily dogfooding: capture real sparks.
