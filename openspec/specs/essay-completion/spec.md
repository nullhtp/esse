# essay-completion Specification

## Purpose

How an essay ends: the publish flow — markdown copy, file export, an optional
publication link, `published_at` — and the deliberate shelve action. Both
transition the essay out of "in progress" through the essay-lifecycle rules and
free the single work-in-progress slot as a consequence.

Completion is offered from Edit mode only: deciding an essay is finished is a
judgement about the whole text, made in the room where the text is visible
whole. It answers beginner problem #5 ("ashamed to publish") by making
publishing the routine last step of the conveyor, and supports #4 ("gives up
after two weeks") by turning finished essays into a visible pile.

## Requirements

### Requirement: Completion is offered from Edit mode
Edit mode SHALL offer a quiet finish control that opens a completion overlay
with exactly two outcomes: Publish and Shelve. The control MUST NOT exist in
Write mode. Dismissing the overlay SHALL change nothing — the essay stays in
Editing and Edit mode stays on screen.

#### Scenario: Finish control opens the completion overlay
- **WHEN** the user activates the finish control in Edit mode
- **THEN** the completion overlay appears offering Publish and Shelve, and the text remains visible behind it

#### Scenario: Cancelling changes nothing
- **WHEN** the user dismisses the completion overlay without choosing an outcome
- **THEN** the essay remains in Editing state and Edit mode remains on screen unchanged

### Requirement: Publishing an essay
Confirming Publish SHALL save the essay, transition it to Published through
the essay-lifecycle rules, record `published_at`, and record
`publication_url` when the user provided a link — the link field is optional
and an empty value leaves the field absent. After a successful publish the
app SHALL return to the Today screen with the window state restored. If the
save or the transition fails, the app SHALL stay in Edit mode with the text
intact and surface the error quietly.

#### Scenario: Publish with a link
- **WHEN** the user enters a publication link and confirms Publish
- **THEN** the essay file records state Published, `published_at`, and the link in `publication_url`, and the Today screen is shown

#### Scenario: Publish without a link
- **WHEN** the user confirms Publish with the link field empty
- **THEN** the essay is Published with `published_at` recorded and no `publication_url` field, and the Today screen is shown

#### Scenario: A failed publish stays put
- **WHEN** the save or the transition to Published fails
- **THEN** Edit mode remains on screen with the text intact and the error shown quietly

### Requirement: Copy as markdown and export to file
The publish flow SHALL offer "Copy as markdown" and "Export to file…" before
the confirming action, so the text can be posted first and the resulting link
pasted back. Both SHALL carry the markdown body only — never the TOML front
matter. Copy places the body on the system clipboard; export opens the
native save dialog pre-filled with `<slug>.md` and writes the body to the
chosen path. Neither action changes the essay's state.

#### Scenario: Copy puts the body on the clipboard
- **WHEN** the user activates "Copy as markdown" in the publish flow
- **THEN** the clipboard contains exactly the essay body with no front matter, and the essay's state is unchanged

#### Scenario: Export writes the body to a chosen file
- **WHEN** the user activates "Export to file…" and chooses a location in the save dialog
- **THEN** a file containing exactly the essay body is written there, and the essay's state is unchanged

#### Scenario: Cancelled export writes nothing
- **WHEN** the user dismisses the save dialog without choosing a location
- **THEN** no file is written and the publish flow remains open

### Requirement: Shelving is deliberate
Choosing Shelve SHALL require one explicit confirmation step before acting.
On confirmation the essay is saved and transitioned to Shelved through the
essay-lifecycle rules, and the app returns to the Today screen with the
window state restored. Declining the confirmation SHALL return to the
completion overlay with nothing changed. A failed save or transition SHALL
keep Edit mode on screen with the error surfaced quietly.

#### Scenario: Confirmed shelve ends the essay
- **WHEN** the user chooses Shelve and confirms
- **THEN** the essay is saved, its state becomes Shelved, and the Today screen is shown

#### Scenario: Declined confirmation keeps the essay
- **WHEN** the user chooses Shelve and declines the confirmation
- **THEN** the essay remains in Editing state and the completion overlay is still available

### Requirement: Completion frees the WIP slot
Completing an essay — Published or Shelved — SHALL leave no essay in
progress, with no dedicated slot-freeing step: the slot is free because the
essay's state no longer counts as in progress under the essay-lifecycle
rules.

#### Scenario: Writing can start again after publishing
- **WHEN** the user publishes the in-progress essay and presses "Write" on the Today screen
- **THEN** the spark list is offered to start a new essay from

#### Scenario: Writing can start again after shelving
- **WHEN** the user shelves the in-progress essay and presses "Write" on the Today screen
- **THEN** the spark list is offered to start a new essay from
