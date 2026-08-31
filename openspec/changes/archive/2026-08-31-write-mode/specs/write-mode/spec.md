# write-mode Delta Specification

## ADDED Requirements

### Requirement: Fullscreen flow screen
Write mode SHALL fill the window with the editor and nothing else — no
panels, toolbars, or navigation chrome; the only overlay is the quiet session
indicator. Entering Write mode SHALL request native fullscreen for the
window; leaving SHALL restore the previous window state. The screen's look
MUST be visibly distinct from the Today screen (dark, flow-focused
treatment), so the mode change is felt.

#### Scenario: Entering Write mode
- **WHEN** the user starts or continues an essay from the Today screen
- **THEN** the window enters fullscreen and shows only the editor and the session indicator, in the Write-mode visual treatment

#### Scenario: Leaving restores the window
- **WHEN** the user leaves Write mode
- **THEN** fullscreen is exited, the previous window state is restored, and the Today screen is shown

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
when the app quits or the window closes. `updated_at` follows each save.
There is no manual save and no unsaved-changes state surfaced to the user.

#### Scenario: A pause saves the draft
- **WHEN** the user stops typing for the pause interval
- **THEN** the essay file on disk contains the current body

#### Scenario: Leaving never loses text
- **WHEN** the user leaves Write mode or quits the app immediately after typing
- **THEN** the essay file on disk contains the text as last typed

### Requirement: Leaving Write mode is explicit
Write mode SHALL be left only by an explicit action (the Escape key or an
equivalent quiet control). Leaving saves the essay, records the session, and
returns to Today; the essay stays in Draft.

#### Scenario: Escape ends the writing session
- **WHEN** the user presses Escape in Write mode (outside IME composition)
- **THEN** the essay is saved, the session is recorded, and the Today screen is shown
