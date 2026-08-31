# keyboard-shortcuts — Delta

## ADDED Requirements

### Requirement: The daily loop is keyboard-drivable
The core loop SHALL be operable without a pointer: from the Today screen a
keyboard shortcut SHALL trigger the Write button's action (start or continue
writing, exactly as defined by start-from-spark), and a keyboard shortcut
SHALL open the Shelf. Together with the existing bindings (Escape leaves the
Shelf and either editor mode; the mode switch has its shortcut), this closes
the loop: capture, write, browse, return — all from the keyboard.

#### Scenario: Starting to write from the keyboard
- **WHEN** the user presses the Write shortcut on the Today screen
- **THEN** the app behaves exactly as if the Write button had been clicked

#### Scenario: Reaching the Shelf from the keyboard
- **WHEN** the user presses the Shelf shortcut on the Today screen
- **THEN** the Shelf screen is shown

#### Scenario: Returning from the Shelf by the same gesture
- **WHEN** the user presses the Shelf shortcut on the Shelf screen
- **THEN** the app returns to the Today screen

### Requirement: Shortcuts yield to typing
Every shortcut available on the Today screen MUST use a modifier key, so that
plain typing — including Cyrillic and IME composition — always lands in the
spark input and never triggers navigation.

#### Scenario: Plain typing is capture, not navigation
- **WHEN** the user types unmodified characters on the Today screen
- **THEN** the characters go into the spark input and no screen change or action is triggered

### Requirement: Shortcuts are chrome, not a feature
Shortcuts SHALL have no in-app surface: no cheat-sheet screen, no shortcut
hints layered onto the interface, and no customization UI.

#### Scenario: No shortcut UI anywhere
- **WHEN** the user visits the Today, editor, and Shelf screens
- **THEN** no shortcut listing, hint overlay, or customization control is present
