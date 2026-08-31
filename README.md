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
and for the method. It installs with Homebrew as a background app and comes
forward on one system-wide key.

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
the shelved drawer, `cmd-o` the data folder in Finder, `esc` or `cmd-l` back to
Today.

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

## Installing

```
brew install nullhtp/tap/esse
```

Apple silicon, macOS 13 or later. Then open Esse.app once — and after that you
never have to find it again:

esse is a background app: no Dock icon, no menu bar, nothing in the way. It is
reached one way — **`ctrl-alt-e`**, from inside whatever you were doing. Press
it and esse is in front, focused, ready to type; press it again and it is gone,
with the text on disk. Closing the window does the same as pressing it away —
esse keeps running and the key keeps working. `cmd-q` quits for real.

To have esse waiting for that key from the first minute of the day:

```
/Applications/Esse.app/Contents/Resources/login-item        # on
/Applications/Esse.app/Contents/Resources/login-item --off  # off
```

The key is held system-wide, so it can collide with something else on the
machine. Any other combination, spelled the way the app spells keys everywhere
else:

```
HOTKEY=cmd-shift-space /Applications/Esse.app/Contents/Resources/login-item
```

To try one without installing anything: `ESSE_HOTKEY=cmd-shift-space cargo run
-p esse-app`. If the combination is already taken, esse says so in the log and
runs on without it.

`brew uninstall --cask esse` removes the app; the login item goes with
`login-item --off` first. Your writing is in `~/Documents/Esse` and none of
this ever touches it.

The released build is signed by its author, not notarised by Apple — a yearly
developer subscription is not a trade this app makes — so the cask clears the
quarantine flag macOS would otherwise refuse the app over, after checking the
download against the checksum in the recipe. If you would rather not take that
on faith, build your own copy:

```
make install       # builds Esse.app and puts it in /Applications
make login-item    # and starts it at login, hidden, holding its key
```

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
make run                   # or: cargo run -p esse-app
make test                  # or: cargo test
cargo test -p esse-core    # the core only: seconds, no gpui build
make app                   # assemble target/Esse.app without installing it
make release               # the zip a release is made of, and the cask for it
make icon                  # redraw resources/esse.icns from scripts/icon.py
```

The first build pulls ~700 gpui dependencies and takes minutes; later ones take
seconds.

## Data

Everything is plain files in `~/Documents/Esse` — a folder you can open, read
and back up like any other:

```
sparks.jsonl        sparks, one JSON object per line
sessions.jsonl      writing sessions: essay, start, minutes
essays/<slug>.md    an essay: TOML front matter between +++ and the text
```

The Shelf says the path at its foot and `cmd-o` opens the folder, so the app
never has to be asked where the writing went.

No database and no sync: the files are read and edited by hand and survive any
refactoring. An installation from before the folder was made visible moves
itself out of `~/Library/Application Support/esse` on the first launch, once
and whole; if both folders exist, the visible one is used and the old one is
left alone. `ESSE_DATA_DIR` overrides the directory — that is for development
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
Makefile              run, test, build the app, install it, take it away again
crates/esse-core/     the data model and storage — no GUI, tests in a second
crates/markdown-lite/ the markup parser: headings, **bold**, *italic*
crates/esse-app/      the gpui app: Today, the Shelf and the editor's two modes
resources/            Info.plist, the icon, the Homebrew cask it is rendered from
scripts/              the bundle, the release, the login item, the icon
prototypes/           the frozen stage 0 editor prototype
openspec/config.yaml  the project context for AI assistants
openspec/specs/       the specs in force
openspec/changes/     proposals in progress
```
