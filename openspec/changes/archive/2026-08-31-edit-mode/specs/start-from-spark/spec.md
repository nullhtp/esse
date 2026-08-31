# start-from-spark Delta

## MODIFIED Requirements

### Requirement: Writing starts only through the Write button
The Today screen's "Write" button SHALL be the only way into the editor.
When an essay is in progress, the button continues it in the mode matching
its state: a Draft opens in Write mode, an essay in Editing opens in Edit
mode. When the slot is free, the button offers the spark list to start from,
and a new essay always opens in Write mode. The app MUST have no route that
opens the editor without an essay and no "create empty document" action
anywhere.

#### Scenario: Continue a Draft in Write mode
- **WHEN** the user presses "Write" while the in-progress essay is in Draft state
- **THEN** that essay opens in Write mode with its existing text

#### Scenario: Continue an Editing essay in Edit mode
- **WHEN** the user presses "Write" while the in-progress essay is in Editing state
- **THEN** that essay opens in Edit mode with its existing text

#### Scenario: Free slot offers a spark to start from
- **WHEN** the user presses "Write" with no essay in progress and at least one spark in the box
- **THEN** the spark list is offered for choosing, and choosing a spark opens the new draft in Write mode

#### Scenario: No sparks, no essay — writing needs a spark first
- **WHEN** the user presses "Write" with no essay in progress and an empty spark box
- **THEN** no editor opens and the app says a spark is needed to start
