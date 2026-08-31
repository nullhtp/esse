## ADDED Requirements

### Requirement: The Shelf says where the files are
The Shelf SHALL show the data folder's path in one quiet line at its foot,
written the way a person reads a path (the home directory as `~`), and
SHALL open that folder in the system file browser on `cmd-o` or a click on
the line. The line states a fact and offers no other action: no folder
picker, no move, no file listing inside the app.

#### Scenario: The path is on the screen
- **WHEN** the Shelf is opened
- **THEN** the path of the folder holding the sparks, sessions and essays is shown at its foot

#### Scenario: The folder opens from the keyboard
- **WHEN** the user presses `cmd-o` on the Shelf
- **THEN** the data folder opens in the system file browser and the Shelf stays as it was

#### Scenario: The line offers nothing else
- **WHEN** the user looks at the line or activates it
- **THEN** the only thing on offer is opening the folder — no picker, no move, no in-app file listing
