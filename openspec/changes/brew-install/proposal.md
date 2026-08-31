# brew-install — Proposal

## Why

esse installs with `make install`, which means: clone the repository, install
a Rust toolchain, download the Metal Toolchain component, and wait out a
seven-hundred-crate build. That is a developer's install, and it stands
between the app and the one thing it is for — writing. A person who wants to
write should get the app in a minute, on any Mac, with one line they already
know how to type.

This serves the same beginner problem #2 ("cannot start") the summon key
serves, one step earlier: the cost of beginning is not only the keystroke that
opens esse but everything that had to happen before there was an esse to open.

## What Changes

- **A released build.** `make release` assembles the bundle, zips it the way
  macOS wants a signed bundle zipped, and attaches it to a GitHub release
  under a version tag. That artifact is the thing people install.
- **A Homebrew cask** in the author's existing tap: `brew install
  nullhtp/tap/esse` puts Esse.app in /Applications, and `brew uninstall
  --cask esse` takes it away. The cask is generated from a template in this
  repository, so the tap holds a copy rather than a second source of truth.
- **The quarantine flag is cleared by the cask.** The app is signed ad hoc —
  it is built by its author, for its author — so a downloaded copy would
  otherwise be refused by Gatekeeper with a dialog about an unverified
  developer. The cask says so out loud in its caveats rather than leaving a
  broken app in /Applications.
- **Starting at login without the repository.** The launch-agent script moves
  into the bundle, so a person who installed with Homebrew can run one command
  to have esse waiting for its key from the first minute of the day. `make
  login-item` runs the same script.

## Capabilities

### New Capabilities

None. This is the same "esse installs as a desktop application" capability,
reached by a shorter road.

### Modified Capabilities

- `summon-from-anywhere`: the install requirement gains the released build and
  the one-command install, and the login agent becomes something the installed
  app carries rather than something the source tree does.

## Non-goals

- No auto-update, no Sparkle, no update checks. `brew upgrade` is the update
  mechanism, and it is the writer's to run.
- No Apple Developer ID, no notarisation: signing esse properly costs a yearly
  fee for an app with one user. The cask is honest about what that means.
- No Intel build. The app is built and used on Apple silicon, and shipping an
  untested x86_64 binary is worse than not shipping one.
- No `brew install esse` in homebrew-core: a personal tool belongs in a
  personal tap.
- No installer package, no DMG with a drag-here background.

## Impact

- `scripts/release.sh` (new) — the zip, the checksum, the release, and the
  rendered cask.
- `scripts/login-item.sh` (new) — the launch agent, installed and removed from
  one place; copied into the bundle by `scripts/bundle.sh`.
- `resources/cask.rb.in` (new) — the cask, with the version and checksum left
  blank for the release to fill in.
- `Makefile` — `release`, and `login-item` delegating to the script.
- `README.md` — Homebrew first, `make install` after it.
- `nullhtp/homebrew-tap` — `Casks/esse.rb`, generated.
