# first-run-onboarding — Delta

## ADDED Requirements

### Requirement: An empty app asks for sparks
When the Today screen opens and there are no sparks and no essays in any
state, it SHALL show a quiet inline prompt inviting the user to record 3–5
sparks. The prompt MUST sit with the spark input, MUST NOT be a modal,
wizard, or separate screen, and MUST NOT take keyboard focus away from the
spark input or displace the Write button's visual primacy.

#### Scenario: First launch shows the ask
- **WHEN** the app opens with no sparks and no essays in any state
- **THEN** the Today screen shows the prompt inviting 3–5 sparks, and the spark input holds keyboard focus

#### Scenario: The ask is not a barrier
- **WHEN** the prompt is visible and the user starts typing
- **THEN** the typed text goes straight into the spark input, with no interaction with the prompt required

### Requirement: The ask recedes as the box fills
The prompt SHALL remain while fewer than three sparks exist and no essay
exists in any state, and SHALL NOT be shown once at least three sparks exist
or at least one essay exists in any state. The condition MUST be computed
from stored data alone — no onboarding flag is persisted.

#### Scenario: The first spark does not dismiss the ask
- **WHEN** the user captures a first spark and no essay exists
- **THEN** the prompt is still shown, since the box has fewer than three sparks

#### Scenario: Three sparks complete the onboarding
- **WHEN** the third spark is captured
- **THEN** the prompt is no longer shown

#### Scenario: Starting to write completes the onboarding
- **WHEN** an essay exists in any state
- **THEN** the prompt is not shown regardless of how many sparks exist

### Requirement: This is the only onboarding
The 3–5 sparks prompt SHALL be the entirety of first-run onboarding: the app
MUST NOT show tours, tooltip sequences, sample content, or setup steps on
first launch.

#### Scenario: First launch is the ordinary Today
- **WHEN** the app is launched for the first time
- **THEN** apart from the sparks prompt, the Today screen looks and behaves exactly as on any later launch
