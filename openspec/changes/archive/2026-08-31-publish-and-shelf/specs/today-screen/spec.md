# today-screen Delta

## ADDED Requirements

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
