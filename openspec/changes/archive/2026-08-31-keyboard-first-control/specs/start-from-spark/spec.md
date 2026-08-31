# start-from-spark Delta

## ADDED Requirements

### Requirement: Choosing a spark works from the keyboard
When the Write action finds the slot free and sparks present, the choosing
state SHALL move keyboard focus from the capture line to the spark choice:
`up`/`down` move a visible highlight through the list starting at the newest
spark, `enter` starts the essay from the highlighted spark exactly as
clicking it does, and `escape` leaves the choosing state with nothing
started and focus returned to the capture line.

#### Scenario: Choosing entirely by keyboard
- **WHEN** the user presses the Write shortcut with the slot free, moves the highlight with the arrow keys, and presses `enter`
- **THEN** a Draft essay is created from the highlighted spark and opens in Write mode, exactly as if that spark had been clicked

#### Scenario: Escape cancels the choice
- **WHEN** the user presses `escape` in the choosing state
- **THEN** no essay is created, the Today screen returns to rest, and typing goes to the spark input again

#### Scenario: The highlight starts at the newest spark
- **WHEN** the choosing state opens
- **THEN** the newest spark is highlighted, so `enter` alone starts from the most recent idea
