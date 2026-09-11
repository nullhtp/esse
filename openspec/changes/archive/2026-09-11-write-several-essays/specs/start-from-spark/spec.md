# start-from-spark Specification

## Purpose

How writing begins: the "Write" button's chooser, turning a chosen spark into
a Draft essay, carrying on with an essay already in progress, and the slug
derived from the spark's text.

This is the answer to beginner problem #2 ("cannot start"): the empty-page
screen does not exist, because no route reaches the editor without an essay
already loaded, and the chooser always has something warm in it. Consuming the
spark is what joins the spark box to the essay pipeline, and the storage
layer's refusal at the WIP limit is surfaced here rather than worked around.

## MODIFIED Requirements

### Requirement: Writing starts only through the Write button
The Today screen's "Write" button SHALL be the only way into the editor, and
it SHALL always open the work chooser rather than routing straight into the
editor. The chooser offers the essays in progress first — most recently worked
first, each showing whether it is a Draft or in Editing — and the sparks below
them, newest first. Choosing an essay continues it in the mode matching its
state: a Draft opens in Write mode, an essay in Editing opens in Edit mode.
Choosing a spark creates a new essay that always opens in Write mode. When
nothing is in progress and the spark box is empty, no editor opens and the app
says a spark is needed to start. The app MUST have no route that opens the
editor without an essay and no "create empty document" action anywhere.

#### Scenario: The chooser offers the open essays first
- **WHEN** the user presses "Write" with essays in progress and sparks in the box
- **THEN** the chooser lists those essays above the sparks, most recently worked first, each showing its state

#### Scenario: Continue a Draft in Write mode
- **WHEN** the user chooses an essay in Draft state from the chooser
- **THEN** that essay opens in Write mode with its existing text

#### Scenario: Continue an Editing essay in Edit mode
- **WHEN** the user chooses an essay in Editing state from the chooser
- **THEN** that essay opens in Edit mode with its existing text

#### Scenario: Nothing open — the chooser is the spark list
- **WHEN** the user presses "Write" with no essay in progress and at least one spark in the box
- **THEN** the chooser offers the sparks alone, and choosing one opens the new draft in Write mode

#### Scenario: No sparks, no essay — writing needs a spark first
- **WHEN** the user presses "Write" with no essay in progress and an empty spark box
- **THEN** no editor opens and the app says a spark is needed to start

### Requirement: WIP refusal is surfaced, not swallowed
If essay creation is ever attempted while three essays are in progress, the
storage layer's refusal SHALL be shown to the user, saying that three essays
are open and that publishing or shelving one makes room. Routing normally
prevents the attempt; this is the defensive path, and it MUST NOT be bypassed
in the UI.

#### Scenario: The refusal names the limit and the way out
- **WHEN** essay creation is refused because three essays are in progress
- **THEN** the user is told that three essays are open and that publishing or shelving one makes room, and nothing is created

#### Scenario: The spark survives a refused start
- **WHEN** a start is refused because three essays are in progress
- **THEN** the spark remains in the box unchanged

### Requirement: Sparks stop offering a start when three essays are open
While three essays are in progress, the chooser SHALL show the sparks without a
start action and SHALL say in one quiet line why they cannot be started. The
essays in progress SHALL still be choosable, so the chooser never becomes a
dead end.

#### Scenario: A full conveyor disables starting but not continuing
- **WHEN** the user presses "Write" with three essays in progress
- **THEN** the three essays are choosable, the sparks offer no start action, and one line says that publishing or shelving an essay makes room

#### Scenario: Room returns when an essay ends
- **WHEN** one of three open essays is published or shelved and the user presses "Write"
- **THEN** the sparks offer a start action again and the line is gone

## REMOVED Requirements

### Requirement: Choosing a spark works from the keyboard

**Reason**: The choosing state now offers essays as well as sparks, so a
requirement named for sparks alone no longer describes it.

**Migration**: Replaced by "Choosing what to work on works from the keyboard"
below. The keys are unchanged — `up`/`down`, `enter`, `escape` — and so is the
rule that the capture line gives up focus for as long as the choice is open.

## ADDED Requirements

### Requirement: Choosing what to work on works from the keyboard
When the Write action opens the chooser, the choosing state SHALL move keyboard
focus from the capture line to the chooser: `up`/`down` move a visible
highlight through the whole list, crossing between the essays and the sparks
without a separate key, `enter` activates the highlighted entry exactly as
clicking it does, and `escape` leaves the choosing state with nothing started
or opened and focus returned to the capture line. The highlight SHALL start on
the first entry — the most recently worked essay when any essay is in progress,
otherwise the newest spark — so `enter` alone carries on with the last thing
written, or starts from the most recent idea.

#### Scenario: Continuing entirely by keyboard
- **WHEN** the user presses the Write shortcut, moves the highlight with the arrow keys to an essay in progress, and presses `enter`
- **THEN** that essay opens in the mode matching its state, exactly as if the entry had been clicked

#### Scenario: Starting entirely by keyboard
- **WHEN** the user presses the Write shortcut, moves the highlight down to a spark, and presses `enter`
- **THEN** a Draft essay is created from that spark and opens in Write mode, exactly as if the spark had been clicked

#### Scenario: Enter alone continues the last essay
- **WHEN** the chooser opens with at least one essay in progress
- **THEN** the most recently worked essay is highlighted, so `enter` alone reopens it

#### Scenario: Enter alone starts from the newest spark
- **WHEN** the chooser opens with no essay in progress and sparks in the box
- **THEN** the newest spark is highlighted, so `enter` alone starts from the most recent idea

#### Scenario: The highlight crosses between the sections
- **WHEN** the highlight is on the last essay in progress and the user presses `down`
- **THEN** the highlight moves to the newest spark

#### Scenario: Escape cancels the choice
- **WHEN** the user presses `escape` in the choosing state
- **THEN** no essay is created or opened, the Today screen returns to rest, and typing goes to the spark input again
