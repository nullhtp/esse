# spark-capture Delta Specification

## MODIFIED Requirements

### Requirement: Spark list, newest first
The app SHALL show all captured sparks that have not yet been turned into an
essay, as a flat list ordered newest first, with no folders, tags, or
ordering options. A spark consumed by starting an essay leaves the list.

#### Scenario: New spark appears on top
- **WHEN** a spark is captured
- **THEN** it appears at the top of the spark list immediately

#### Scenario: List persists across restarts
- **WHEN** the app is closed and reopened
- **THEN** the spark list shows all previously captured sparks that were not consumed, newest first

#### Scenario: Consumed spark leaves the list
- **WHEN** an essay is started from a spark
- **THEN** that spark no longer appears in the spark list, in this run or after a restart
