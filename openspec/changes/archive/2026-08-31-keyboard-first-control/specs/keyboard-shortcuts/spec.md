# keyboard-shortcuts Delta

## ADDED Requirements

### Requirement: Every operation is keyboard-drivable
Every operation the app offers SHALL be operable without a pointer: the
daily loop (a shortcut triggers the Write button's action on Today, a
shortcut opens the Shelf, Escape and the existing mode keys return and
switch), choosing a spark when writing starts, the whole completion flow,
the Shelf's columns, cards, and drawer, and applying markdown-lite
formatting in the editor. Keyboard paths SHALL trigger exactly the same
actions and transitions as the pointer paths, through the same enforcement
points.

#### Scenario: Starting to write from the keyboard
- **WHEN** the user presses the Write shortcut on the Today screen
- **THEN** the app behaves exactly as if the Write button had been clicked

#### Scenario: Reaching the Shelf and back from the keyboard
- **WHEN** the user presses the Shelf shortcut on the Today screen, and presses it again on the Shelf
- **THEN** the Shelf screen is shown, and then the Today screen again

#### Scenario: The full conveyor without a pointer
- **WHEN** the user captures a spark, starts writing from it, writes, switches to Edit mode, and publishes — using only the keyboard
- **THEN** every step completes exactly as it would by pointer, and at no point is a pointer action required

## MODIFIED Requirements

### Requirement: Shortcuts yield to typing
Every shortcut available on the resting Today screen MUST use a modifier
key, so that plain typing — including Cyrillic and IME composition — always
lands in the spark input and never triggers navigation. Transient states
entered by an explicit action and left by a single key (choosing a spark,
the help overlay) MAY bind unmodified keys while they are open, because the
capture line does not hold focus there.

#### Scenario: Plain typing is capture, not navigation
- **WHEN** the user types unmodified characters on the resting Today screen
- **THEN** the characters go into the spark input and no screen change or action is triggered

#### Scenario: A transient state hands the keys back
- **WHEN** the user leaves the spark-choosing state with Escape
- **THEN** focus returns to the spark input and plain typing is capture again

### Requirement: Shortcuts are chrome, not a feature
Shortcuts SHALL have no persistent in-app surface: no shortcut hints layered
onto any screen and no customization UI, ever. The single discoverability
surface is the on-demand help overlay defined by the shortcut-help
capability, visible only while summoned.

#### Scenario: No shortcut UI anywhere
- **WHEN** the user visits the Today, editor, and Shelf screens without summoning help
- **THEN** no shortcut listing, hint overlay, or customization control is present

#### Scenario: Discoverability exists only on demand
- **WHEN** the user summons the help overlay and dismisses it
- **THEN** the shortcut listing was visible only between those two keypresses

## REMOVED Requirements

### Requirement: The daily loop is keyboard-drivable
**Reason**: Superseded — the daily loop was the keyboard-drivable subset;
the widened requirement covers every operation, the loop included.
**Migration**: Covered in full by "Every operation is keyboard-drivable";
the existing bindings and behavior are unchanged.
