## MODIFIED Requirements

### Requirement: esse installs as a desktop application
esse SHALL ship as a macOS application bundle — a release binary, an
`Info.plist` naming the app and its identifier, and an icon — and SHALL be
installable in one command by someone with no toolchain, no clone of the
repository and no knowledge of how it was built: a released build attached to
a version tag, and a Homebrew cask that puts it in /Applications and takes it
away again. Building it from the source tree SHALL remain equally possible.
The installed app SHALL open without the writer having to clear a quarantine
flag by hand, and the installer SHALL say plainly that the app is signed by
its author rather than notarised by Apple. The bundle SHALL declare itself a
background application: no Dock icon and no menu bar, since the app is
reached by its key and by its window, not from the Dock.

#### Scenario: Installed and launched
- **WHEN** the app is installed into /Applications and opened
- **THEN** esse runs with its window on screen and no toolchain is needed to launch it

#### Scenario: Installed in one command
- **WHEN** someone with no clone and no toolchain installs esse from the tap
- **THEN** Esse.app is in /Applications, opens on the first attempt, and uninstalls the same way

#### Scenario: Nothing in the Dock
- **WHEN** esse is running
- **THEN** it has no Dock icon and no menu bar of its own

### Requirement: Starting without a window
esse SHALL be able to start with no window on screen, so that it can be
launched at login and wait for the summon key. In that state the app SHALL
have registered the key and SHALL open the window on the first summon,
without any further launch. The installed bundle SHALL carry the means of
starting itself at login, so that this does not depend on having the source
tree.

#### Scenario: Started at login
- **WHEN** esse is launched with the environment asking it to start hidden
- **THEN** no window appears, and the first press of the summon key shows the app ready to type

#### Scenario: An ordinary launch still shows the window
- **WHEN** esse is launched without that request
- **THEN** the window opens as it always has, focused on the spark line

#### Scenario: Login start from an installed copy
- **WHEN** someone who installed esse with Homebrew asks it to start at login
- **THEN** one command carried by the app itself installs the launch agent, and one removes it
