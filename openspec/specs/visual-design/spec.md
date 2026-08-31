# visual-design Specification

## Purpose

The appearance rules the whole app is held to — the face it is set in, one
type scale and one spacing rhythm, a complete palette for each room, a single
accent, and the register reserved for naming a place.

esse has almost no chrome, so its screens are nearly all text and the type is
the interface. These are also the decisions that erode quietly: they are
written down so that no later change can borrow Today's ink for an Edit-mode
panel, or invent a seventh text size, without noticing.

## Requirements

### Requirement: The app ships the face it is set in
The app SHALL be set in one embedded serif family, registered at startup and
used by every screen, sheet and editor line. The family MUST cover both Latin
and Cyrillic, because essays are written in either. The app MUST NOT depend on
any face being installed on the machine, and MUST NOT offer a font setting.

If the faces fail to load, the app SHALL still start, set in the system face,
and record the failure in the log rather than refusing to run.

#### Scenario: Every weight the app asks for has its own face
- **WHEN** the app asks the font system for the family at body weight, at heading weight, and at bold weight, upright and italic
- **THEN** each request resolves to a distinct embedded face rather than to one face emboldened or slanted by the rasterizer

#### Scenario: Russian and English set in the same face
- **WHEN** an essay containing both Cyrillic and Latin text is rendered
- **THEN** all of it is drawn in the app's own family, with no fallback to a system face mid-paragraph

#### Scenario: The faces are missing
- **WHEN** the embedded faces cannot be registered at startup
- **THEN** the app opens anyway in the system face and the failure is logged

### Requirement: One type scale and one spacing rhythm
Every text size and every gap in the app SHALL come from the scale and the
rhythm defined in the theme. A screen MUST NOT introduce a size or a gap of its
own.

Hierarchy SHALL be carried by size, weight and space. Weight is used at three
steps: body text, headings and labels one step above it, and `**bold**` one
step above that — so a bold word inside a heading is still distinguishable.

#### Scenario: A heading and a bold word are different weights
- **WHEN** a heading containing a `**bold**` word is rendered
- **THEN** the bold word is heavier than the heading around it

### Requirement: Each room has a complete palette of its own
Today and the Shelf, Write mode, and Edit mode SHALL each have a complete set
of colours — background, ink, muted ink, rules, and any control faces they
draw. A room MUST NOT borrow a colour from another room, including for a panel
or overlay that it draws over its own background.

#### Scenario: An overlay belongs to the room it is drawn in
- **WHEN** the completion panel is shown over Edit mode
- **THEN** every colour in it comes from Edit mode's palette

#### Scenario: The rooms stay far apart
- **WHEN** the writer switches between Write mode and Edit mode
- **THEN** the change of room is unmistakable at a glance, without reading anything on the screen

### Requirement: One accent, and it marks the writing
The app SHALL use exactly one saturated accent colour, and it SHALL be reserved
for the caret and the text selection. Every other element is drawn in ink, in
paper, or in a step between them.

#### Scenario: Nothing else is coloured
- **WHEN** any screen is shown
- **THEN** the only saturated colour on it is the caret, or the alarm colour when there is trouble to report

### Requirement: Naming a place is a register of its own
Controls and headings that name a place rather than say something — the corner
controls, the Shelf's column headings, the sheets' section headings — SHALL be
drawn as small letterspaced capitals, distinct from the register used for
sentences.

#### Scenario: A corner control does not read as a sentence
- **WHEN** a screen with a corner control is shown
- **THEN** the control is visibly a label rather than a line of prose, and stays subordinate to the screen's own text
