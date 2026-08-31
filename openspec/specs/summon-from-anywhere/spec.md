# summon-from-anywhere Specification

## Purpose

esse as something installed rather than run: an application bundle in
/Applications, a process that stays alive between sessions, and one
system-wide key that brings it forward and puts it away again.

It answers beginner problem #2 ("cannot start") from outside the app: the cost
of beginning a session is one keypress from inside whatever you were doing,
with no window to find and no launcher to open. It answers problem #1 in the
same motion, since a spark is only worth catching while it is still in your
head. Everything here is presence, not behaviour — what esse is on the machine
between sessions, and how little of it shows.
## Requirements
### Requirement: esse installs as a desktop application
esse SHALL ship as a macOS application bundle — a release binary, an
`Info.plist` naming the app and its identifier, and an icon — installable
into /Applications with one command, and SHALL run from there with no
toolchain present. The bundle SHALL declare itself a background application:
no Dock icon and no menu bar, since the app is reached by its key and by its
window, not from the Dock.

#### Scenario: Installed and launched
- **WHEN** the app is installed into /Applications and opened
- **THEN** esse runs with its window on screen and no toolchain is needed to launch it

#### Scenario: Nothing in the Dock
- **WHEN** esse is running
- **THEN** it has no Dock icon and no menu bar of its own

### Requirement: The app stays resident
Closing the window SHALL put esse away rather than quit it: the essay is
saved first, exactly as on every other way out, and the app keeps running
and keeps answering the summon key. Quitting SHALL remain an explicit act —
`cmd-q` — and SHALL save the essay the same way before it ends.

#### Scenario: Closing the window keeps the app
- **WHEN** the user closes the window while writing
- **THEN** the essay on disk holds the text as last typed, the window is gone, and esse is still running

#### Scenario: Quitting is still quitting
- **WHEN** the user presses `cmd-q`
- **THEN** the essay is saved and the app exits

### Requirement: One key summons esse from anywhere
esse SHALL register a system-wide key combination — `ctrl-alt-e` unless the
installation names another one — and pressing it from any application SHALL
bring esse forward with its window focused and ready to type, on the screen
the writer left it on. Pressing it while esse is the active application
SHALL put esse away again, saving as any other way out does. The
combination MAY be named at install time; it SHALL NOT be configurable from
inside the app, and there SHALL be no shortcut recorder, settings screen or
menu-bar control for it.

#### Scenario: Summoned from another app
- **WHEN** the user presses the key while working in another application
- **THEN** esse comes forward, its window focused, showing the screen it was left on

#### Scenario: The same key puts it away
- **WHEN** the user presses the key while esse is the active application
- **THEN** the essay is saved and esse is hidden, leaving the previous application in front

#### Scenario: The combination can be named at install time
- **WHEN** the installation names a combination in the environment
- **THEN** that combination is registered instead of the default

#### Scenario: A combination that is already taken
- **WHEN** the key cannot be registered because another application holds it
- **THEN** esse starts and works normally, and the failure is reported through the log rather than a dialog

### Requirement: Starting without a window
esse SHALL be able to start with no window on screen, so that it can be
launched at login and wait for the summon key. In that state the app SHALL
have registered the key and SHALL open the window on the first summon,
without any further launch.

#### Scenario: Started at login
- **WHEN** esse is launched with the environment asking it to start hidden
- **THEN** no window appears, and the first press of the summon key shows the app ready to type

#### Scenario: An ordinary launch still shows the window
- **WHEN** esse is launched without that request
- **THEN** the window opens as it always has, focused on the spark line

