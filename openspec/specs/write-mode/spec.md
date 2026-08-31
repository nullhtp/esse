# write-mode Specification

## Purpose

The fullscreen writing screen and the heart of the app: nothing but the
editor, the fragment being written, and a quiet session indicator.

It answers beginner problem #3 ("drafting and editing get mixed"): earlier
text fades and the viewport follows the cursor, so going back is inconvenient
without ever being blocked — friction, not a mechanical lock. Autosave on
every pause removes the other distraction, the question of whether the work is
safe.
## Requirements
### Requirement: Fullscreen flow screen
Write mode SHALL fill the window with the editor and nothing else — no
panels, toolbars, or navigation chrome; the only overlay is the quiet session
indicator. Entering Write mode SHALL make the window cover the whole display,
without a title bar and with the menu bar and the Dock out of the way while
esse is the active app; leaving SHALL restore the window that was there
before, at its previous size and position. The room MUST NOT be a separate
desktop Space: the summon key brings the writing forward where the writer is
already standing, and never moves the desktop out from under them. The
screen's look MUST be visibly distinct from the Today screen (dark,
flow-focused treatment), so the mode change is felt.

#### Scenario: Entering Write mode
- **WHEN** the user starts or continues an essay from the Today screen
- **THEN** the window covers the whole display with no title bar, showing only the editor and the session indicator, in the Write-mode visual treatment

#### Scenario: Leaving restores the window
- **WHEN** the user leaves Write mode
- **THEN** the window returns to the size and position it had before the room was entered, and the Today screen is shown

#### Scenario: Switching rooms does not resize the window
- **WHEN** the user switches between Write and Edit mode
- **THEN** the window keeps covering the whole display, with no intermediate window state

#### Scenario: The room survives being put away
- **WHEN** the user puts esse away with the summon key while in Write mode and summons it back
- **THEN** the same room comes back covering the whole display, on the desktop the user is on, with the text and the session as they were

### Requirement: Only the current fragment in focus
Write mode SHALL keep only the fragment being written at full visual
strength: text above the current paragraph fades toward the background. Free
scrolling SHALL be disabled — the viewport follows the cursor — but cursor
movement, clicking, and selection MUST work over the whole document. Going
back is inconvenient, never mechanically blocked.

#### Scenario: Earlier text is dimmed
- **WHEN** the document contains paragraphs above the one holding the cursor
- **THEN** those paragraphs render dimmed while the current paragraph renders at full strength

#### Scenario: The view follows the cursor, not the wheel
- **WHEN** the user scrolls with the mouse or trackpad in Write mode
- **THEN** the viewport does not scroll away from the cursor's centred position

#### Scenario: Going back is possible
- **WHEN** the user moves the cursor into earlier text with arrows or a click
- **THEN** the cursor arrives there, that paragraph becomes the focused fragment, and editing works

### Requirement: Autosave on pause
Write mode SHALL persist the essay body through the atomic essay save after
roughly a second's pause in editing, and always when leaving Write mode and
whenever the app goes away — quitting, the window closing, or esse being put
away by the summon key. `updated_at` follows each save. There is no manual
save and no unsaved-changes state surfaced to the user.

#### Scenario: A pause saves the draft
- **WHEN** the user stops typing for the pause interval
- **THEN** the essay file on disk contains the current body

#### Scenario: Leaving never loses text
- **WHEN** the user leaves Write mode or quits the app immediately after typing
- **THEN** the essay file on disk contains the text as last typed

#### Scenario: Being put away never loses text
- **WHEN** the user hides esse with the summon key or closes the window immediately after typing
- **THEN** the essay file on disk contains the text as last typed

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

