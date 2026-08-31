# essay-completion Delta

## ADDED Requirements

### Requirement: The completion flow is keyboard-operable
Edit mode SHALL open the completion overlay with `cmd-enter`, the same
action as the finish control. Inside the overlay, `left`/`right` and `tab`
SHALL move a visible highlight between the stage's actions, `enter` SHALL
activate the highlighted action, and `escape` SHALL dismiss as it already
does. In the publishing stage, `tab` SHALL cycle through the link field and
the actions, so copy, export, entering the link, and confirming all work
without a pointer. When a stage opens, the confirming action MUST NOT be the
pre-highlighted target of a bare `enter` — publishing and shelving always
take at least one deliberate movement.

#### Scenario: The shortcut opens the overlay
- **WHEN** the user presses `cmd-enter` in Edit mode
- **THEN** the completion overlay appears exactly as if the finish control had been clicked

#### Scenario: Publishing without a pointer
- **WHEN** the user opens the overlay, moves the highlight to Publish, enters the publishing stage, tabs to the link field, types a link, tabs to the confirming action, and presses `enter`
- **THEN** the essay is published with the link recorded, exactly as the pointer flow would do it

#### Scenario: A stray enter cannot complete
- **WHEN** a completion stage has just opened and the user presses `enter` with no prior movement
- **THEN** the essay is not published and not shelved

#### Scenario: The shortcut does not exist in Write mode
- **WHEN** the user presses `cmd-enter` in Write mode
- **THEN** no completion overlay appears
