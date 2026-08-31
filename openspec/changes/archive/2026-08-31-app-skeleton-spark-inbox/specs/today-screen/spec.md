# today-screen Delta

## ADDED Requirements

### Requirement: Today is the start screen
The app SHALL open directly into the Today screen in a single window. In this
stage the screen contains the spark input and the spark list; later stages
extend it with the "Write" button, the published row, and the session dots.

#### Scenario: Launch lands on Today
- **WHEN** the app is launched
- **THEN** the Today screen is shown with the spark input and the spark list visible

### Requirement: Capture-ready on launch
The spark input SHALL have keyboard focus when the app opens, so capturing a
spark is launch → type → Enter, with no click or navigation in between.

#### Scenario: Typing immediately after launch
- **WHEN** the user launches the app and starts typing
- **THEN** the typed text goes into the spark input without any prior interaction
