# guided-install — Tasks

## 1. The answers the app reads

- [x] 1.1 `esse-core`: a `setup` module reading
      `~/Library/Preferences/com.nullhtp.esse.conf` — `key=value` lines,
      `data_dir` and `hotkey`, a leading `~` expanded, blank lines and
      unknown keys ignored, a missing file meaning "every default" (D3).
      Tests over a path handed in, so nothing touches the real home.
- [x] 1.2 `DataDir::open` consults it: `ESSE_DATA_DIR`, then the recorded
      location, then `~/Documents/Esse` (D5). A recorded location suppresses
      the one-time move out of the hidden directory, exactly as
      `ESSE_DATA_DIR` does (D4). Tests through the existing `open_in` seam.
- [x] 1.3 A recorded location whose parent directory does not exist stops the
      launch with a message naming the path, instead of creating a lookalike
      folder (D9). One error variant, one test.
- [x] 1.4 `summon.rs`: the recorded combination when `ESSE_HOTKEY` names none,
      the built-in `ctrl-alt-e` when neither does.
- [x] 1.5 `esse --check-hotkey <spelling>` — parses and exits 0 or 1, opening
      no window and no data directory (D11).
- [x] 1.6 `cargo test` green, `cargo clippy` clean.

## 2. The setup conversation

- [x] 2.1 `scripts/login-item.sh --status`: prints whether esse starts at
      login and which combination the agent holds, exit code saying the same;
      the `on` and `--off` contracts unchanged (D2).
- [x] 2.2 `scripts/setup.sh` skeleton: read what is currently true (the
      recorded answers, the agent's status), and with no TTY print those three
      facts and exit 0 without touching anything (D8).
- [x] 2.3 The folder question: the current folder as the default, `~`
      expanded, the folder created, a failure reported and the question asked
      again.
- [x] 2.4 The move: count the essays and sparks in the current folder, offer
      to move it whole with `mv`, refuse to merge into a folder that already
      holds esse's files, and say plainly what was left behind when the move
      is declined (D6).
- [x] 2.5 The login question: default from `--status`, yes installs and starts
      the agent, no removes it (D7).
- [x] 2.6 The key question: print the combination, take another one, check it
      with `--check-hotkey`, ask again on a spelling the app cannot read.
- [x] 2.7 Write the answers: only what departs from the defaults, the file
      removed when nothing does, and the launch agent rewritten in the same
      act so the two cannot disagree (D4, D5).
- [x] 2.8 The closing lines — the folder, the key, and what to press first.
      Read the whole conversation aloud once: step by step, human, and worth
      the minute it takes.

## 3. The bundle, the cask, the Makefile

- [x] 3.1 `scripts/bundle.sh` installs the script as
      `Contents/Resources/esse-setup`, executable, beside `login-item`.
- [x] 3.2 `resources/cask.rb.in`: the caveats become the one line to run plus
      the honest paragraph about the ad-hoc signature; `zap` takes the answers
      file along with the launch agent's plist, and still never the writing.
- [x] 3.3 `make setup` runs the installed copy's setup; `make login-item`
      keeps working as it does.

## 4. Walking it through on this machine

- [ ] 4.1 A fresh install: `brew install`, then the one line from the caveats
      — folder named, autostart on, key kept — and `ctrl-alt-e` answers
      without opening the app by hand.
- [ ] 4.2 A second run answered with `enter` all the way through: the folder,
      the agent and the answers file are byte-for-byte what they were.
- [ ] 4.3 A run that names a new folder while the old one holds essays: the
      count is right, the move lands, and the Shelf's foot says the new path
      after a launch from Finder.
- [ ] 4.4 A run that declines the login question: the agent is gone, esse
      still opens and still writes to the same folder.
- [x] 4.5 `esse-setup < /dev/null`: three facts printed, nothing changed.

## 5. The docs

- [x] 5.1 README: Installing becomes `brew install`, then the setup line, then
      what setup asks — with `login-item` kept below as the direct path.
- [x] 5.2 README, Data: the folder can be named at setup, and the order the
      app resolves it in.
- [x] 5.3 README, the file tree: `scripts/setup.sh` and what it becomes inside
      the bundle.
