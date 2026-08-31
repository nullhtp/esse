# guided-install — Design

## Context

After `brew install nullhtp/tap/esse` the app is in /Applications and nothing
else has happened. The three things the writer still needs are spread across
three places: the summon key is in the cask's caveats, the launch agent is a
script buried in `Contents/Resources/`, and the folder is a fact you learn by
opening the app. This change gathers them into one conversation.

Two constraints shape everything below. A Homebrew cask cannot ask questions —
`brew install` is non-interactive by design, so the questions have to be asked
by something the install points at. And esse has no settings: the anti-features
list rules out settings and customization at launch, and `first-run-onboarding`
states that the sparks prompt is the entirety of first run. So the answers must
be collected outside the app and read by it, never collected inside it.

The pieces already in place: `scripts/login-item.sh` writes and removes the
launch agent and travels inside the bundle (brew-install design.md, D5);
`summon.rs` reads `ESSE_HOTKEY` from the environment; `DataDir::open` resolves
`ESSE_DATA_DIR`, else `~/Documents/Esse`, and moves an older installation out
of the hidden platform directory exactly once (visible-data-folder design.md,
D1–D4).

## Goals / Non-Goals

**Goals:**

- One command, run once, leaves esse ready: folder chosen, key known, waiting
  at login if the writer wants it.
- The answers hold on every launch, not only the ones the launch agent starts.
- Running setup a second time is safe, shows what is currently true, and
  changes only what the writer changes.
- Nothing about the app's own behaviour changes for someone who never runs it.

**Non-Goals:**

- Any settings surface inside the app, in any of the three screens.
- Any question asked at launch, ever.
- Reading two data folders, merging two data folders, or syncing.
- Notarisation, auto-update, an installer package.

**Invariants untouched:** the WIP limit of 1 and the Write/Edit separation are
not in this change's path at all — it decides where files live and how the app
is reached, never how many essays are in progress or which mode is on screen.

## Decisions

