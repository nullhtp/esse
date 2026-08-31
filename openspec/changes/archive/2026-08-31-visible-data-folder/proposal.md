# visible-data-folder — Proposal

## Why

esse promises that the writing is kept as plain files a person can read, edit
and sync — that promise is what makes it safe to put the only copy of an essay
into a young app. Today the files sit in `~/Library/Application Support/esse`,
a directory macOS hides from the writer and Finder does not open by default,
and nothing in the app ever says where they are. The promise is technically
kept and practically invisible.

This serves none of the five beginner problems directly, and that is worth
saying plainly. It is upkeep of the storage contract the whole tool rests on:
a writer who cannot point at their essays does not fully believe they own
them, and the doubt costs more than any feature would.

## What Changes

- **The data directory moves to `~/Documents/Esse`** — a plain visible folder,
  next to everything else a person writes, backed up and synced by whatever
  they already use. `ESSE_DATA_DIR` keeps overriding it for development and
  tests.
- **A one-time migration** carries an existing `~/Library/Application
  Support/esse` to the new place on launch, so a running installation keeps
  its sparks, sessions and essays without the writer doing anything.
- **The Shelf says where the files are**: one quiet line under the drawer with
  the folder's path, and a key that opens it in Finder. The Shelf is where the
  whole body of work is already visible; the line belongs at its foot.
- **The readme says it too**, in the place that already describes the formats.

## Capabilities

### New Capabilities

None. This changes where an existing capability keeps its files and adds one
line to an existing screen.

### Modified Capabilities

- `local-storage`: the data-directory requirement names a visible documents
  folder instead of the platform data directory, and gains the one-time
  migration from the old location.
- `shelf-screen`: gains a requirement that the Shelf shows the data folder's
  path and can open it in Finder.

## Non-goals

- No folder picker, no "move my data" UI, no setting. One location, decided
  once; the environment variable stays a development affordance, not a user
  setting.
- No sync, no iCloud integration, no conflict handling — the folder is
  ordinary, and whatever the writer already syncs with treats it as ordinary.
- No file browser inside the app: the Shelf shows essays, Finder shows files,
  and neither grows into the other.
- No change to the file formats, the front matter, or the WIP = 1 invariant.

## Impact

- `crates/esse-core/src/store/data_dir.rs` — where the path comes from, and
  the migration.
- `crates/esse-core/src/error.rs` — a migration failure has to be sayable.
- `crates/esse-app/src/shelf.rs` — the line at the foot of the Shelf and the
  action that opens the folder.
- `crates/esse-app/src/keymap.rs` — one more row in the Shelf's keys, which
  the help sheet then lists on its own.
- `README.md` — the Data section names the new folder.
- No new dependencies: `dirs` already resolves the documents directory, and
  opening Finder is `open` through the platform.
