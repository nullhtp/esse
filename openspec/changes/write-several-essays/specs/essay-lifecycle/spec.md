# essay-lifecycle Specification

## Purpose

The essay pipeline's state machine: the four states an essay can be in, the
transitions allowed between them, and the hard WIP invariant — at most three
essays in progress at any time.

The invariant is enforced in the storage layer rather than in the UI, so no
screen or future API can grow around a looser rule, and the limit is compiled
in rather than configurable. It backs beginner problems #3 ("drafting and
editing get mixed") and #4 ("gives up after two weeks"): three lanes let a
cold essay wait instead of being buried, and the wall at the fourth is what
keeps the waiting from becoming a graveyard.

## REMOVED Requirements

### Requirement: WIP limit of one enforced by the storage layer

**Reason**: The limit rises from one to three. The requirement is replaced
rather than edited because its name states the number.

**Migration**: Replaced by "WIP limit of three enforced by the storage layer"
below. The enforcement point is unchanged — `EssayStore::create` still refuses,
and no API that creates essays may bypass it — only the count it refuses at
changes. No stored data carries the old limit, so nothing on disk migrates.

## ADDED Requirements

### Requirement: WIP limit of three enforced by the storage layer
The storage layer SHALL refuse to create a new essay while three essays are in
progress (Draft or Editing). Fewer than three in progress SHALL leave creation
free. No API that creates essays may bypass this check, and the limit MUST NOT
be configurable at runtime.

#### Scenario: Three essays can be in progress at once
- **WHEN** essays are created one after another with each left in Draft or Editing
- **THEN** the first three are created and all three are reported as in progress

#### Scenario: A fourth essay is refused
- **WHEN** essay creation is requested and three essays are already in Draft or Editing state
- **THEN** the store returns an error naming the limit and creates nothing

#### Scenario: Room again after one essay completes
- **WHEN** three essays are in progress, one of them is published or shelved, and essay creation is requested
- **THEN** the store creates the essay in Draft state

#### Scenario: Completed essays never count against the limit
- **WHEN** essay creation is requested with any number of Published or Shelved essays and fewer than three in progress
- **THEN** the store creates the essay in Draft state

### Requirement: The essays in progress are reported as an ordered list
The storage layer SHALL report the essays in progress as a list ordered by when
each was last worked on, most recent first. The list SHALL be empty when no
essay is in progress and SHALL never hold more than three essays.

#### Scenario: Most recently worked essay comes first
- **WHEN** several essays are in progress and one of them is saved
- **THEN** that essay is first in the reported list

#### Scenario: An empty conveyor reports an empty list
- **WHEN** every essay is Published or Shelved
- **THEN** the reported list of essays in progress is empty
