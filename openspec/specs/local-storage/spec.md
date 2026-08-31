# local-storage Specification

## Purpose

The on-disk contract: where esse keeps its data and in what formats — a
visible folder in the writer's documents, essays as `essays/<slug>.md` with
TOML front matter, sparks and sessions as JSONL.

Plain human-readable files, no database, no accounts, no sync. These formats
are load-bearing for every later stage, so the round-trip and durability rules
below are part of the contract, not implementation detail.
## Requirements
### Requirement: Data directory
The app SHALL keep all data as plain files in a folder the writer can see:
`Esse` inside the platform documents directory (on macOS
`~/Documents/Esse`), creating it on first launch. If the platform reports no
documents directory, the home directory SHALL be used instead. The
`ESSE_DATA_DIR` environment variable SHALL override the location for
development and tests.

#### Scenario: First launch creates the data directory
- **WHEN** the app starts and the data directory does not exist
- **THEN** `Esse` is created inside the documents directory and the app proceeds normally

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

### Requirement: One-time move out of the hidden directory
When the data directory does not yet exist and the app's previous location
(the platform data directory, on macOS `~/Library/Application
Support/esse`) does, the app SHALL move that directory to the new location
once, so an existing installation keeps its sparks, sessions and essays
without the writer doing anything. The app SHALL NOT merge two directories:
if the new location already exists, the old one is left untouched. A move
that fails SHALL stop the launch with a message naming both paths rather
than starting on an empty directory. When `ESSE_DATA_DIR` is set no move is
attempted, because that path was named deliberately.

#### Scenario: An existing installation moves itself
- **WHEN** the app starts with data in the old platform data directory and no directory at the new location
- **THEN** the files are moved to the new location, the app opens on them, and nothing is left behind at the old path

#### Scenario: Nothing to move
- **WHEN** the app starts with no old directory
- **THEN** the new directory is created empty and no move is attempted

#### Scenario: Both locations exist
- **WHEN** the app starts with data at both the old and the new location
- **THEN** the new location is opened as it is and the old directory is left untouched

#### Scenario: A named directory is never migrated into
- **WHEN** `ESSE_DATA_DIR` is set and an old platform directory exists
- **THEN** the named directory is used as it is and no move is attempted

