## Context

The app already has exactly one on-demand overlay: the shortcut sheet
(`help.rs`), summoned with `cmd-h`, dismissed by any keypress, listing the
keys of the innermost `keymap::Place`. Root tracks a `helping` flag, parks
focus on a dedicated handle while the sheet is up, and gives it back on
dismissal; keys bound `Scope::Everywhere` are excluded from the help context
(`"!Help"`) so that no key can act through the sheet.

The new surface is the same shape with different content: not *which keys
work here* but *how to work here* — the method the mechanics enforce, written
down where the writer can summon it. The method texts already exist in prose
form in CONCEPT.md (the five mechanics, the two modes, the first week); the
app just never says them.

## Goals / Non-Goals

**Goals:**
- A method overlay on `cmd-shift-h`, one instruction per `keymap::Place`,
  following every behavioral rule the shortcut sheet already obeys
  (on-demand, any-key dismissal without acting, presentation only).
- Instructions a beginner can work through rather than principles to agree
  with: a lead line on what the place is for, then the steps in order, each
  carrying its own reason, ending with what leaves the place.

**Non-Goals:**
- No merging with the shortcut sheet — `cmd-h` stays keys-only, per its spec.
- No first-launch appearance, no persistent hints, no AI, no settings.
- No per-essay or text-aware advice; the texts never read the draft.

## Decisions

### D1. A second overlay, not an extension of the shortcut sheet

The shortcut-help spec fixes its overlay as a keys listing, and the two
surfaces answer different questions at different reading speeds: keys are
glanced at mid-keystroke, method is read between sessions. Folding method
paragraphs into the keys sheet would make both worse and would need a
MODIFIED requirement on shortcut-help's core listing. Instead: a sibling
module (`guidance.rs`) mirroring `help.rs` — same veil, same card, same
dismissal contract — so the two sheets feel like one family.

*Alternative considered:* one sheet with two tabs or a second page. Rejected:
paging inside a transient overlay contradicts "any key dismisses".

### D2. Same dismissal contract as the shortcut sheet

Any keypress closes the guidance and performs nothing else. Having two
overlays with two different dismissal rules would make both unpredictable.
Mechanically this reuses the existing pattern: the guidance overlay gets its
own key context (`Guidance`), `Scope::Everywhere` bindings exclude it
(`"!Help && !Guidance"`), and the overlay's own `on_key_down` closes it.

*Consequence:* the sheet cannot scroll, so an instruction has to fit on
screen whole. Six steps is the working ceiling, and a test holds it at seven.

*Consequence:* `cmd-h` pressed over guidance dismisses guidance and does not
open the shortcut sheet, and vice versa — the sheets never stack.

### D3. Content is keyed by `keymap::Place`, one instruction each

Root already computes "where the writer is standing" for the shortcut sheet;
guidance reuses the same resolution so the two surfaces can never disagree
about context. Each of the six places gets one hand-written Russian `Method`
— a lead line plus an ordered list of steps — sourced from CONCEPT.md and
from the capability specs, so a step never promises something the app does
not do:

- **Today** — capture a spark in one line; press Write; spend one ~20-minute
  stretch; leave when the indicator says so; the dotted day is the record.
- **ChoosingSpark** — scan newest-first; take what itches now; don't
  deliberate; the chosen spark is consumed, the rest wait; leaving starts
  nothing.
- **Write** — any first sentence; keep going without rereading; write your way
  through a block; leave gaps for later; saving happens by itself; stop at the
  indicator, move to Edit when the draft is whole.
- **Edit** — read it whole first; cut largest-first; rearrange; fill the gaps,
  then work sentences; go back to Write for new material; title last, then
  finish.
- **Finishing** — decide it goes to a reader; copy or export; publish for real;
  paste the link back; confirm; or shelve deliberately, knowing it is one-way.
- **Shelf** — what each column is; the WIP rule and why; the published row as
  the progress display; the drawer is readable but one-way; the only way to
  free the slot.

`Method` lives beside the overlay in the app crate, one per `Place` behind an
exhaustive match, so a new `Place` fails to compile without an instruction
(the same trick that keeps the keymap truthful).

### D4. The key is `cmd-shift-h`, bound everywhere

It pairs with `cmd-h` — the lighter press asks *what can I press*, the
heavier one asks *what am I doing here* — is free in the current keymap, and
carries a modifier so Today's plain-typing rule is untouched. Registered as a
normal `Shortcut` with `Scope::Everywhere`, so the shortcut-help truthfulness
requirement automatically lists it in the "Везде" section with a label.

*Alternative considered:* `cmd-/` (a common help key). Rejected: on Russian
keyboard layouts `/` moves around; `shift-h` is layout-stable.

### D5. One sheet field in root, not a flag apiece

Root's `helping: bool` becomes `sheet: Option<Sheet>` with `Sheet::Help` and
`Sheet::Guidance`. Two booleans would leave "both up at once" representable
and enforced only by remembering to clear the other one; one field makes the
exclusivity structural. The two overlays also share a single `sheet_focus`
handle, since only one is ever on screen — the key *contexts* stay distinct
(`Help`, `Guidance`), which is what the bindings care about. Focus capture and
restoration are the existing help behavior, moved into `toggle`/`close_sheet`.

*Implementation note:* this replaced the planned `guiding: bool` beside
`helping` during apply. Same behavior, one less invariant to maintain by hand.

## Risks / Trade-offs

- [Steps drift from the mechanics as the app evolves] → the instructions are
  keyed by `Place` in one file, written from the capability specs rather than
  from memory; any change touching a place's behavior reviews its steps, and
  the compile-time exhaustive match catches new places.
- [Guidance reads as onboarding creep] → it appears only between two
  keypresses, never on first launch; the first-run-onboarding spec's "only
  onboarding" requirement stays intact and is cited in the new spec.
- [An instruction outgrows the sheet, which cannot scroll] → a test caps the
  steps at seven; longer would mean the place is doing too much.
- [Two similar overlays confuse: which one did I summon?] → visibly different
  bodies (a keys table vs. a numbered instruction) and distinct titles; same
  family styling keeps them from feeling like different apps.

## Open Questions

None — key, contexts, and content sources are fixed above.
