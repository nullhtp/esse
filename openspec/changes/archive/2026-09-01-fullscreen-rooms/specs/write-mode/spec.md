# write-mode — Delta

## MODIFIED Requirements

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
