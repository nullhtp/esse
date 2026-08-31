## MODIFIED Requirements

### Requirement: esse installs as a desktop application
esse SHALL ship as a macOS application bundle — a release binary, an
`Info.plist` naming the app and its identifier, and an icon — and SHALL be
installable in one command by someone with no toolchain, no clone of the
repository and no knowledge of how it was built: a released build attached to
a version tag, and a Homebrew cask that puts it in /Applications and takes it
away again. The install SHALL be finished by one further command, carried by
the bundle and named in the installer's own output, which settles where the
writing lives, whether esse waits at login and which key reaches it. Building
it from the source tree SHALL remain equally possible, and SHALL offer the
same command. The installed app SHALL open without the writer having to clear
a quarantine flag by hand, and the installer SHALL say plainly that the app is
signed by its author rather than notarised by Apple. The bundle SHALL declare
itself a background application: no Dock icon and no menu bar, since the app is
reached by its key and by its window, not from the Dock.

#### Scenario: Installed and launched
- **WHEN** the app is installed into /Applications and opened
- **THEN** esse runs with its window on screen and no toolchain is needed to launch it

#### Scenario: Installed in one command
- **WHEN** someone with no clone and no toolchain installs esse from the tap
- **THEN** Esse.app is in /Applications, opens on the first attempt, and uninstalls the same way

#### Scenario: The install says how to finish itself
- **WHEN** the install completes
- **THEN** its output names one command that asks where the essays live, whether esse waits at login, and which key brings it forward

#### Scenario: Nothing in the Dock
- **WHEN** esse is running
- **THEN** it has no Dock icon and no menu bar of its own

### Requirement: One key summons esse from anywhere
esse SHALL register a system-wide key combination — `ctrl-alt-e` unless the
installation names another one — and pressing it from any application SHALL
bring esse forward with its window focused and ready to type, on the screen
the writer left it on. Pressing it while esse is the active application
SHALL put esse away again, saving as any other way out does. The
combination MAY be named at install time, and one named there SHALL be
answered however esse was started — from the launch agent, from Finder, or
from the source tree. It SHALL NOT be configurable from inside the app, and
there SHALL be no shortcut recorder, settings screen or menu-bar control for
it.

#### Scenario: Summoned from another app
- **WHEN** the user presses the key while working in another application
- **THEN** esse comes forward, its window focused, showing the screen it was left on

#### Scenario: The same key puts it away
- **WHEN** the user presses the key while esse is the active application
- **THEN** the essay is saved and esse is hidden, leaving the previous application in front

#### Scenario: The combination can be named at install time
- **WHEN** the installation names a combination in the environment
- **THEN** that combination is registered instead of the default

#### Scenario: A combination named at setup outlives the launch agent
- **WHEN** a combination was named at setup and esse is opened without the launch agent, with nothing naming a combination in the environment
- **THEN** that combination is registered instead of the default

#### Scenario: A combination that is already taken
- **WHEN** the key cannot be registered because another application holds it
- **THEN** esse starts and works normally, and the failure is reported through the log rather than a dialog

### Requirement: Starting without a window
esse SHALL be able to start with no window on screen, so that it can be
launched at login and wait for the summon key. In that state the app SHALL
have registered the key and SHALL open the window on the first summon,
without any further launch. The installed bundle SHALL carry the means of
starting itself at login, so that this does not depend on having the source
tree, and the setup command SHALL offer it as a question rather than leaving
it as a command to find.

#### Scenario: Started at login
- **WHEN** esse is launched with the environment asking it to start hidden
- **THEN** no window appears, and the first press of the summon key shows the app ready to type

#### Scenario: An ordinary launch still shows the window
- **WHEN** esse is launched without that request
- **THEN** the window opens as it always has, focused on the spark line

#### Scenario: Login start from an installed copy
- **WHEN** someone who installed esse with Homebrew runs setup and accepts the login question
- **THEN** the launch agent is installed and started, and esse answers its key without the app being opened by hand

#### Scenario: Login start without setup
- **WHEN** someone drives the bundle's launch-agent command directly, as before
- **THEN** one command installs the launch agent and one removes it, unchanged
