# dogfooding-polish — Design

## Context

The conveyor is complete (stages 1–5): sparks, Write/Edit, publishing, the
Shelf, the rhythm and the showcase on Today. The app is used daily. What is
left is the edges the concept's "first week" walks through: the very first
launch lands on an empty Today with no hint that the spark box wants seeding;
Today's two actions (Write, Shelf) are mouse-only while everything past them
is keyboard-friendly; and the "spark in five seconds" promise has never been
measured against a cold start. On top of that, PLAN.md schedules the review of
the concept's open questions now that there is live experience to answer them
with.

Current state that matters:

- `main.rs` already focuses the spark input before the first frame
  (`main.rs:96-101`) — capture-readiness as a focus rule is done; the missing
  half is the time budget.
- All key bindings are declared in one place (`main.rs:45-67` plus
  `editor::key_bindings()`), with context-scoped bindings (`"Write"`,
  `"Edit"`, `"Shelf"`, `"LineInput"`). Adding Today-scoped bindings follows
  the existing pattern.
- `TodayView::reload` already reads sparks, essays, and sessions on refresh —
  the "is the app empty?" question is answerable from data already in hand,
  so onboarding needs no new storage.
- `theme.rs` centralizes colors and type; typography tuning has one home.

## Goals / Non-Goals

**Goals:**

- The first launch teaches itself: an empty app asks for 3–5 sparks and then
  never speaks of it again.
- The daily loop — capture, write, shelf, back — works without the pointer.
- Launch-to-capture-ready is measured and fits inside the five-second spark
  promise.
- One deliberate typography pass over the three screens and the editor.
- The concept's open questions get answers from dogfooding, recorded in the
  Russian docs; the WIP = 1 invariant and the Write/Edit separation are not
  touched by this change (the review may *recommend*, never implement).

**Non-Goals:**

- No OS-level global hotkey, background process, or separate quick-capture
  window (fallback only if the measured budget fails, as its own change).
- No persisted onboarding flag, no onboarding "flow", no sample content.
- No shortcut cheat-sheet UI, no customization, no settings of any kind.
- No markdown-lite extensions, no font pickers.

## Decisions

### D1. Onboarding is an empty-state, not a state machine

The prompt shows when the data says the app is unused: **no sparks and no
essays in any state**. It stays while the box holds fewer than three sparks
and no essay exists, and disappears once either threshold is crossed. No
"onboarding completed" flag is written anywhere — the condition is computed
from the stores `TodayView` already reads.

*Why not a persisted flag:* a flag is state that can go stale, needs a home in
the data directory, and buys nothing — the only way to see the prompt again
is to have an empty data directory, and an empty app asking for sparks is
correct behavior, not a bug. This also keeps `esse-core` untouched.

*Form:* one or two quiet lines around the spark input on Today (the concept's
words: ideas arrive daily and get lost — write down 3–5 now). It must not
displace the Write button's primacy or steal focus; it is text, not a modal.
While one or two sparks exist the prompt stays (slightly shortened), because
"3–5 sparks" is the point — one spark is not yet a box to pick from.

### D2. Shortcuts extend the existing binding table, modifiers only

New bindings, all in `main.rs` next to the current ones, scoped to the
`"Today"` key context (added to `TodayView`'s root element the way the editor
modes do it):

| Key | Where | Action |
|---|---|---|
| `cmd-enter` | Today | The Write button's action (start or continue) |
| `cmd-l` | Today | Open the Shelf |
| `cmd-l` | Shelf | Back to Today (symmetric with opening) |

`escape` already leaves the Shelf and both editor modes; `cmd-e` already
switches modes. With these three additions the loop closes: launch → type
spark → Enter → `cmd-enter` → write → `escape` → `cmd-l` → browse → `cmd-l`.

*Why modifiers only:* plain keys must keep landing in the spark input — the
capture-first contract of Today. `cmd-enter` is chosen over a bare shortcut
letter for Write for the same reason. *Why `cmd-l`:* free on all screens, and
mnemonic enough (полка → library shelf); the exact letter is cheap to change
during dogfooding since it lives in one line.

