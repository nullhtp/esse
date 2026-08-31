# today-screen Specification

## Purpose

The app's main screen and entry point — the frame the whole product opens
into, in a single window.

It holds the big "Write" button, the spark input, and the spark list, and it
opens capture-ready so that recording an idea is launch → type → Enter. Later
stages extend the same screen with the row of published essays and the session
dots.

## Requirements

### Requirement: Today is the start screen
The app SHALL open directly into the Today screen in a single window. In this
stage the screen contains the "Write" button, the spark input, and the spark
list; later stages extend it with the published row and the session dots.

#### Scenario: Launch lands on Today
- **WHEN** the app is launched
- **THEN** the Today screen is shown with the "Write" button, the spark input, and the spark list visible

### Requirement: The Write button is the primary action
The Today screen SHALL show one big "Write" button as its most prominent
element. Activating it starts or continues writing as the start-from-spark
capability defines; the button itself carries no other behavior.

#### Scenario: Write is the dominant action
- **WHEN** the Today screen is shown
- **THEN** the "Write" button is visibly the primary action, above the spark input in prominence

### Requirement: Capture-ready on launch
The spark input SHALL have keyboard focus when the app opens, so capturing a
spark is launch → type → Enter, with no click or navigation in between.

#### Scenario: Typing immediately after launch
- **WHEN** the user launches the app and starts typing
- **THEN** the typed text goes into the spark input without any prior interaction

### Requirement: Quiet navigation to the Shelf
The Today screen SHALL offer a quiet affordance that opens the Shelf screen.
It MUST be visually subordinate to the "Write" button and the spark input —
navigation, not a competing action — and MUST NOT take keyboard focus away
from the spark input on launch.

#### Scenario: Opening the Shelf from Today
- **WHEN** the user activates the Shelf affordance on the Today screen
- **THEN** the Shelf screen is shown

#### Scenario: Capture stays primary
- **WHEN** the app launches and the user starts typing
- **THEN** the typed text still goes into the spark input, with the Shelf affordance untouched
