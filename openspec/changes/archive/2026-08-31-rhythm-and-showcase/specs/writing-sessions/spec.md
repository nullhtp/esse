# writing-sessions Delta

## MODIFIED Requirements

### Requirement: Sessions are recorded
Leaving Write mode SHALL append a session record — `essay_slug`,
`started_at`, `duration_min` — to the session history. Sessions shorter than
one minute SHALL NOT be recorded. The dotted session calendar on the Today
screen reads this history; nothing else displays, counts, or aggregates it.

#### Scenario: A finished session lands in the history
- **WHEN** the user leaves Write mode after writing for at least a minute
- **THEN** one session record with the essay's slug, the start time, and the duration in minutes is appended to the session history

#### Scenario: An accidental peek is not a session
- **WHEN** the user leaves Write mode less than a minute after entering
- **THEN** no session record is written

#### Scenario: A recorded session shows up on Today
- **WHEN** the user finishes a session and returns to the Today screen
- **THEN** the dot for today in the session calendar is filled
