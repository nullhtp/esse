# esse — the concept

**In one line:** this is not an editor, it is a conveyor — from a spark to a
published essay.

The world has enough editors. A beginning essayist does not stop writing for
want of an editor; they stop because they trip over five specific problems.
esse is five mechanics against those five problems, and nothing else.

---

## The user

Someone who wants writing essays to become part of their life. The goal is
public texts: a blog, a newsletter, a channel. They are not a writer, they have
no habit, no confidence and no process. The first version is built "for
myself" (for Anton), with a possible product later.

### The five beginner problems

1. **"Nothing to write about."** Ideas actually arrive every day — in a
   conversation, in the shower, while reading — but they are never captured and
   are forgotten. By the time the person sits down to write, their head is
   empty.
2. **"I cannot start."** The blank page is the strongest barrier. Beginning a
   session costs disproportionately much.
3. **"I write and edit at once, and finish nothing."** A beginner edits the
   first sentence for half an hour. Drafting and editing are mixed into one
   process, and that process never converges.
4. **"I give up after two weeks."** The unit of progress — "a finished essay" —
   is too large. Between sessions there is no sense of movement, and the habit
   never sets.
5. **"I am ashamed to publish, everything dies in drafts."** The text is "not
   ready yet" forever. Without publishing there is no completion cycle — and no
   reason to start the next essay.

---

## The conveyor

```
Spark  →  Draft (free writing)  →  Editing  →  Published
```

Every essay lives in exactly one of these states. The app always knows what
comes next and suggests that step.

**An opinionated simplification: at most three essays in progress (WIP limit
= 3).** A fourth cannot be started until one of the three is published or
deliberately shelved. The limit removes procrastination-by-choice and the
graveyard of ten half-started drafts; three rather than one because an essay
that has gone cold needs somewhere to wait, and burying it was the only thing
a single slot allowed. Sparks are unlimited — the essays in progress are
three.

---

## The five mechanics

### 1. The spark box
Capturing an idea as a single line, instantly: open, write, close — under five
seconds. A spark is not a title and not a commitment, it is a seed: "why all
habit advice fails", "the story of the lost backpack". The box removes
"nothing to write about": by the time the session starts there is always
something to choose from.

### 2. Start from a spark — there is no blank page
A writing session always begins one of two ways: continue the current essay, or
unfold a spark into a new draft. A "create an empty document" screen does not
exist in the app at all.

### 3. Two modes: Writing and Editing
The central mechanic. It physically separates the two processes a beginner
ruinously mixes:

- **Writing** — a fullscreen flow: only the current fragment is visible
  (typewriter mode, cursor centred), going back and fixing things is awkward on
  purpose. The job of this mode is to pour the text out *badly*. Volume, not
  quality.
- **Editing** — the whole text is visible, the layout is calm, the job is to
  cut, rearrange and sharpen. Quality, not volume.

Switching is an explicit action (and the two modes look visibly different).
That is how the app teaches the essayist's core skill: **first write it badly,
then make it good.**

### 4. Sessions, not deadlines
The unit of the habit is a 15–25 minute session, not "a finished essay". Every
session is a completed success, even when the essay is not done. The rhythm to
aim at: ~3 sessions a week ≈ an essay every one or two weeks. Session history
is an unobtrusive dotted calendar, with no oppressive streaks and no guilt for
a missed day.

### 5. Publishing is completion
An essay ends in exactly two ways: **published** (exported to markdown or
copied into a blog, plus a "publication link" field) or deliberately
**shelved**. The main display of progress is the growing row of published
essays. Not a word count, not a streak — the shelf of what reached people.

---

## The screens

Three in total.

### 1. Today (the main one)
- One big **Write** button: it continues the current essay or, if there is
  none, offers a spark to start from.
- A quick spark input line.
- At the bottom — the row of published essays (cover cards) and a thin dotted
  line of the last weeks' sessions.

### 2. The editor
- Fullscreen, no panels. Markdown-lite: `#`, `*italic*`, `**bold**` — that is
  all.
- The feeling to aim at is **Typora**: markdown renders in place as you type,
  with no split preview and no raw markup left in finished text.
- The **Writing / Editing** switch (see mechanic 3).
- The session timer — quiet, in the corner; when the time is up it gently
  suggests stopping.

### 3. The Shelf
Three columns: **Sparks | In progress | Published**. An essay can be started
from a spark (while fewer than three are in progress); "In progress" holds up
to three, most recently worked first; published ones show their dates and
links. Separately, a collapsed drawer for what was deliberately set aside.

