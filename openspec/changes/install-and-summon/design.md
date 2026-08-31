# install-and-summon — Design

## Context

Today `main` opens one window and quits when it closes: "esse has nothing to
stay resident for". That sentence is what this change overturns — a key that
works from inside another application needs a process to press it against.

Two facts about gpui shape everything below. It runs an ordinary
`NSApplication` on the main thread, so Carbon's `RegisterEventHotKey` — the
oldest and least privileged way to hold a system-wide key — dispatches through
the run loop esse already has, with no accessibility permission and no event
tap. And gpui sets `NSApplicationActivationPolicyRegular` itself in
`applicationDidFinishLaunching`, which lands the app in the Dock no matter what
`Info.plist` says; our `run` callback happens after that, which is the one
moment the policy can be set back.

## Goals / Non-Goals

**Goals:**

- One command installs an app a person can open like any other.
- One key reaches esse from anywhere, and the same key puts it away.
- Nothing on screen when nothing is wanted: no Dock icon, no menu bar, no
  status item.
- Never a lost paragraph: every new way out saves exactly like the old ones.

**Non-Goals:**

- Configuring the key inside the app, a recorder, a settings screen.
- A separate quick-capture panel — the summon key opens esse itself.
- Signing, notarising, updating, or distributing to anyone but the author.
- Any second window, or a window per screen.

## Decisions

**D1. Carbon hot keys through `global-hotkey`, not an event tap.**
`RegisterEventHotKey` needs no permission, fires on the physical key
regardless of the keyboard layout — which matters when half the writing
happens in a Russian layout — and cannot see any keystroke that is not the
registered combination. An event tap (`CGEventTapCreate`) would need
accessibility access and would be able to read everything typed on the
machine, which is not a trade a writing app should ask for. The
`global-hotkey` crate wraps the Carbon call, installs its handler on
`GetApplicationEventTarget()`, and so lives inside gpui's own run loop; its
media-key path, the one part that does use an event tap, is never touched.

**D2. `ctrl-alt-e` by default, `ESSE_HOTKEY` at install time.** A system-wide
key is taken from every other application on the machine, so the default has
to be one almost nothing wants: `ctrl-alt-e` is free on macOS, free in the
editors esse is likely to sit next to, and reachable with one hand. Since a
collision is personal — nobody can pick a combination that is right on every
machine — the launch agent may name another one in `ESSE_HOTKEY`, in the same
`cmd-shift-e` spelling the app uses everywhere else. That is a fact of the
installation, like `ESSE_DATA_DIR`, not a setting: there is no UI for it, and
the app never writes it.

**D3. A press toggles.** Summoning something that is already in front of you
should put it away — that is what makes one key enough, and it is the gesture
every launcher has taught. "Is esse in front" is `window.is_window_active()`
plus the app being active; if it is, the essay is flushed and `App::hide` puts
esse away, and if it is not, the app is unhidden, activated and the window
focused. The window itself is never destroyed, so a summon is a hide away from
being ready — measured in a frame, not in a launch.

**D4. Closing the window hides the app instead of quitting it.**
`on_window_should_close` already saves the essay and the session; it now
returns `false` and hides, so the one window esse has lives as long as the
process. The alternative — letting the window close and building a new one on
the next summon — throws away the editor's state and makes the summon slow
exactly when it should feel instant. `cmd-q` is untouched and remains the way
out, with its own save through `on_app_quit`.

**D5. The activation policy is set back to Accessory in our own callback.**
gpui hard-codes Regular in `applicationDidFinishLaunching` and offers no API
for it, so esse calls `setActivationPolicy(NSApplicationActivationPolicyAccessory)`
through `objc` at the top of its `run` closure — after gpui, which is what
makes it stick. `LSUIElement` stays in `Info.plist` as well, so a future gpui
that stops forcing the policy needs no code change here. An accessory
application has no Dock icon and no menu bar; esse defines no application menu
anyway, and every key it answers to is its own.

**D6. Starting hidden is an environment flag, not a mode.** The launch agent
sets `ESSE_START_HIDDEN=1`; the app builds its window as usual and hides
immediately, so the first summon is as fast as every later one and the login
path shares every line of the ordinary one. Building the window lazily was the
alternative and buys nothing but a slower first press.

**D7. The summon key is not in the shortcut table.** `keymap.rs` is the list of
keys the app answers to while you are standing in it, and the help sheet is
its shadow. The summon key belongs to the system: it works when esse is not
even on screen, it is chosen at install time, and it cannot be listed
faithfully in a sheet that says "what works here". The README is where it is
written down.

**D8. `make install`, a shell script, and files that are read as files.** The
bundle is `scripts/bundle.sh`: `cargo build --release`, then a directory tree
with the binary, `Info.plist` and the icon copied from `resources/`. No
`cargo-bundle` and no packaging crate — the whole job is six `cp` invocations,
and a script in the repository is inspectable in a way a build-time dependency
is not. `make install` copies the bundle to /Applications; `make login-item`
writes the launch agent into `~/Library/LaunchAgents` and loads it; both are
undone by `make uninstall`.

## Risks / Trade-offs

- **A resident app is a process the writer forgot about** → it has no Dock
  icon to remind them, so the only way it announces itself is the key. That is
  the point, and `cmd-q` from the summoned window is always the way out.
- **`ctrl-alt-e` collides with something on this machine** → registration
  fails, esse logs it and runs on; the writer names another combination in the
  launch agent. The app never dies over a key.
- **gpui stops forcing the activation policy, or starts fighting the accessory
  one** → the `Info.plist` already declares `LSUIElement`, and the objc call is
  three lines in one place.
- **A hidden app with an essay in memory** → every path that hides also
  flushes, which is the same code the window close and the quit already run.
- **/Applications is not writable** → `make install` fails loudly with the
  path; nothing partial is left behind.

## Migration Plan

1. `make install` once; `make login-item` if the key should work from login.
2. `cargo run -p esse-app` keeps working unchanged for development — it simply
   runs a resident, Dock-less app in the terminal's foreground.
3. Rollback is `make uninstall` and the previous binary: nothing about the data
   or the file formats changes here.

## Open Questions

None.