### D3. Cold start gets measured before it gets optimized

The spec budget: Today on screen, spark input focused, within **one second**
of process start on the target machine (Anton's Mac). Implementation order:

1. Instrument: log the time from `main()` entry to the first frame ready
   (behind `ESSE_STARTUP_TIMING=1`, printed via the existing `env_logger`
   route so it costs nothing normally).
2. Measure a cold and a warm start a few times.
3. Only if over budget: move data loading off the critical path (open the
   window first, fill lists on first refresh) — `Data::open` reads a handful
   of small files and is unlikely to be the problem; gpui window/GPU init
   will dominate and mostly is not ours to fix.

*Why not optimize speculatively:* the plan's quick-capture-window idea is a
big hammer (background residency contradicts "closing the window quits");
buying it before measuring would be premature. If the budget genuinely cannot
be met, that fallback becomes its own proposed change.

**Addendum — what it measured (31 Aug 2026, release build, Apple silicon).**
The app reports `main()` entry to the first frame drawn; the second column is
what the shell sees from spawning the process, which adds exec and dyld:

| | `main()` → first frame | spawn → first frame |
|---|---|---|
| Cold (first launch of a freshly linked binary) | 127–200 ms | 473–784 ms |
| Warm (repeat launches) | 125–169 ms | 138–188 ms |

Inside the budget with room to spare, and the app's own share of it is about
150 ms whether cold or warm — the cold-start difference is macOS validating a
binary it has not seen before, which is not ours to fix and does not need to
be. **Step 3 was therefore not taken**: `Data::open` is not on the critical
path in any measurable way, and moving data loading off it would buy nothing.

### D4. Typography is one pass, in one place

A single deliberate pass over `theme.rs` and the per-screen sizes: type
scale, line heights, measure (text column widths), spacing rhythm between
Today's zones, Shelf columns, and the editor's two modes — checked against
real essays from the dogfooding data, in both modes. Timeboxed and done by
eye; no spec requirements, because "looks right" is not testable — the specs
already pin the structural rules (prominence, subordination, contrast between
modes).

**Addendum — what the pass changed: nothing.** Walked through Today, the
Shelf and both editor modes against real essays; the scale, the line heights,
the measures and the spacing all held up, and the Write/Edit contrast still
reads at a glance. `theme.rs` is untouched. A pass that finds nothing is the
pass done, not the pass skipped — further nits wait for real irritation.

### D5. The open-questions review is a documentation deliverable

A written revision of CONCEPT.md's «Открытые вопросы» and PLAN.md's
recommendations table: for each question (WIP strictness, Write-mode
backtracking, export forms — plus anything dogfooding surfaced), record the
lived answer and either "keep as is" or a sketch of a future change. In
Russian, since those are the human-facing docs. **This change implements none
of those decisions** — the WIP = 1 invariant and the Write/Edit separation
stay exactly as specified.

## Risks / Trade-offs

- [The one-second budget may be unreachable on gpui cold start] → Measure
  first (D3); if the floor is gpui's own init, record the real number in the
  review (D5) and propose the quick-capture fallback as a separate change
  rather than force-fitting it here.
- [Onboarding prompt reappears if the user empties their data directory] →
  Accepted by design (D1): it is an empty-state, and an empty app asking for
  sparks is the right behavior.
- [`cmd-enter` / `cmd-l` may collide with muscle memory or IME] → All new
  bindings are modifier-based and live on one line each; dogfooding will
  falsify a bad choice within days and the fix is trivial.
- [Typography polish is subjective and can churn forever] → One timeboxed
  pass (D4), committed; further nits wait for real irritation, not
  speculation.
- [Scope creep from the open-questions review into behavior changes] → Hard
  rule in D5: the review writes documents only; any mechanic change gets its
  own OpenSpec change.
