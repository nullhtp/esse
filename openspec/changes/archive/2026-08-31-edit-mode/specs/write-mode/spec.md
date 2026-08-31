# write-mode Delta

## MODIFIED Requirements

### Requirement: Leaving Write mode is explicit
Write mode SHALL be left only by an explicit action, of which there are two.
The Escape key (or an equivalent quiet control) saves the essay, records the
session, and returns to Today with the essay staying in Draft. The mode
switch (a small corner control and the `cmd-e` shortcut) saves the essay,
records the session, transitions the essay Draft → Editing through the
essay-lifecycle rules, and shows Edit mode with the same text, without
leaving fullscreen. If the switch's save or transition fails, the app SHALL
stay in Write mode and surface the error quietly.

#### Scenario: Escape ends the writing session
- **WHEN** the user presses Escape in Write mode (outside IME composition)
- **THEN** the essay is saved, the session is recorded, and the Today screen is shown

#### Scenario: Switching to Edit mode ends the session too
- **WHEN** the user activates the mode switch in Write mode
- **THEN** the essay is saved, the session is recorded, the essay's state becomes Editing, and Edit mode is shown with the same text and the window still fullscreen

#### Scenario: A failed switch stays put
- **WHEN** the switch's save or state transition fails
- **THEN** Write mode remains on screen with the text intact and the error shown quietly
