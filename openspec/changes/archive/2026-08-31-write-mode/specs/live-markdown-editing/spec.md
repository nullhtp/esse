# live-markdown-editing Delta Specification

## ADDED Requirements

### Requirement: Soft word wrap
The editor SHALL wrap lines at the viewport width; horizontal scrolling MUST
NOT exist. Wrapping is presentation only — it never inserts characters into
the document, and the text round-trips through save and clipboard with only
the newlines the user typed. Cursor up/down movement and mouse hit-testing
SHALL operate on visual (wrapped) lines. Marker visibility (raw markers on
the cursor's line) stays keyed to the source line, so a wrapped paragraph
reveals and hides its markers as one unit.

#### Scenario: A long paragraph wraps
- **WHEN** the user types a paragraph wider than the viewport
- **THEN** the text continues on the next visual line and no horizontal scrolling occurs

#### Scenario: Wrap leaves the text untouched
- **WHEN** a wrapped paragraph is saved or copied
- **THEN** the resulting text contains only the newlines the user typed

#### Scenario: Vertical movement follows visual lines
- **WHEN** the cursor is on the first visual line of a wrapped paragraph and the user presses down
- **THEN** the cursor moves to the next visual line of the same paragraph, keeping its horizontal position

#### Scenario: Click lands inside a wrapped line
- **WHEN** the user clicks a position on any visual line of a wrapped paragraph
- **THEN** the cursor is placed at the clicked character

## MODIFIED Requirements

### Requirement: Typewriter mode
The editor SHALL support a typewriter mode in which the visual (wrapped) line
containing the cursor stays vertically centred in the viewport.

#### Scenario: Cursor stays centred while typing
- **WHEN** typewriter mode is active and typing moves the cursor to a new visual line — by wrapping past the viewport width or by pressing Enter
- **THEN** the view scrolls so the cursor's visual line remains vertically centred
