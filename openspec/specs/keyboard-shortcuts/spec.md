# keyboard-shortcuts Specification

## Purpose

Keyboard access to every operation the app offers — capture, write, choose,
browse, mark up, finish — with no persistent discoverability UI.

It answers beginner problem #2 ("cannot start") from the other side: the cost
of beginning a session drops to a keystroke, since nothing in the conveyor
requires hunting for a control with the pointer. It also protects flow
(problem #3), because reaching for the mouse mid-paragraph is what mixes
drafting with editing. The bindings stay chrome rather than a feature — per
the anti-features list there is no hint layered onto any screen and no
customization, and the single place they are listed is the on-demand overlay
the shortcut-help capability defines. On the resting Today screen every one of
them carries a modifier, so plain typing always lands in the spark input.

## Requirements

### Requirement: Shortcuts yield to typing
Every shortcut available on the resting Today screen MUST use a modifier
key, so that plain typing — including Cyrillic and IME composition — always
lands in the spark input and never triggers navigation. Transient states
entered by an explicit action and left by a single key (choosing a spark,
the help overlay) MAY bind unmodified keys while they are open, because the
capture line does not hold focus there.

#### Scenario: Plain typing is capture, not navigation
- **WHEN** the user types unmodified characters on the resting Today screen
- **THEN** the characters go into the spark input and no screen change or action is triggered

#### Scenario: A transient state hands the keys back
- **WHEN** the user leaves the spark-choosing state with Escape
- **THEN** focus returns to the spark input and plain typing is capture again

### Requirement: Shortcuts are chrome, not a feature
Shortcuts SHALL have no persistent in-app surface: no shortcut hints layered
onto any screen and no customization UI, ever. The single discoverability
surface is the on-demand help overlay defined by the shortcut-help
capability, visible only while summoned.

#### Scenario: No shortcut UI anywhere
- **WHEN** the user visits the Today, editor, and Shelf screens without summoning help
- **THEN** no shortcut listing, hint overlay, or customization control is present

#### Scenario: Discoverability exists only on demand
- **WHEN** the user summons the help overlay and dismisses it
- **THEN** the shortcut listing was visible only between those two keypresses

### Requirement: Every operation is keyboard-drivable
Every operation the app offers SHALL be operable without a pointer: the
daily loop (a shortcut triggers the Write button's action on Today, a
shortcut opens the Shelf, Escape and the existing mode keys return and
switch), choosing a spark when writing starts, the whole completion flow,
the Shelf's columns, cards, and drawer, and applying markdown-lite
formatting in the editor. Keyboard paths SHALL trigger exactly the same
actions and transitions as the pointer paths, through the same enforcement
points.

#### Scenario: Starting to write from the keyboard
- **WHEN** the user presses the Write shortcut on the Today screen
- **THEN** the app behaves exactly as if the Write button had been clicked

#### Scenario: Reaching the Shelf and back from the keyboard
- **WHEN** the user presses the Shelf shortcut on the Today screen, and presses it again on the Shelf
- **THEN** the Shelf screen is shown, and then the Today screen again

#### Scenario: The full conveyor without a pointer
- **WHEN** the user captures a spark, starts writing from it, writes, switches to Edit mode, and publishes — using only the keyboard
- **THEN** every step completes exactly as it would by pointer, and at no point is a pointer action required

