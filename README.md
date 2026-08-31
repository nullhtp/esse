# esse

This is not an editor, it is a conveyor — from a spark to a published essay.

A desktop app for someone who wants writing essays to become part of their
life. Five mechanics against five beginner problems, and nothing else.

```
Spark  →  Draft (free writing)  →  Editing  →  Published
```

A hard constraint: **only one essay in progress**. Sparks are unlimited.

The full concept, including the anti-features and the open questions, is in
[CONCEPT.md](CONCEPT.md).

## Status

The conveyor is closed and in daily use: a spark becomes a draft, the draft is
written fullscreen, editing happens in its own room, and an essay ends by being
published or deliberately shelved. Today carries the session dots and the row
of published essays; the Shelf shows the whole conveyor at once; the whole app
is driven from the keyboard, and `cmd-h` and `cmd-shift-h` answer for the keys
and for the method.

The stack is a native Rust GUI: **gpui**, the engine behind Zed, chosen at
stage 0 against a live-markdown prototype. The `markdown-lite` parser moved out
of the prototype into [crates/markdown-lite](crates/markdown-lite); the editor
was rewritten with line wrapping, which the prototype never had. The prototype
code stays in [prototypes/](prototypes/), frozen.

## How it works

**Spark.** A line on the Today screen, Enter — the thought is kept. There can
be any number of sparks; they sit in a list, newest first.

**Starting.** The Write button is the only way into the editor; an empty
document does not exist in this app.

- if an essay is already in progress, the button opens it;
- if not, the list of sparks becomes a choice: click the one to start from;
- if there are no sparks, the app says so: a spark comes first.

The chosen spark is **spent**: its text goes into the new draft's front matter,
the file name is derived from it (Cyrillic is transliterated: "почему эссе" →
`essays/pochemu-esse.md`), and the spark leaves the list.

**Writing.** Fullscreen, dark, nothing but the text: markup renders in place,
raw characters show only on the cursor's line, that line stays centred, and
everything above it is dimmed. There is no scrolling — the screen follows the
cursor. The text saves itself: a second after you stop typing, and always on
leaving, on closing the window and on quitting.

**Session.** It starts together with Writing mode; the minutes tick quietly in
the corner. At the twentieth minute the counter softly turns into "session
done" — no modal, no sound, and you can keep writing as long as you like.
`Esc` leaves: the essay is saved, the session is recorded (anything under a
minute is not), and the app returns to Today.

**Editing.** A different room, and it shows from the doorway: a light
background, every line at full strength, nothing dimmed, scrolling free — the
screen no longer hangs on the cursor. The text saves itself the same way. No
session runs here: editing is not writing, and its minutes are not counted.

**Switching.** In the corner of each mode there is a quiet label with the name
of the neighbouring room ("Editing" while writing, "Writing" while editing);
it is also `cmd-e`, the same gesture both ways. Switching saves the essay and
moves it between the draft and editing states — so the mode on screen and the
state on disk cannot drift apart. The Write button on Today opens the essay in
whichever mode it was left in.

## The keyboard

The whole app is driven from the keyboard: capture a spark, start an essay from
it, walk the Shelf, mark up the text, finish and publish — the mouse is needed
nowhere.

None of it has to be memorised: **`cmd-h`** shows a sheet with exactly the
combinations that work where you are standing. Any next key dismisses it and
does nothing else, so you can glance at it mid-sentence. **`cmd-shift-h`**
answers the heavier question — not what can be pressed here, but how to work
here: the method for this place, step by step.

**Today.** `cmd-enter` — Write, `cmd-l` — the Shelf. Both take a modifier, so
ordinary typing always lands in the spark line.

**Choosing a spark.** When Write asks which spark to start from, the spark line
gives its keys to the list: `↑`/`↓` walk it (the freshest is highlighted),
`enter` starts from it, `esc` changes your mind.

**The Shelf.** `←`/`→` between columns, `↑`/`↓` down a column, `enter` opens
what is selected (a spark starts an essay only while the slot is free), `cmd-d`
the shelved drawer, `esc` or `cmd-l` back to Today.

**The editor.** Text keys are the ordinary ones: arrows and `shift`+arrows,
`alt`+arrows by words, `home`/`end` by the visible line, `cmd-c`/`cmd-x`/
`cmd-v`, `cmd-z` / `cmd-shift-z`, `cmd-a`. Markup is on the keyboard too, the
same in both modes: `cmd-b` bold, `cmd-i` italic, `cmd-1`/`cmd-2`/`cmd-3`
headings. Every press edits the markup characters themselves — what you would
have typed by hand — and one `cmd-z` takes it back. `cmd-e` switches modes,
`esc` leaves.

**Finishing an essay.** In Editing, `cmd-enter` opens the completion panel.
Inside it `tab` (and `←`/`→`) walks the actions, `enter` chooses, `esc` closes.
Until you take a step nothing is highlighted: a stray `enter` cannot publish an
essay.

## Building and running

Rust 1.97.1 is required — the toolchain is pinned in `rust-toolchain.toml` and
rustup installs it by itself. On macOS gpui compiles shaders and needs the
Metal Toolchain component, which Xcode 26 no longer installs by default
(~700 MB):

```
xcodebuild -downloadComponent MetalToolchain
```

Then:

```
cargo run -p esse-app      # run the app
cargo test                 # everything
cargo test -p esse-core    # the core only: seconds, no gpui build
```

The first build pulls ~700 gpui dependencies and takes minutes; later ones take
seconds.

## Data

Everything is plain files in `~/Library/Application Support/esse`:

```
sparks.jsonl        sparks, one JSON object per line
sessions.jsonl      writing sessions: essay, start, minutes
essays/<slug>.md    an essay: TOML front matter between +++ and the text
```

No database and no sync: the files are read and edited by hand and survive any
refactoring. `ESSE_DATA_DIR` overrides the directory — that is for development
and tests, not a user setting.

## Development

The project is run through [OpenSpec](https://github.com/Fission-AI/OpenSpec):
the spec first, then the code.

```
openspec list          # changes in progress
openspec list --specs  # the specs in force
openspec view          # the interactive dashboard
```

Commands in Claude Code: `/opsx:explore`, `/opsx:propose`, `/opsx:apply`,
`/opsx:archive`.

The structure:

```
CONCEPT.md            the product concept
PLAN.md               the implementation plan, stage by stage
crates/esse-core/     the data model and storage — no GUI, tests in a second
crates/markdown-lite/ the markup parser: headings, **bold**, *italic*
crates/esse-app/      the gpui app: Today, the Shelf and the editor's two modes
prototypes/           the frozen stage 0 editor prototype
openspec/config.yaml  the project context for AI assistants
openspec/specs/       the specs in force
openspec/changes/     proposals in progress
```
