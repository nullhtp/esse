# today-screen Delta

## MODIFIED Requirements

### Requirement: Today is the start screen
The app SHALL open directly into the Today screen in a single window. The
screen contains the "Write" button, the spark input, the spark list, the row
of published essays, and the dotted session calendar. The published row and
the session calendar MUST stay visually subordinate to the "Write" button and
the spark input; their presence rules are defined by the published-row and
session-calendar capabilities.

#### Scenario: Launch lands on Today
- **WHEN** the app is launched
- **THEN** the Today screen is shown with the "Write" button, the spark input, and the spark list visible, plus the published row and the session calendar when they have data to show

#### Scenario: The showcase does not outshine the launchpad
- **WHEN** the Today screen is shown with published essays and session history present
- **THEN** the "Write" button remains the most prominent element and the spark input still holds keyboard focus on launch
