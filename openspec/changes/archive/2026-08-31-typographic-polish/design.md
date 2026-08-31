# typographic-polish — Design

## Context

esse draws almost nothing but text. There are four screens, two of them a
single column of prose, and the widest piece of chrome in the app is a 56px
button. So the appearance is very nearly just the typography, and until now the
typography was gpui's default: `.SystemUIFont`, at sizes each screen picked for
itself.

What the code looked like going in:

- `theme.rs` held colours and four sizes; the individual paddings (`px(9.)`,
  `px(14.)`, `px(18.)`, `px(26.)`) lived at their call sites, thirty-odd of
  them, with no relation between them.
- The three palettes existed but leaked: `edit.rs`'s completion button was
  `theme::INK` on `theme::BACKGROUND` — Today's paper, in a panel sitting on
  Edit mode's daylight — and the panel's muted text was Today's `MUTED`.
- `CARET` was `0x2563eb`, a UI blue, in an app with no other saturated colour.
- Corner controls, column headings, notices and body copy were all
  `SMALL_SIZE` or `BODY_SIZE` in `MUTED`: four different jobs, one register.

Nothing in the existing specs constrains appearance, so none of this was a
violation — which is exactly why it drifted, and why the rules land in a spec
at the end of this change.

## Goals / Non-Goals

**Goals:**

- The app is set in a face chosen for reading essays, and ships with it.
- Hierarchy comes from size, weight and space; colour stops carrying it.
- The three rooms are told apart at a glance and cannot converge by accident.
- Every gap and every size in the app comes from one small scale.
- The whole thing is still recognisably esse: the same screens, the same
  restraint, no ornament added anywhere.

**Non-Goals:**

- No behaviour change of any kind. The WIP = 1 invariant, the pipeline, the
  keyboard map and the routing are untouched, and the Write/Edit separation is
  strengthened rather than altered — the two rooms are further apart after this
  change than before it, which is what the mode separation asks for.
- No settings, no themes, no system-appearance following (anti-features).
- No new runtime dependency.

## Decisions

### D1. One embedded family, not the system face

The app names one family, `Literata`, and embeds it. Three reasons in order of
weight:

1. **Cyrillic.** Essays here are written in Russian as often as in English —
   `esse-core`'s slug transliteration exists for exactly that. The first face
   drafted for this design, Newsreader, has no Cyrillic at all: every Russian
   paragraph would have fallen back to a system sans mid-word. This ruled it
   out and very nearly ruled out the whole approach; it is the single most
   important thing to check when picking a face for this app, and
   `fonts.rs` now has a test that fails if a future face forgets.
2. **A reading face for a writing tool.** Literata was drawn for long-form
   reading on screen. The editor is the app; the face it renders in is not a
   decoration.
3. **Embedded, not installed.** Nothing to install, and esse looks the same on
   any Mac it is copied to.

The cost is ~1.3 MB of binary and a licence file. Startup does not measurably
change: registration is memory-mapped, and first frame is still 201 ms in a
debug build, against a budget of one second.

### D2. Static instances, because gpui cannot use a variable font

This is the one genuinely fiddly decision, and it is invisible from the screen.

gpui hands font bytes to font-kit's in-memory source, which groups faces by the
family name Core Text reports and then picks one by weight and slant
(`MemSource::select_best_match`). It does not instance variable axes. Handing it
`Literata[opsz,wght].ttf` would therefore register **one** face — the default
400 instance — and every request for 500 or 600 would land on it, leaving the
rasterizer to fake the difference.

So `scripts/fonts.py` cuts five static instances (400/500/600 upright, 400/600
italic, all at optical size 16) and rewrites each name table so that the
typographic-family names are gone and all five report family `Literata`, with
the weight carried in `OS/2.usWeightClass`. That is what makes one
`.font_family("Literata")` at the root plus a weight per element work.

This is exactly the kind of arrangement that breaks silently — a re-cut that
leaves nameID 16 in place would collapse every weight onto one face, and only
the screen would say so. `fonts.rs` therefore tests it directly, against the
same matcher gpui uses, which is why `font-kit` is a dev-dependency (at gpui's
own pin, so no second copy is built).

### D3. Hierarchy out of size, weight and space — and one accent

Six sizes (32/26/22 headings, 20 editor, 19 input, 17 body, 13 small, 11 label)
and five gaps (6/10/16/24/36). Every value in the app is one of these.

Weight does the rest: body at 400, headings and labels at 500, `**bold**` at
600. A serif at 600 shouts next to a 400 body, so a heading takes 500 — and
bold sits a step above it, which means a bold word inside a heading is still
visible as one. The editor previously used 700 for both, so it could not be.

The one saturated colour left in the app is the caret. Sienna on paper, amber at
night. Everything else is ink, paper, or the distance between them.

### D4. A palette per room, complete, with nothing shared

`theme::write` and `theme::edit` now carry their own muted ink, their own rules,
and — the leak this fixes — Edit mode's own button ink. The rule is written into
the spec as much as the code: **no room borrows a colour from another.** Sharing
a value is not a small economy here; it is how two rooms that must be told apart
at a glance start converging, one reasonable reuse at a time.

Write mode goes warm (`0x191613`) rather than blue-grey. It is the same paper
and the same ink as Today with the light off, which makes the two rooms read as
one place at different hours instead of two apps. The dimmed-paragraph ink is
2.45:1 against it — deliberately low, as before (2.32:1), because dimming what
is already written is the mechanic.

### D5. Labels are letterspaced capitals, spaced with thin spaces

The corner controls and column headings name a place. They now sit in their own
register — 11px capitals at weight 500, letterspaced — so they stop reading as
sentences.

gpui's `TextStyle` has no letter-spacing, so `theme::label` interleaves U+2009
THIN SPACE between characters. That is a deliberate choice rather than a
workaround being tolerated: a thin space is *whitespace*, so a face that lacks
the glyph costs an advance and draws nothing. There is no way for it to fail
visibly, which is what makes it safe to do at all. Literata has it, and
`fonts.rs` asserts so.

## Risks

- **The look itself is a judgement, and it is not verifiable from here.** The
  tests prove the faces load and the weights resolve; they cannot prove the page
  looks right. Optical size 16 across a range from 11px labels to 32px headings
  is the most likely thing to want a second pass, followed by the dimmed ink in
  Write mode. Both are one constant.
- **Tracked capitals in Cyrillic** are less conventional than in Latin. `label`
  handles both alphabets and is tested on both, but whether «И С К Р Ы» reads
  well is a question for the screen.
- **Binary and repo size** grow by ~1.3 MB, committed as binary blobs.
  Accepted: it buys a self-contained app.
