# published-row Specification

## Purpose

The row of published essays on the Today screen: the pile of finished work,
shown as cards on the screen the app opens into.

The Shelf has a Published column, but the Shelf has to be opened. Putting the
same essays on Today makes finished work the main progress display, which is
what answers beginner problem #4 ("gives up after two weeks") and reinforces
#5 ("ashamed to publish") — publishing is the thing the app visibly rewards.
It is a showcase and not a workspace: nothing in it starts, opens, or changes
an essay, and per the anti-features list nothing counts them.

## Requirements

### Requirement: Published essays as a row of cards
The Today screen SHALL show published essays as a row of cards, newest first
by `published_at`. Each card SHALL show the essay's title, derived by the
same rule the Shelf uses, and the publication link when one was recorded; a
card for an essay without a link shows no link element.

#### Scenario: The row lists published essays newest first
- **WHEN** the Today screen is shown and published essays exist
- **THEN** the row shows one card per published essay, ordered newest first by `published_at`

#### Scenario: A card carries its publication link
- **WHEN** a published essay has a recorded `publication_url`
- **THEN** its card shows that link, and cards of essays without a link show none

### Requirement: The row is a showcase, not a workspace
The published row SHALL offer no action that changes any essay's state and
SHALL NOT open the editor. It MUST show no counts or statistics, and it MUST
be visually subordinate to the "Write" button and the spark input — the
showcase never competes with starting to write.

#### Scenario: Cards change nothing
- **WHEN** the user interacts with a card in the published row
- **THEN** no essay changes state and the editor does not open

#### Scenario: The pile is not counted
- **WHEN** the Today screen is shown with published essays
- **THEN** the row displays the cards without a count, total, or any other statistic

### Requirement: Hidden before the first published essay
While no essay is Published, the Today screen SHALL show no published row and
no placeholder for it. The row first appears when the first essay is
published.

#### Scenario: No published essays means no row
- **WHEN** the Today screen is shown and no essay has been published
- **THEN** neither a row nor an empty-state placeholder appears

#### Scenario: The first publish reveals the row
- **WHEN** the user publishes the first essay and returns to the Today screen
- **THEN** the published row appears with that one card
