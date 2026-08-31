# brew-install — Design

## Context

`scripts/bundle.sh` already produces `Esse.app`, and `make install` copies it
into /Applications. What is missing is everything between "the author has a
bundle" and "anyone has the app": an artifact with a version on it, somewhere
to fetch it from, and a recipe that knows both.

Two macOS facts shape the rest. A `.app` is a directory, and only `ditto`
preserves its signature and symlinks through a zip. And Homebrew marks
everything a cask downloads with `com.apple.quarantine`, which on Apple
silicon turns an ad-hoc-signed app into a Gatekeeper dialog rather than an
app.

## Goals / Non-Goals

**Goals:**

- One line installs esse on a Mac that has never seen Rust.
- The same bundle, whether it arrives by Homebrew or by `make install`.
- The cask has one author — this repository — and the tap holds a copy.
- Someone who installed by Homebrew can still have esse at login.

**Non-Goals:**

- Notarisation, a Developer ID, auto-update, an Intel build.
- Any change to how the app behaves once it is running.

## Decisions

**D1. A cask, not a formula.** Homebrew formulae install binaries into a
prefix; casks install applications into /Applications, which is where a `.app`
with an `Info.plist`, an icon and an activation policy has to live to be an
app at all. A formula that built esse from source would also drag every
installer through the gpui build and the Metal Toolchain download — minutes
and hundreds of megabytes for something the author can build once.

**D2. `ditto`, not `zip`.** `ditto -c -k --keepParent` is the only archiver
that carries a bundle's symlinks, resource forks and code signature through
intact; a `zip -r` of an `.app` produces something that unpacks into a broken
application. The same tool unpacks it on the way in, which is what Homebrew
uses.

**D3. The cask clears the quarantine flag it set, and says so.** esse is
signed ad hoc — `codesign --sign -` — because notarisation costs a yearly
developer subscription for an app with one writer. Homebrew quarantines cask
downloads, and a quarantined ad-hoc-signed app on Apple silicon is refused
with a dialog about an unidentified developer. The choice is between a
`postflight` that removes the flag from the app it just installed, and an
install that visibly does not work. The cask takes the first and writes the
reason in its caveats, so nobody is quietly told that this app is more
trustworthy than it is: it is trustworthy exactly as far as its author is.

**D4. The cask is generated from a template here.** `resources/cask.rb.in`
carries everything but the version and the checksum, and `scripts/release.sh`
renders it after computing them. The tap gets a generated file, so the two
copies cannot drift into two different opinions about what esse is.

**D5. The launch agent moves into the bundle.** `scripts/login-item.sh` writes
the agent, bootstraps it, and removes it again with `--off`; `bundle.sh` copies
it to `Contents/Resources/login-item`, where it can find its own executable by
walking up from where it is. A Homebrew install therefore carries its own way
to start at login, and `make login-item` calls the same script rather than
keeping a second copy of the same `sed`.

**D6. arm64 only, declared.** The app is built and used on Apple silicon. An
x86_64 build would be untested on every machine including the author's, and a
cask that installs a broken app is worse than a cask that refuses to install.
`depends_on arch: :arm64` makes the refusal explicit and early.

## Risks / Trade-offs

- **A release is published by hand** → `make release` does the whole
  sequence in one command, and the checksum in the cask is computed from the
  artifact that was actually uploaded, never typed.
- **The tap's copy of the cask goes stale** → it is generated, and the release
  script prints the file to copy; a stale copy points at a version tag that
  still exists, so nothing breaks silently — it simply installs the older esse.
- **Removing the quarantine flag weakens Gatekeeper for this app** → it is the
  author's own build, fetched over TLS from a release in the author's own
  repository, and the checksum in the cask is checked before anything is
  unpacked. The caveats say what was traded.
- **Homebrew's cask audit dislikes `xattr` in postflight** → the cask lives in
  a personal tap, which is exactly the place for a personal trade-off; nothing
  here is proposed to homebrew-cask.

## Migration Plan

1. Tag the version, `make release`, copy the rendered cask into the tap.
2. `brew install nullhtp/tap/esse` on a machine that has never built esse.
3. An existing `make install` copy is simply replaced by the cask's copy;
   the writing in `~/Documents/Esse` is not touched by either.

## Open Questions

None.
