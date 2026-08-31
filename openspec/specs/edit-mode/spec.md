# edit-mode Specification

## Purpose

The other half of the editor: the room an essay is read whole, rearranged and
cut in. Light where Write mode is dark, the entire text at full strength where
Write mode fades what is already written, free to scroll where Write mode pins
the caret to the middle of the screen.

It is the second half of the answer to beginner problem #3 ("drafting and
editing get mixed"): editing gets its own room, visibly different from the
writing one, so the two processes stay physically separate and the writer
always knows which job they are doing. The switch between the rooms is
explicit and persists the essay's Draft ↔ Editing state, so the mode on screen
and the essay's state cannot disagree.

## Requirements

### Requirement: Calm full-text screen
Edit mode SHALL fill the window with the editor and nothing else — no panels,
toolbars, or navigation chrome; the only overlays are the quiet mode-switch
control, the quiet finish control that opens the essay-completion flow, and,
when needed, the save-trouble notice. The whole text renders at full strength
with no dimming, in a light, calm treatment that MUST be visibly distinct
from Write mode's dark flow treatment. Entering Edit mode from the Today
screen SHALL request native fullscreen; leaving to Today SHALL restore the
previous window state.

#### Scenario: Entering Edit mode shows the whole essay
- **WHEN** the user opens an essay that is in Editing state
- **THEN** the window enters fullscreen and shows the full text at full strength in the light Edit treatment

#### Scenario: The two modes look different at a glance
- **WHEN** the user switches between Write mode and Edit mode
- **THEN** the background and text treatment change between the dark flow look and the light calm look

#### Scenario: The finish control stays quiet
- **WHEN** the user is editing in Edit mode without touching the finish control
- **THEN** the finish control remains a small unobtrusive overlay and nothing else is added to the screen

### Requirement: Free navigation and scrolling
Edit mode SHALL allow free movement through the whole document: the mouse
wheel and trackpad scroll the view without moving the cursor, and clicking,
selecting, and editing work anywhere in the text. There is no typewriter
centring; when an edit or a cursor movement would put the caret outside the
view, the view SHALL scroll just enough to keep the caret visible.

#### Scenario: Scrolling leaves the cursor in place
- **WHEN** the user scrolls with the wheel or trackpad in Edit mode
- **THEN** the view moves through the document and the cursor position in the text does not change

#### Scenario: Typing pulls the caret into view
- **WHEN** the caret is outside the visible area and the user types or moves the cursor
- **THEN** the view scrolls the minimum needed to make the caret visible, without centring it

### Requirement: Autosave on pause
Edit mode SHALL persist the essay body through the atomic essay save after
roughly a second's pause in editing, and always when leaving Edit mode and
when the app quits or the window closes. `updated_at` follows each save.
There is no manual save and no unsaved-changes state surfaced to the user.

#### Scenario: A pause saves the text
- **WHEN** the user stops editing for the pause interval
- **THEN** the essay file on disk contains the current body

#### Scenario: Leaving never loses text
- **WHEN** the user leaves Edit mode or quits the app immediately after editing
- **THEN** the essay file on disk contains the text as last edited

### Requirement: Explicit switch back to Write mode
Edit mode SHALL offer an explicit, quiet switch to Write mode (a small corner
control and the `cmd-e` shortcut). Switching saves the essay, transitions it
Editing → Draft through the essay-lifecycle rules, and shows Write mode with
the same text, without leaving fullscreen. If the save or the transition
fails, the app SHALL stay in Edit mode and surface the error quietly.

#### Scenario: Switching lands in Write mode as a Draft
- **WHEN** the user activates the mode switch in Edit mode
- **THEN** the essay is saved, its state becomes Draft, and Write mode is shown with the same text and the window still fullscreen

#### Scenario: A failed switch stays put
- **WHEN** the switch's save or state transition fails
- **THEN** Edit mode remains on screen with the text intact and the error shown quietly

### Requirement: Leaving Edit mode is explicit
Edit mode SHALL be left only by an explicit action (the Escape key or an
equivalent quiet control). Leaving saves the essay and returns to Today; the
essay stays in Editing. No session is recorded — sessions belong to Write
mode.

#### Scenario: Escape returns to Today
- **WHEN** the user presses Escape in Edit mode (outside IME composition)
- **THEN** the essay is saved, remains in Editing state, and the Today screen is shown with the window state restored
