## Why

The conveyor has one slot, and a stalled essay stops the whole line. When the
open essay has gone cold — the angle was wrong, the idea needs to sit — the
only legal moves are to publish something unfinished or to shelve it for good.
Both are lies about what happened, so the real move is a third one: not
writing. That is beginner problem #4 ("gives up after two weeks") arriving
through the front door of the mechanic meant to prevent it.

Six weeks of daily use have named the shape of it: ideas do not queue politely
behind one another, and an essay that needs to rest is not an essay that needs
to be buried. This change gives the conveyor three lanes instead of one.

Serves beginner problem #4 primarily, and #2 ("cannot start") secondarily: a
cold essay is a blank page wearing a different hat, and the answer to both is
that there is always something warm to open.

## What Changes

- **BREAKING (invariant).** The WIP limit rises from 1 to 3. At most three
  essays may be in Draft or Editing at once; the storage layer keeps enforcing
  the limit, it simply counts to three. `Error::EssayInProgress` becomes a
  refusal about the limit rather than about one named essay.
- **The "Write" button always asks what to work on.** Pressing Write opens one
  chooser with the open essays first (each showing its state) and the sparks
  below them. `enter` alone takes the top entry — the essay worked on most
  recently — so the common case stays launch → Write → Enter → typing. The
  routing rule that no editor opens without an essay is untouched.
- **An empty conveyor and a full one are the two ends of the same chooser.**
  With nothing open, the chooser is the spark list exactly as it is today. With
  three open, the sparks offer no start action and one quiet line says why.
- **The Shelf's "In progress" column holds up to three cards** instead of zero
  or one, ordered most recently worked first.
- **Completion frees one lane, not the whole conveyor.** Publishing or shelving makes
  room for one more essay; the other open essays are untouched.
- The concept documents that call WIP = 1 a hard invariant — `CONCEPT.md` and
  `openspec/config.yaml` — are rewritten to say three, with the reasoning kept
  rather than deleted.

## Capabilities

### New Capabilities

None. This change loosens an existing invariant and reshapes the screens that
express it; it introduces no new capability of its own.

### Modified Capabilities

- `essay-lifecycle`: the WIP requirement changes from "refuse while any essay is
  in progress" to "refuse while three are in progress", and the error names the
  limit. The four states and the legal transitions are unchanged.
- `start-from-spark`: the "Write" button routes to a chooser in every case
  instead of branching on whether the slot is occupied; the chooser lists open
  essays and sparks together and is keyboard-operable; the WIP refusal surfaced
  here becomes the refusal about the limit.
- `shelf-screen`: the "In progress" column holds up to three cards with a
  defined order; sparks stop offering a start action only when three essays are open;
  keyboard navigation moves through a column that can now have several cards.
- `essay-completion`: completion frees one lane rather than emptying the conveyor,
  and the post-completion scenarios describe what remains open.

## Non-goals

- **No fourth lane, and no way to ask for one.** Three is a wall, not a default
  to be raised in settings. The graveyard of ten half-started drafts is the
  thing the original limit exists to prevent, and it still does.
- **No organisation of the open essays.** No folders, no tags, no ordering
  options, no renaming, no pinning — three items sorted by when they were last
  worked need none of it (anti-feature: folders, tags, nesting).
- **No counts or progress metrics on screen.** The limit is invisible until it
  is reached; there is no "2 / 3" badge, no word counts, no per-essay progress
  (anti-feature: statistics and graphs).
- **No reviving shelved essays.** Shelved stays terminal. A cold essay now has
  somewhere to wait, which is precisely why it need not be shelved by mistake.
- **No change to the Write/Edit separation.** Each open essay carries its own
  state and opens in the room matching it, exactly as one essay does now.
- **No change to sessions.** A session already records its `essay_slug`, so
  session history needs nothing from this change.

Checked against the anti-features list: this adds no AI, no toolbars, no
folders or tags, no social features, no statistics, and no settings. The only
thing it adds is two more lanes on a conveyor that already had one.

## Impact

Code:

- `crates/esse-core/src/store/essays.rs` — `create` counts in-progress essays
  against the limit; `in_progress()` (singular) becomes a list.
- `crates/esse-core/src/error.rs` — `EssayInProgress` is re-shaped to speak of the limit rather
  than of one named essay.
- `crates/esse-core/src/start.rs` — passes the new refusal through unchanged.
- `crates/esse-app/src/root.rs` — `write_pressed` stops branching and opens the
  chooser; the `ShelfEvent::Continue` handler carries a slug.
- `crates/esse-app/src/today.rs` — the spark chooser becomes the work chooser.
- `crates/esse-app/src/shelf.rs` — the In-progress column and its keyboard
  highlight handle several cards.
- `crates/esse-core/tests/` — `storage.rs` and `pipeline.rs` assert WIP = 1 in
  several places and are rewritten against the new limit.

Data: none. The on-disk format is unchanged — the limit was never recorded
anywhere, it was counted from the essay files, and it still is. A data folder
written by the current version opens unchanged, and one with three open essays
opens in an older build as a folder whose limit that build will refuse to
respect rather than one it cannot read.

Docs: `CONCEPT.md` (the WIP limit, the open question about loosening it, the
Shelf's description) and `openspec/config.yaml` (the hard invariant in the
project context, which governs every future change).
