# write-several-essays — Design

## Context

WIP = 1 is the oldest opinionated decision in the product and the most
load-bearing: `openspec/config.yaml` calls it a HARD INVARIANT, `CONCEPT.md`
argues for it, and `EssayStore::create` enforces it in the storage layer
precisely so that no screen could ever grow around it (essay-lifecycle spec:
"The invariant is enforced in the storage layer rather than in the UI, so no
screen or future API can grow around a looser rule").

That enforcement point is why this change is small. The limit was never a
number stored anywhere; it was the predicate `is_in_progress` applied to the
essay files, asked in one function. Everything else — the Write button's
branch, the Shelf's single card, the completion flow's "the slot is free
because nothing counts as in progress any more" — reads that one answer.

The concept anticipated this moment and wrote down the shape of the loosening
it expected: "it is loosened not with a free second slot but with a **slot with
friction**: to start a new essay the current one has to be shelved by an
explicit action." That sketch is being overruled rather than followed, and the
reason is in D1.

Constraints carried in unchanged: the on-disk format (`local-storage`), the
rule that no route reaches the editor without an essay (`start-from-spark`),
and the Write/Edit separation (`write-mode`, `edit-mode`).

## Goals / Non-Goals

**Goals:**
- Three essays may be in Draft or Editing at once, refused at the same single
  point in the storage layer that refuses the second one today.
- Pressing Write always answers the question "what am I working on?" in one
  list, with the common case still Write → Enter → typing.
- The Shelf shows all the open essays without becoming a project manager.
- The concept documents stop asserting an invariant the code no longer holds.

**Non-Goals:**
- A configurable limit. Three is compiled in as a constant with a comment, not
  a setting (anti-feature: settings and customization).
- Per-essay metadata of any kind — no titles beyond the slug and the spark, no
  ordering controls, no progress marks.
- Any change to how a session is recorded, to autosave, or to either room's
  behaviour once an essay is open.
- Reviving shelved essays, or any new transition in the lifecycle.

## Decisions

### D1: Three lanes, not one lane with friction

The concept's sketched loosening — a second slot that costs an explicit
shelving — is rejected. It prices switching in the one currency the writer
must never be asked to spend: declaring an unfinished essay dead. Shelving is
specified as terminal (`shelf-screen`: "no action on a shelved essay SHALL
change its state"), so "shelve the current one to start the next" means every
switch destroys an essay. The friction lands on exactly the wrong action.

Three plain lanes put the friction where it belongs — at the fourth essay,
where the graveyard actually starts — and leave switching free, which is the
thing being bought.

*Why three:* one is a jam; two is one warm essay and one cold one, which is the
problem restated rather than solved; three fits the rhythm the concept targets
(~3 sessions a week ≈ an essay every one or two weeks) so that a stalled essay
can rest a full cycle without blocking the next two. Beyond three the list
stops being something a person holds in their head, which is the moment the
limit stops protecting anything.

*Alternative considered — no limit:* rejected. The limit is not a technical
constraint to be relaxed but a mechanic: it is what makes "I will just start
another one" cost something. With no wall there is no difference between three
essays and the graveyard of ten the WIP limit exists to prevent.

*Invariant impact (required note):* this is the change to the WIP invariant.
It moves from 1 to 3 and stays hard — enforced in `EssayStore::create`, never
in the UI, with no bypass and no setting. The Write/Edit separation is
untouched: the two rooms, the switch between them, and each essay's Draft ↔
Editing state work exactly as they do now, three times over instead of once.

### D2: `WIP_LIMIT` is one constant, read in one place

`pub const WIP_LIMIT: usize = 3` lives in `esse-core::model`, next to
`EssayStatus`, and is read by `EssayStore::create` and by nothing else that
decides. Screens ask the store what is open and whether there is room; they
never count to three themselves.

*Rationale:* the current design's strength is that there is exactly one
sentence in the codebase that can refuse. Raising the limit must not multiply
that sentence — otherwise the next change has three places to keep in step.

### D3: `in_progress()` returns a list, ordered most recently worked first

`EssayStore::in_progress() -> Result<Option<Essay>>` becomes
`-> Result<Vec<Essay>>`, filtering `load_all()` rather than `find`-ing in it.
`load_all` already sorts by `updated_at` descending, so "most recently worked
first" is free and, because autosave touches `updated_at`, it means what the
writer expects: the essay they were last typing into is first.

The compiler finds every caller — `shelf.rs`, `root.rs`, the two test files —
which is the point of changing the signature rather than adding a second
method beside it. A stray `in_progress()` that still returns one essay is
exactly the loosened-rule-grown-around-the-invariant this codebase avoided.

*Alternative considered:* keep `in_progress()` singular for "the most recent"
and add `all_in_progress()`. Rejected: two functions differing by one word,
one of which quietly ignores two essays, is a bug waiting for a tired reader.

### D4: One chooser, open essays above sparks

Pressing Write no longer branches on whether the slot is occupied. It always
opens the Today screen's choosing state, which now renders two sections:
**Continue** (the open essays, each showing Draft or Editing) and **Start from
a spark** (the spark box, newest first). The highlight opens on the first
entry — the most recently worked essay, or the newest spark when nothing is
open — so `enter` alone does the overwhelmingly common thing.

This is a widening of the existing `picking` state, not a new screen: the
highlight becomes an index into one flat `Vec<Choice>` built from the two
sections, `up`/`down`/`enter`/`escape` keep their current handlers, and
`TodayEvent` gains `Continue(String)` beside `Start(String)`.

*Rationale:* with several essays open there is no defensible default, and a
button that silently picks one of three is a button that opens the wrong essay
often enough to be distrusted. The alternative — Write continues the newest and
switching lives elsewhere — was weighed and set aside: it hides the second and
third essays behind a screen the writer has to remember, which is how two of
the three go cold.

*Cost, stated plainly:* one keystroke per session in the single-essay case,
where Write used to open the editor directly. `enter` is that keystroke, on the
essay that was already going to be opened.

The two degenerate cases fall out of the same list and need no branch of their
own: nothing open and no sparks is an empty list, which keeps today's "a spark
comes first" message; nothing open and sparks present is the spark list exactly
as it is now.

### D5: The limit is invisible until it is reached

No badge, no "2 / 3", no counts on the Shelf's columns (which `shelf-screen`
forbids outright). When three essays are open, the sparks section of the
chooser and the Sparks column on the Shelf offer no start action — the same
disabling the occupied slot does today — and one quiet line says why: *"Three
essays are open. Publish or shelve one to make room."*

*Rationale:* a visible counter turns the limit into a score to manage. The
writer should meet the wall once, understand it in one sentence, and go on
writing. This also keeps the anti-feature list intact: the line is an
explanation at the moment of refusal, not a statistic on display.

### D6: The refusal names the limit, not a single essay

`Error::EssayInProgress { slug, status }` becomes
`Error::TooManyInProgress { limit: usize }`. Naming one of three essays when
any of them could be finished is misdirection; the useful sentence is the
count and the way out.

`start-from-spark`'s "WIP refusal is surfaced, not swallowed" requirement
survives with its intent intact — the storage layer's refusal is still shown
rather than swallowed, and the UI still never bypasses it. What changes is what
the message says.

### D7: The Shelf's In-progress column grows a list, and nothing else

`Contents::in_progress` becomes `Vec<Essay>`; `columns()` already reports card
counts per column and `step_row` already clamps to them, so `up`/`down` work
over several cards with no new code. `ShelfEvent::Continue` gains the slug of
the card that was activated. `startable()` becomes `self.contents.in_progress.len() < WIP_LIMIT`.

The column stays a column of plain cards: no counts, no ordering options, no
drag, no rename — `shelf-screen`'s existing prohibition applies unchanged.

## Risks / Trade-offs

- **Three warm starts, three unfinished essays** → The wall at four is the
  mitigation, and it is the same mechanic as before, moved. If the pile still
  grows, the next change is to lower the limit back — a one-constant edit,
  which is why D2 keeps it to one constant.
- **The extra keystroke before writing erodes the launch promise** → Measured
  ground: launch to ready-to-type is ~0.15 s warm, and the chooser opens on the
  already-loaded Today screen. `enter` on a pre-placed highlight costs no disk
  read. If it grates in daily use, D4's alternative (Write continues the newest,
  a modifier opens the chooser) is a small, contained change.
