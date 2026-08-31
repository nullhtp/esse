# today-screen — Delta

## MODIFIED Requirements

### Requirement: Capture-ready on launch
The spark input SHALL have keyboard focus when the app opens, so capturing a
spark is launch → type → Enter, with no click or navigation in between.
Launch itself SHALL be fast enough that opening the app fits inside the
five-second capture promise: on the target machine, the Today screen is on
screen with the spark input focused within one second of app start.

#### Scenario: Typing immediately after launch
- **WHEN** the user launches the app and starts typing
- **THEN** the typed text goes into the spark input without any prior interaction

#### Scenario: Launch is not the slow part
- **WHEN** the app is cold-started on the target machine
- **THEN** the Today screen is visible with the spark input focused within one second of process start
