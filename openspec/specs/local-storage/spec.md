# local-storage Specification

## Purpose

The on-disk contract: where esse keeps its data and in what formats — the
platform data directory, essays as `essays/<slug>.md` with TOML front matter,
sparks and sessions as JSONL.

Plain human-readable files, no database, no accounts, no sync. These formats
are load-bearing for every later stage, so the round-trip and durability rules
below are part of the contract, not implementation detail.

## Requirements

### Requirement: Data directory
The app SHALL keep all data as plain files in the platform data directory
(on macOS: `~/Library/Application Support/esse`), creating it on first
launch. The `ESSE_DATA_DIR` environment variable SHALL override the location
for development and tests.

#### Scenario: First launch creates the data directory
- **WHEN** the app starts and the data directory does not exist
- **THEN** the directory is created and the app proceeds normally

#### Scenario: Override for tests
- **WHEN** `ESSE_DATA_DIR` is set to a path
- **THEN** all reads and writes use that path instead of the platform default

### Requirement: Essay file format
Each essay SHALL be stored as `essays/<slug>.md`: TOML front matter between
`+++` fences (fields: `status`, `created_at`, `updated_at`, and when present
`published_at`, `publication_url`, `spark`), followed by the markdown-lite
body. Parsing then serializing an essay file MUST reproduce its content,
including fields the current version does not understand.

#### Scenario: Essay round-trips through the store
- **WHEN** an essay file is read and written back without modification
- **THEN** the resulting file content is identical, unknown front-matter fields included

#### Scenario: Essay writes are atomic
- **WHEN** an essay is saved
- **THEN** the file is replaced via write-temp-then-rename so no partially written essay file can exist

### Requirement: Sparks stored as JSONL
Sparks SHALL be stored in `sparks.jsonl`, one JSON object per line with `id`,
`text`, and `created_at`. Capturing a spark appends a line; any future
mutation rewrites the file atomically.

#### Scenario: Capture appends one line
- **WHEN** a spark is captured
- **THEN** exactly one JSON line is appended to `sparks.jsonl` and previously stored lines are untouched

#### Scenario: Sparks survive restart
- **WHEN** the store is reopened on an existing data directory
- **THEN** it returns every previously captured spark

### Requirement: Sessions stored as JSONL
Sessions SHALL be stored in `sessions.jsonl`, one JSON object per line with
`essay_slug`, `started_at`, and `duration_min`. This change defines the
format and round-trip code; nothing records sessions yet.

#### Scenario: Session record round-trips
- **WHEN** a session record is written and the file is read back
- **THEN** the parsed record equals the original
