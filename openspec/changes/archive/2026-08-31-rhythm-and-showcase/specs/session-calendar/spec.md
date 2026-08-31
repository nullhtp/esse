# session-calendar Delta

## ADDED Requirements

### Requirement: The last four weeks as a dotted line
The Today screen SHALL show the session history as a single horizontal row of
28 dots — one per calendar day, the last 28 days, oldest on the left and today
rightmost. A dot SHALL be filled when at least one session record's
`started_at` falls on that local calendar day, and empty otherwise. The depth
of 28 days is a fixed constant, not a setting.

#### Scenario: A day with a session gets a filled dot
- **WHEN** the Today screen is shown and a session was recorded earlier on some day within the last 28 days
- **THEN** the dot for that day is filled

#### Scenario: A day without sessions stays an empty dot
- **WHEN** the Today screen is shown and some day within the last 28 days has no session records
- **THEN** the dot for that day is empty, with no other marking

#### Scenario: A session crossing midnight marks its start day
- **WHEN** a session started at 23:50 local time and ran past midnight
- **THEN** only the dot of the day it started on is filled by that session

### Requirement: The calendar shows presence, never quantity
The dotted calendar SHALL distinguish only "wrote" from "did not write". It
MUST NOT show counts, durations, streak lengths, numbers, weekday or date
labels, or tooltips, and a day with several sessions MUST look identical to a
day with one.

#### Scenario: Two sessions on one day look like one
- **WHEN** two sessions were recorded on the same day
- **THEN** that day's dot is identical to the dot of a day with a single session

#### Scenario: Nothing counts the dots
- **WHEN** the Today screen is shown with any session history
- **THEN** no number, streak indicator, or label accompanies the dotted row

### Requirement: The calendar is inert and quiet
The dotted calendar SHALL be display-only — activating or hovering it does
nothing and navigates nowhere — and visually subordinate to the "Write"
button, the spark input, and the spark list. When no session falls within the
last 28 days the calendar SHALL NOT be shown at all, with no placeholder in
its place.

#### Scenario: Dots respond to nothing
- **WHEN** the user clicks or hovers the dotted calendar
- **THEN** nothing changes and nothing opens

#### Scenario: An empty month means no calendar
- **WHEN** the Today screen is shown and no session was recorded in the last 28 days
- **THEN** no dotted row and no placeholder appear