**D1. The setup is a terminal conversation carried by the bundle.** A cask
cannot prompt, so the only place left is a command the caveats point at. The
alternative — a one-time card on first launch — was rejected on the specs: it
would contradict `first-run-onboarding` ("the app MUST NOT show tours, tooltip
sequences, sample content, or setup steps on first launch") and the
anti-feature against settings at launch, and it would put a wizard in front of
an app whose whole claim is that there is nothing between you and the first
sentence. A terminal conversation is also the honest shape: it happens once, at
the moment the writer is already in a terminal, and leaves no surface behind.

**D2. Two scripts: `esse-setup` asks, `login-item` acts.** The launch agent
already has a working, documented, non-interactive contract (`login-item`,
`login-item --off`, `HOTKEY=…`), and the README, the current cask's caveats and
`make login-item` all depend on it. Setup calls it rather than reproducing its
`launchctl` dance, and it gains `--status` so setup can offer the current state
as the default answer. One script doing both jobs would have to decide whether
a bare invocation prompts or acts, and would break every copy of the old
one-liner.

**D3. The answers live in one line-oriented file at
`~/Library/Preferences/com.nullhtp.esse.conf`.** Three `key=value` lines at
most — `data_dir` and `hotkey`, which the app reads, and `at_login`, which is
setup's own memory of a "no" (D7) and which the app never looks at. Written
only by setup. The
obvious home — `~/Library/Application Support/esse/` — is exactly the directory
`DataDir::open` treats as an older installation to be moved: a file there would
make `old.is_dir()` true forever and the first launch on a fresh machine would
carry the config directory into `~/Documents/Esse`. A plist through
`CFPreferences` was the other candidate and was rejected because a shell script
would need `defaults write` and the app would need a plist parser, where a
`key=value` file is written with `printf` and read in a dozen lines of Rust —
the same plain-files reasoning the data folder is built on.

**D4. The file records departures, not answers.** Accepting a default writes
nothing for that question, and the file is removed when nothing departs. This
keeps the one-time move out of the hidden directory
working for an installation older than this change — a recorded
`~/Documents/Esse` would suppress it, since a recorded location is deliberate
in exactly the way `ESSE_DATA_DIR` is (visible-data-folder design.md, D4) — and
it means a machine that took every default is indistinguishable from one that
never ran setup.

**D5. Resolution order: environment, then the file, then the default.** For
both answers: `ESSE_DATA_DIR` and `ESSE_HOTKEY` keep winning, because they are
the development and test override and because the launch agent uses them.
Setup writes the file and rewrites the agent in one act, so the two cannot
drift apart; someone driving `login-item` directly with `HOTKEY=` still gets
what they asked for on login launches, which is the behaviour that is already
documented.

**D6. Setup does the move; the app's one-time migration stays as it is.**
Setup looks for existing writing in the folder the app would open today — the
recorded location, else `~/Documents/Esse`, else the old hidden directory —
counts what is in it, and offers to move it whole with `mv`, which already
falls back to copy-then-unlink across volumes. It refuses to merge: if the
named folder exists and is not empty, it says so and moves nothing, the same
rule the app follows (visible-data-folder design.md, D3). The Rust migration is
left untouched so that someone who never runs setup is unaffected.

**D7. Autostart defaults to yes, and a "no" is remembered.** A key that only
works while the app happens to be running is not a key you learn to reach for,
so the prompt is `[Y/n]` on a machine that has never been set up. Afterwards
the default is what is true: `login-item --status` when an agent is installed,
and otherwise the `at_login=no` line setup wrote when the writer declined.
Without that line the two rules collide — a writer who deliberately turned
autostart off would have it turned back on by pressing `enter` through a second
run, which is precisely what "safe to run again" must not mean. The line is a
departure from the default like any other, so a machine that took every default
still has no file at all.

**D8. Without a terminal, setup changes nothing.** If stdin is not a TTY the
script prints the three current answers and exits 0 without touching the file,
the agent or the writing. Taking defaults silently in a pipeline would mean a
script could install a launch agent nobody asked for, and adding flags for
scripted use would grow a second interface for a command that is run once by
hand.

**D9. A recorded folder that has gone missing stops the launch.** A path like
`/Volumes/Backup/Essays` on an unmounted volume would otherwise be recreated
locally by `create_dir_all`, and the writer would open esse onto an empty
lookalike while their essays sat on a disk that was merely unplugged. When the
recorded location's parent directory does not exist, the launch stops with a
message naming the path — the same shape as the existing failed-move message.
A missing leaf directory is still created, since that is the ordinary case of a
folder named at setup and not yet used.

**D10. The app gets no new UI.** The Shelf already says the data folder at its
foot and `cmd-o` opens it, so a named folder shows up in the one place the app
already talks about where the writing is.

**D11. The binary checks the spelling of a combination.** Setup has to reject
`cmd-shift-spacebar` before it records it, and the vocabulary of key names
lives in `summon.rs`. Rather than a regex in the shell script that would drift
away from the parser, the binary answers one question — `esse --check-hotkey
cmd-shift-space`, exit 0 or 1, no window, no data directory opened. It is not a
command-line interface for esse; it is the parser, asked out loud.

## Risks / Trade-offs

- **The launch agent's environment and the recorded file can disagree** — run
  `HOTKEY=x login-item` by hand after setup recorded `y`, and login launches
  answer `x` while Finder launches answer `y` → setup writes both in one act
  and is the documented way to change the key; the README points at setup and
  keeps `login-item` as the mechanism it drives.
- **A file the app reads is a settings file in all but name** → it has no
  writer inside the app, no UI, no defaults dialog and two keys, both of which
  the `summon-from-anywhere` spec already allows to be named at install time.
  If it ever grows a third key that is about writing rather than installation,
  that is the signal it has become the thing this project does not build.
- **Someone upgrades and never runs setup** → nothing changes for them: no
  file means the current behaviour exactly, and the caveats say what the one
  line would do.
- **Setup moves the writing and something fails halfway** → `mv` of a whole
  directory is atomic within a volume; across volumes it copies before it
  unlinks, so a failure leaves the source standing, and setup records the new
  location only after the move reports success.
- **The old cask's caveats point at `login-item`** → that command keeps working
  unchanged, so an upgrade never invalidates instructions someone wrote down.

## Migration Plan

1. Build the bundle with both scripts; `make setup` runs the new one against
   the installed copy.
2. Verify on this machine: fresh answers, then a second run taking every
   default (nothing changes), then a run that names a new folder with writing
   in the old one (it moves), then a run that turns autostart off.
3. Release: the new cask caveats point at `esse-setup`. An existing install
   picks it up on `brew upgrade`, keeps its launch agent, and keeps writing to
   `~/Documents/Esse` until someone says otherwise.
4. Rollback is `brew uninstall --cask esse` plus removing
   `~/Library/Preferences/com.nullhtp.esse.conf`; the writing is never in the
   blast radius.

## Open Questions

None.
