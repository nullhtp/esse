# essay-completion Specification

## Purpose

How an essay ends: the publish flow — markdown copy, file export, an optional
publication link, `published_at` — and the deliberate shelve action. Both
transition the essay out of "in progress" through the essay-lifecycle rules
and make room for another essay as a consequence, leaving the others open.

Completion is offered from Edit mode only: deciding an essay is finished is a
judgement about the whole text, made in the room where the text is visible
whole. It answers beginner problem #5 ("ashamed to publish") by making
publishing the routine last step of the conveyor, and supports #4 ("gives up
after two weeks") by turning finished essays into a visible pile.

## REMOVED Requirements

### Requirement: Completion frees the WIP slot

**Reason**: There is no single slot to free — completing an essay makes room
for one more while the other open essays carry on.

**Migration**: Replaced by "Completion makes room for another essay" below.
Publishing and shelving are unchanged; only what they leave behind is restated.

## ADDED Requirements

### Requirement: Completion makes room for another essay
Completing an essay — Published or Shelved — SHALL leave that essay out of the
essays in progress and SHALL leave every other open essay untouched, with no
dedicated room-making step: room exists because the completed essay's state no
longer counts as in progress under the essay-lifecycle rules.

#### Scenario: The other open essays survive a publish
- **WHEN** the user publishes one of three essays in progress
- **THEN** the other two remain in progress with their text and states unchanged

#### Scenario: Starting becomes possible again after publishing
- **WHEN** the user publishes an essay while three were in progress and then presses "Write"
- **THEN** the chooser offers the two remaining essays and the sparks with a start action

#### Scenario: Starting becomes possible again after shelving
- **WHEN** the user shelves an essay while three were in progress and then presses "Write"
- **THEN** the chooser offers the two remaining essays and the sparks with a start action

## MODIFIED Requirements

### Requirement: Completion is offered from Edit mode
Edit mode SHALL offer a quiet finish control that opens a completion overlay
with exactly two outcomes: Publish and Shelve. The control MUST NOT exist in
Write mode. The overlay SHALL act on the essay open in that room and on no
other, whatever else is in progress. Dismissing the overlay SHALL change
nothing — the essay stays in Editing and Edit mode stays on screen.

#### Scenario: Finish control opens the completion overlay
- **WHEN** the user activates the finish control in Edit mode
- **THEN** the completion overlay appears offering Publish and Shelve, and the text remains visible behind it

#### Scenario: The overlay names the essay it will end
- **WHEN** the user opens the completion overlay while other essays are in progress
- **THEN** the outcome applies to the essay open in the room, and the other essays in progress are not offered and not changed

#### Scenario: Cancelling changes nothing
- **WHEN** the user dismisses the completion overlay without choosing an outcome
- **THEN** the essay remains in Editing state and Edit mode remains on screen unchanged
