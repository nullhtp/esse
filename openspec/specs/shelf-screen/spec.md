# shelf-screen Specification

## Purpose

The third and final screen: the whole conveyor at a glance — Sparks, In
progress, Published, and a collapsed drawer of shelved essays — plus the
navigation between it and Today.

The Shelf is where the pile of finished work becomes visible, which is what
makes the conveyor feel like it is running (beginner problem #4, "gives up
after two weeks"). It is a plain window screen: fullscreen belongs to the
editor rooms. It reflects state rather than owning it — starting an essay from
a spark here goes through the same start-from-spark rules as the Today screen's
"Write" button, so the WIP = 1 invariant has one enforcement point.

## Requirements

### Requirement: Three columns and a drawer
The Shelf screen SHALL show three columns — Sparks, In progress, Published —
and a collapsed drawer for shelved essays. Sparks list every uncaptured-into-
an-essay spark newest first; In progress holds the zero or one essay in Draft
or Editing state; Published lists published essays newest first by
`published_at`, each showing its publication link when one was recorded.
There MUST be no counts, statistics, ordering options, folders, or tags on
any column.

#### Scenario: The shelf reflects the stores
- **WHEN** the Shelf screen is opened
- **THEN** the Sparks column shows the spark box newest first, In progress shows the in-progress essay if one exists, and Published shows published essays newest first

#### Scenario: At most one card in progress
- **WHEN** the Shelf screen is shown while an essay is in Draft or Editing state
- **THEN** the In progress column contains exactly that one essay

### Requirement: Shelved drawer is collapsed by default
The shelved drawer SHALL be collapsed when the Shelf opens and SHALL expand
only on an explicit action, listing shelved essays for looking at. No action
on a shelved essay SHALL change its state — the lifecycle forbids leaving
Shelved.

#### Scenario: Drawer opens on request
- **WHEN** the user expands the shelved drawer
- **THEN** the shelved essays are listed, and collapsing it hides them again

#### Scenario: Shelved essays cannot be revived
- **WHEN** the user interacts with a shelved essay in the drawer
- **THEN** no action is offered that would move it out of the Shelved state

### Requirement: Navigation between Today and the Shelf
The Today screen SHALL offer a quiet affordance opening the Shelf, and the
Shelf SHALL return to Today via an equivalent affordance or the Escape key.
The Shelf is a plain window screen — entering and leaving it MUST NOT change
the window's fullscreen state.

#### Scenario: Round trip Today → Shelf → Today
- **WHEN** the user opens the Shelf from Today and then presses Escape
- **THEN** the Shelf is shown and then Today again, with the window state unchanged throughout

### Requirement: Starting and continuing from the Shelf
Choosing a spark in the Sparks column SHALL start an essay from it through
the start-from-spark rules, only when the WIP slot is free; while the slot is
occupied, sparks SHALL NOT offer a start action. Activating the In progress
card SHALL continue the essay exactly as the Today screen's "Write" button
does: a Draft opens in Write mode, an essay in Editing opens in Edit mode.

#### Scenario: Spark starts an essay when the slot is free
- **WHEN** the user chooses a spark on the Shelf with no essay in progress
- **THEN** a Draft essay is created from that spark and opens in Write mode

#### Scenario: Occupied slot disables starting
- **WHEN** an essay is in progress and the user looks at the Sparks column
- **THEN** the sparks offer no start action

#### Scenario: The in-progress card continues the essay
- **WHEN** the user activates the In progress card
- **THEN** the essay opens in the editor in the mode matching its state

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

