# shelf-screen Delta

## ADDED Requirements

### Requirement: The Shelf is keyboard-navigable
The Shelf SHALL hold a visible highlight over its cards: `left`/`right`
move between columns that have content, `up`/`down` move within a column,
and `enter` activates the highlighted card exactly as a click does — a spark
starts an essay only while the slot is free, the in-progress card continues
the essay, and cards with no click action (published, shelved) do nothing.
`cmd-d` SHALL toggle the shelved drawer. The highlight starts on the first
actionable card when the Shelf opens.

#### Scenario: Starting from a spark by keyboard
- **WHEN** no essay is in progress and the user moves the highlight to a spark and presses `enter`
- **THEN** a Draft essay is created from that spark and opens in Write mode, through the same start-from-spark rules as a click

#### Scenario: Continuing the essay by keyboard
- **WHEN** the user moves the highlight to the In progress card and presses `enter`
- **THEN** the essay opens in the editor in the mode matching its state

#### Scenario: Enter respects the occupied slot
- **WHEN** an essay is in progress and the user presses `enter` on a highlighted spark
- **THEN** nothing happens, exactly as sparks offer no start action by pointer

#### Scenario: The drawer opens from the keyboard
- **WHEN** the user presses `cmd-d` on the Shelf
- **THEN** the shelved drawer expands, and pressing it again collapses the drawer
