# today-screen Delta Specification

## MODIFIED Requirements

### Requirement: Today is the start screen
The app SHALL open directly into the Today screen in a single window. In this
stage the screen contains the "Write" button, the spark input, and the spark
list; later stages extend it with the published row and the session dots.

#### Scenario: Launch lands on Today
- **WHEN** the app is launched
- **THEN** the Today screen is shown with the "Write" button, the spark input, and the spark list visible

## ADDED Requirements

### Requirement: The Write button is the primary action
The Today screen SHALL show one big "Write" button as its most prominent
element. Activating it starts or continues writing as the start-from-spark
capability defines; the button itself carries no other behavior.

#### Scenario: Write is the dominant action
- **WHEN** the Today screen is shown
- **THEN** the "Write" button is visibly the primary action, above the spark input in prominence
