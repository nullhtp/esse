# start-from-spark Specification

## Purpose

How writing begins: the "Write" button's routing, turning a chosen spark into
a Draft essay, and the slug derived from the spark's text.

This is the answer to beginner problem #2 ("cannot start"): the empty-page
screen does not exist, because no route reaches the editor without an essay
already loaded. Consuming the spark is what joins the spark box to the essay
pipeline, and the storage layer's WIP = 1 refusal is surfaced here rather than
worked around.

## Requirements

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

### Requirement: Starting an essay consumes the spark
Choosing a spark SHALL create the essay in Draft with the spark's text
recorded in the essay's front matter, and then remove the spark from the
spark box. Creation happens before removal, so a failure at any step MUST
never lose the spark.

#### Scenario: Spark becomes a draft
- **WHEN** the user chooses a spark to start from
- **THEN** a Draft essay exists carrying the spark's text as its seed, and the spark is no longer in the box

#### Scenario: Failure keeps the spark
- **WHEN** essay creation fails for any reason
- **THEN** the spark remains in the box unchanged

### Requirement: Slug derived from the spark text
The new essay's slug SHALL be derived from the spark text: Cyrillic
transliterated, lowercased, non-alphanumerics collapsed to hyphens, truncated
at a word boundary. When nothing usable survives, the slug falls back to
`essay-<date>`. When the derived slug is taken, a numeric suffix (`-2`,
`-3`, …) SHALL be appended until it is free.

#### Scenario: Cyrillic spark yields a readable slug
- **WHEN** an essay starts from the spark «почему эссе»
- **THEN** its file is created under `essays/` with a transliterated ASCII slug such as `pochemu-esse`

#### Scenario: Colliding slug gets a suffix
- **WHEN** an essay starts from a spark whose derived slug already names an existing essay file
- **THEN** the essay is created with the first free suffixed slug

### Requirement: WIP refusal is surfaced, not swallowed
If essay creation is ever attempted while the slot is occupied, the storage
layer's refusal SHALL be shown to the user, identifying the in-progress
essay. Routing normally prevents the attempt; this is the defensive path, and
it MUST NOT be bypassed in the UI.

#### Scenario: Refusal names the open essay
- **WHEN** essay creation is refused because an essay is in progress
- **THEN** the user sees which essay occupies the slot and nothing is created