### The first week with the app
- **Day 1.** Open it — the app asks for 3–5 sparks (the only onboarding). Pick
  one, a 15-minute session in Writing mode. The result is a raw piece of text.
  That is a success.
- **Days 2–4.** A couple of sparks tossed in along the way. Two more Writing
  sessions — the draft is finished end to end: bad, but whole.
- **Days 5–6.** A session in Editing mode: cut, rearranged. A second pass —
  cleaned up.
- **Day 7.** Press Publish, paste the text into the blog, save the link. The
  first essay is on the shelf. All three lanes are free again, and five sparks
  are waiting in the box.

---

## Technical direction

- **The editor benchmark is Typora:** live markdown rendering right inside the
  text as you type — no split preview, no source mode, minimalism, nothing but
  the text. esse takes that feeling and adds the conveyor and the
  Writing/Editing modes on top.
- **The chosen stack is a native Rust GUI, desktop.** Candidate frameworks:
  - **gpui** (the Zed engine) — built for exactly this: a text editor, GPU
    rendering, the best text handling in Rust; the downside is that it is
    young, thinly documented and its API is unstable.
  - **iced** — more mature, Elm architecture, fits a three-screen app well;
    advanced text handling would have to be written as a custom widget.
  - **egui** — the fastest start (immediate mode), but weaker typography — fine
    for a prototype, unlikely to carry the final "Typora feeling".
  - Live markdown rendering is written by hand either way (parser:
    **pulldown-cmark** or **comrak**). This is the most expensive part of the
    project — so it is where to start.

---

## Anti-features

Deliberately NOT built — and why:

| What is missing | Why |
|---|---|
| AI (generation, suggestions, rewriting) | The habit forms only through your own writing; AI text devalues the practice |
| Formatting toolbars, styles, fonts | Fiddling with appearance is the favourite procrastination; markdown-lite is enough |
| Folders, tags, nesting | With three essays in progress and a flat spark box there is nothing to organise |
| Social features, comments, in-app sharing | Publishing happens outside, in a real blog; an in-app social network is a trap |
| Word/day statistics, graphs | The only metrics are sessions and what got published; the rest is vanity |
| Settings and customization at launch | Every setting is a way not to write |

---

## The path: tool → product

The "for myself" version tests the main thing: does the conveyor work, and does
the habit take. If it does and building a product for others becomes
attractive, this gets added:

1. **Onboarding** — the "first week" above, shaped into a built-in path.
2. **Accounts and sync** across devices (local data is enough for the personal
   version).
3. **Publishing integrations** — one click into Ghost, Telegraph or Substack
   instead of copy-paste.
4. **A mobile companion for sparks** — real writing stays on the big screen,
   the phone is only for the box.

What will NOT be added even in a product: AI and an in-app social network —
those are positioning, not a temporary limitation.

---

## Open questions

Revisited at stage 6, when the conveyor was already working and in daily use.
The questions are closed; where the decision stayed the same, what would change
it is written down too. The revision changes no behaviour — any change of
mechanics goes through its own OpenSpec change.

- **GUI framework — closed.** **gpui** was chosen: the stage 0 prototype met
  every criterion on the first attempt, so the iced prototype was never built.
  The reasoning is in the Decision Record
  `openspec/changes/editor-framework-prototype/design.md`.
- **Export — closed, both are needed.** The completion panel has both "Copy as
  markdown" and "Save to a file…". Both turned out cheap, so there was nothing
  to choose between.
- **How hard the WIP limit is — closed, and it stays hard at three.** The ban
  did get in the way: a stalled essay left only two legal moves, publishing
  something unfinished or burying it, so the real third move was not writing.
  The sketch written here — a second slot bought by shelving the current essay
  — was rejected when it came to it, because it prices every switch in the one
  currency a writer must not spend. Three plain lanes put the friction at the
  fourth essay instead, where the graveyard actually starts. The reasoning is
  in `openspec/changes/archive/*-write-several-essays/design.md` (D1). What
  would move it again: if three open essays turn out to mean three unfinished
  ones, the limit comes back down — it is one constant.
- **Editing backwards in Writing mode — stays merely awkward.** There will be
  no mechanical ban on the cursor: dimming the text above the current line does
  the job, and a hard ban would break the small things people go back for — a
  typo in the previous word, a sentence left hanging.
- **Launch speed — measured, question closed.** Time to being ready to type:
  ~0.5–0.8 s cold and ~0.15 s warm (measured with `ESSE_STARTUP_TIMING=1`, a
  release build). About 0.15 s of that is the app itself; macOS takes the rest.
  The promise of "a spark in under five seconds" holds even including opening
  the app, so a separate fast companion window for sparks is not needed.
