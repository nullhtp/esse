# brew-install — Tasks

## 1. The login agent leaves the source tree

- [x] 1.1 `scripts/login-item.sh`: write the agent for a given executable,
      bootstrap it, and take it out again with `--off`; honour `HOTKEY` (D5).
- [x] 1.2 The script finds its own app when it is run from inside the bundle,
      so an installed copy needs no arguments.
- [x] 1.3 `scripts/bundle.sh` copies it to `Contents/Resources/login-item`,
      executable; the plist template in `resources/` goes, since the script
      writes the agent itself.
- [x] 1.4 `make login-item` calls the script instead of keeping its own `sed`.

## 2. The released build

- [x] 2.1 `scripts/release.sh`: bundle, then `ditto -c -k --keepParent` into
      `target/Esse-<version>-arm64.zip`, then the SHA-256 (D2).
- [x] 2.2 The same script renders `resources/cask.rb.in` into
      `target/esse.rb` with the version and the checksum filled in (D4).
- [x] 2.3 It creates the GitHub release for the version tag and attaches the
      zip, or attaches to the release that is already there.
- [x] 2.4 `make release` runs it.

## 3. The cask

- [x] 3.1 `resources/cask.rb.in`: name, homepage, `app "Esse.app"`, macOS and
      arm64 requirements (D1, D6).
- [x] 3.2 `postflight` clears the quarantine flag, and `caveats` says why and
      what it means (D3).
- [x] 3.3 `uninstall` quits the app and unloads the agent; `zap` takes the
      agent's plist and nothing else — never the writing.
- [x] 3.4 The caveats carry the summon key and the login-item command.

## 4. Publishing

- [ ] 4.1 Tag `v0.1.0`, run `make release`.
- [ ] 4.2 Put the rendered cask in `nullhtp/homebrew-tap` as `Casks/esse.rb`.
- [ ] 4.3 Install it on this machine from the tap and open the app: the whole
      point is that this path works.

## 5. The docs

- [x] 5.1 README: Homebrew first in the Installing section, `make install`
      after it as the from-source path.
- [x] 5.2 README: what the ad-hoc signature means, in one honest sentence.
- [x] 5.3 README: the login-item command for an installed copy.