- **Switching essays mid-thought becomes a new way to procrastinate** →
  Accepted deliberately, and watched. Switching is cheap on purpose; if it
  turns out to be the new avoidance, the answer is a rule about sessions (one
  essay per session), not a smaller limit.
- **An older build opening a three-essay folder** → It reads every file fine —
  the format is unchanged — and will refuse to create a fourth, then a third,
  then a second, according to its own limit. No data is lost or corrupted; the
  old build is simply stricter. No migration, no version field.
- **`CONCEPT.md` and `config.yaml` drifting from the code** → They are edited
  in the same change, not after it. The config's context block governs every
  future proposal, so leaving it saying "HARD INVARIANT: WIP limit = 1" would
  quietly instruct the next change to undo this one.

## Migration Plan

None required. The on-disk format, the data directory, and every existing file
are untouched; the limit is computed from the essay files at every call. The
change ships as a normal release, and the rollback is reverting the commit —
after which a data folder with three open essays keeps all three on disk and
the old build refuses new starts until two of them end.

## Open Questions

- **Does the chooser need to show more than the slug and the state?** A slug
  and "Draft" may not be enough to recognise which of three essays is which
  after a week away. The first line of the body is the obvious addition and is
  deliberately not in this change: the answer needs three real essays open for
  a fortnight first.
