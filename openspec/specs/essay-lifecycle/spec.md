# essay-lifecycle Specification

## Purpose

The essay pipeline's state machine: the four states an essay can be in, the
transitions allowed between them, and the hard WIP = 1 invariant — at most one
essay in progress at any time.

The invariant is enforced in the storage layer rather than in the UI, so no
screen or future API can grow around a looser rule. It backs beginner problems
#3 ("drafting and editing get mixed") and #4 ("gives up after two weeks").

## Requirements

### Requirement: Essay states
An essay SHALL be in exactly one of four states: Draft, Editing, Published,
or Shelved. An essay in Draft or Editing counts as in progress.

#### Scenario: New essay starts as Draft
- **WHEN** an essay is created from a spark
- **THEN** its state is Draft

### Requirement: Legal state transitions
The system SHALL permit only these transitions: Draft ↔ Editing,
Draft → Published, Editing → Published, Draft → Shelved, Editing → Shelved.
Any other transition MUST be rejected.

#### Scenario: Draft moves to Editing and back
- **WHEN** an in-progress essay is switched between Draft and Editing
- **THEN** the transition succeeds in both directions

#### Scenario: Completed essay cannot re-enter the pipeline
- **WHEN** a transition is requested out of Published or Shelved
- **THEN** the transition is rejected

### Requirement: WIP limit of one enforced by the storage layer
The storage layer SHALL refuse to create a new essay while any essay is in
progress (Draft or Editing). No API that creates essays may bypass this check.

#### Scenario: Second essay refused while slot is occupied
- **WHEN** essay creation is requested and an essay in Draft or Editing state exists
- **THEN** the store returns an error identifying the in-progress essay and creates nothing

#### Scenario: Slot free after completion
- **WHEN** every existing essay is Published or Shelved and essay creation is requested
- **THEN** the store creates the essay in Draft state
