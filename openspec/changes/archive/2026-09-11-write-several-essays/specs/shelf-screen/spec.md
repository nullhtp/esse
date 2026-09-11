# shelf-screen Specification

## Purpose

The third and final screen: the whole conveyor at a glance — Sparks, In
progress, Published, and a collapsed drawer of shelved essays — plus the
navigation between it and Today.

The Shelf is where the pile of finished work becomes visible, which is what
makes the conveyor feel like it is running (beginner problem #4, "gives up
after two weeks"). It is a plain window screen: fullscreen belongs to the
editor rooms. It reflects state rather than owning it — starting an essay from
a spark here goes through the same start-from-spark rules as the Today
screen's "Write" button, so the WIP limit has one enforcement point.

## MODIFIED Requirements

### Requirement: Three columns and a drawer
The Shelf screen SHALL show three columns — Sparks, In progress, Published —
and a collapsed drawer for shelved essays. Sparks list every uncaptured-into-
an-essay spark newest first; In progress holds the zero to three essays in
Draft or Editing state, most recently worked first, each card showing its
state; Published lists published essays newest first by `published_at`, each
showing its publication link when one was recorded. There MUST be no counts,
statistics, ordering options, folders, or tags on any column — the limit of
three is not displayed as a tally.

#### Scenario: The shelf reflects the stores
- **WHEN** the Shelf screen is opened
- **THEN** the Sparks column shows the spark box newest first, In progress shows every essay in progress most recently worked first, and Published shows published essays newest first

#### Scenario: Several cards in progress
- **WHEN** the Shelf screen is shown while three essays are in Draft or Editing state
- **THEN** the In progress column contains exactly those three essays, each showing whether it is a Draft or in Editing

#### Scenario: No tally of the limit
- **WHEN** the Shelf screen is shown with any number of essays in progress
- **THEN** no column shows a count, and nothing on the screen states how many essays remain startable

### Requirement: Starting and continuing from the Shelf
Choosing a spark in the Sparks column SHALL start an essay from it through the
start-from-spark rules, only while fewer than three essays are in progress;
once three are open, sparks SHALL NOT offer a start action and one quiet line
SHALL say that publishing or shelving an essay makes room. Activating a card in
the In progress column SHALL continue that essay exactly as choosing it in the
Today screen's chooser does: a Draft opens in Write mode, an essay in Editing
opens in Edit mode.

#### Scenario: Spark starts an essay while there is room
- **WHEN** the user chooses a spark on the Shelf with fewer than three essays in progress
- **THEN** a Draft essay is created from that spark and opens in Write mode

#### Scenario: Three open essays disable starting
- **WHEN** three essays are in progress and the user looks at the Sparks column
- **THEN** the sparks offer no start action and a line says that publishing or shelving an essay makes room

#### Scenario: Each in-progress card continues its own essay
- **WHEN** the user activates one of the cards in the In progress column
- **THEN** that essay — and not another — opens in the editor in the mode matching its state

### Requirement: The Shelf is keyboard-navigable
The Shelf SHALL hold a visible highlight over its cards: `left`/`right`
move between columns that have content, `up`/`down` move within a column —
including through several cards in the In progress column — and `enter`
activates the highlighted card exactly as a click does: a spark starts an essay
only while fewer than three essays are in progress, an in-progress card
continues that essay, and cards with no click action (published, shelved) do
nothing. `cmd-d` SHALL toggle the shelved drawer. The highlight starts on the
first actionable card when the Shelf opens.

#### Scenario: Starting from a spark by keyboard
- **WHEN** fewer than three essays are in progress and the user moves the highlight to a spark and presses `enter`
- **THEN** a Draft essay is created from that spark and opens in Write mode, through the same start-from-spark rules as a click

#### Scenario: Moving between the open essays
- **WHEN** the highlight is on the In progress column with several cards and the user presses `down`
- **THEN** the highlight moves to the next essay in that column

#### Scenario: Continuing a chosen essay by keyboard
- **WHEN** the user moves the highlight to an in-progress card and presses `enter`
- **THEN** that essay opens in the editor in the mode matching its state

#### Scenario: Enter respects a full conveyor
- **WHEN** three essays are in progress and the user presses `enter` on a highlighted spark
- **THEN** nothing is created, exactly as sparks offer no start action by pointer

#### Scenario: The drawer opens from the keyboard
- **WHEN** the user presses `cmd-d` on the Shelf
- **THEN** the shelved drawer expands, and pressing it again collapses the drawer
