# writing-sessions Specification

## Purpose

The writing session as the unit of habit: a timer that starts with Write mode,
a soft finish when the target is reached, and a record appended when the
session ends.

This is the answer to beginner problem #4 ("gives up after two weeks"): the
session is the thing that succeeds, however the essay itself went. The target
is a fixed constant rather than a setting, and per the anti-features list
nothing counts, displays, or streaks what is recorded — the dotted calendar
that reads the history comes later.

## Requirements

### Requirement: A session starts with Write mode
Entering Write mode SHALL start a writing session. A quiet elapsed-time
indicator sits in a corner of the screen — small, dim, ignorable. There are
no other counters, word counts, or statistics anywhere.

#### Scenario: Timer runs quietly in the corner
- **WHEN** the user enters Write mode
- **THEN** a session begins and the corner indicator shows elapsed time unobtrusively

### Requirement: Soft finish at the session target
When the session reaches its target length (20 minutes, a fixed constant —
not a setting), the indicator SHALL change state gently to say the session is
complete. There MUST be no modal, no sound, and no interruption of typing;
writing past the target is allowed indefinitely.

#### Scenario: Target reached mid-typing
- **WHEN** the session reaches the target while the user is typing
- **THEN** the corner indicator changes to its "session complete" state and typing continues uninterrupted

### Requirement: Sessions are recorded
Leaving Write mode SHALL append a session record — `essay_slug`,
`started_at`, `duration_min` — to the session history. Sessions shorter than
one minute SHALL NOT be recorded. Nothing displays the history yet; the
dotted calendar reads it in a later stage.

#### Scenario: A finished session lands in the history
- **WHEN** the user leaves Write mode after writing for at least a minute
- **THEN** one session record with the essay's slug, the start time, and the duration in minutes is appended to the session history

#### Scenario: An accidental peek is not a session
- **WHEN** the user leaves Write mode less than a minute after entering
- **THEN** no session record is written
