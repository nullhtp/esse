# spark-capture Delta

## ADDED Requirements

### Requirement: One-line spark capture
The app SHALL capture a spark as a single line of text: submitting a
non-empty line persists it with a timestamp and clears the input. Whitespace-
only input SHALL be ignored. Cyrillic and IME composition MUST work in the
input.

#### Scenario: Spark is captured
- **WHEN** the user types a non-empty line into the spark input and presses Enter
- **THEN** the spark is persisted with the current timestamp and the input is cleared

#### Scenario: Empty input is ignored
- **WHEN** the user presses Enter with an empty or whitespace-only input
- **THEN** nothing is persisted and the input state is unchanged

### Requirement: Spark list, newest first
The app SHALL show all captured sparks as a flat list ordered newest first,
with no folders, tags, or ordering options.

#### Scenario: New spark appears on top
- **WHEN** a spark is captured
- **THEN** it appears at the top of the spark list immediately

#### Scenario: List persists across restarts
- **WHEN** the app is closed and reopened
- **THEN** the spark list shows all previously captured sparks, newest first
