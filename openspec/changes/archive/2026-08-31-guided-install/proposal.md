# guided-install — Proposal

## Why

Installing esse today is one line and then three things nobody told you to do:
find the caveats again to learn the summon key, run a `login-item` command out
of `/Applications/Esse.app/Contents/Resources/` by hand, and discover where the
writing went by opening the app and looking at the foot of the Shelf. Each of
them is a small thing the writer has to carry, and together they are the
difference between "esse is installed" and "esse is ready".

This serves beginner problem #2 ("cannot start") one step before the summon key
does: the install should end with esse already waiting for its key, the writer
already knowing which key it is, and the folder already being the one they
wanted. It serves #4 ("gives up after two weeks") in the same motion, since an
app that only answers its key while it happens to be running is an app you stop
reaching for.

## What Changes

- **One command finishes the install.** The bundle carries `esse-setup`, and
  the cask's caveats print a single line to run. It asks three questions, in
  the order a person cares about them: where the essays live, whether esse
  should be waiting at login, and which key brings it forward.
- **The folder is chosen, not discovered.** Setup offers `~/Documents/Esse`
  and takes any other path. If writing already exists at the old location, it
  says how much and offers to move it whole.
- **The summon key is told, not documented.** Setup prints the combination in
  the middle of the conversation and takes another one if `ctrl-alt-e` is
  taken on this machine.
- **The answers survive an ordinary launch.** Today the hotkey only reaches
  the app through the launch agent's environment, so a copy opened from Finder
  is always on the default key and always in the default folder. Setup records
  its two answers in one small file that the app reads at startup, so a login
  launch and a Finder launch agree.
- **Setup can be run again.** It shows the current answers as the defaults and
  changes what changed — including turning autostart back off.
- **The caveats shrink.** Three paragraphs of instructions become one line to
  run, plus the honest sentence about the ad-hoc signature that has to stay.

## Capabilities

### New Capabilities

- `guided-setup`: the one command that finishes an install — the three
  questions it asks, how it records the answers, what it does when run a
  second time, and the promise that the app itself never writes that file and
  never asks these questions in a window.

### Modified Capabilities

- `summon-from-anywhere`: the installed app carries a setup command rather than
  a bare launch-agent script, and a combination named at setup time is honoured
  on every launch — not only the ones the launch agent starts.
- `local-storage`: the data directory may be named at setup time and is then
  recorded; the resolution order becomes `ESSE_DATA_DIR`, then the recorded
  location, then `~/Documents/Esse`. A recorded location suppresses the
  one-time move out of the hidden directory, exactly as a named `ESSE_DATA_DIR`
  already does.

## Non-goals

- **No settings screen, no preferences window, no in-app folder picker.** The
  anti-features list rules out settings and customization at launch, and this
  change does not smuggle them in through a side door: setup is a terminal
  conversation that happens once, the app only ever reads what it wrote, and
  nothing in any of the three screens can change these answers.
- **No setup step inside the app.** `first-run-onboarding` says the 3–5 sparks
  prompt is the entirety of first-run onboarding, and it stays that way — no
  wizard, no modal, no "before you write" card.
- **No GUI installer, no DMG, no `.pkg`.** The install is still `brew install`.
- **No auto-update, no notarisation.** Unchanged from `brew-install`.
- **No prompting on launch.** If setup was never run, the app behaves exactly
  as it does today: `~/Documents/Esse`, `ctrl-alt-e`, no login agent, and no
  nagging about any of it.
- **No second data folder.** Naming a new folder either moves the writing or
  leaves it where it is and says so. esse never reads two folders at once.

## Impact

- `scripts/setup.sh` (new) — the three questions, the recorded answers, and
  the move; copied into the bundle as `Contents/Resources/esse-setup`.
- `scripts/login-item.sh` — keeps its contract (`on`, `--off`) and gains
  `--status`, so setup can offer the right default; setup drives it rather
  than duplicating it.
- `scripts/bundle.sh` — carries the setup script into the bundle.
- `crates/esse-core` — a new module reading the recorded answers, and
  `DataDir::open` consulting it.
- `crates/esse-app/src/summon.rs` — the recorded combination when the
  environment does not name one.
- `resources/cask.rb.in` — caveats reduced to the one line; `zap` takes the
  recorded answers along with the launch agent, and still never the writing.
- `Makefile` — `make setup` for a copy built from source.
- `README.md` — the Installing section becomes install, then setup.
