# writing-guidance Specification

## Purpose

The one surface where the method is written down: an overlay summoned with
`cmd-shift-h` that says what to do in the place the writer is standing — a
line on what the place is for, then the steps in the order they are performed,
from the first move to the one that leaves — and is gone on the next keypress.

It serves beginner problem #2 ("cannot start" — knowing the draft is allowed
to be bad unblocks the first sentence) and problem #3 ("drafting and editing
get mixed" — the guidance states the rule the two modes enforce); the spark
and finishing texts touch problems #1 and #5. It is not onboarding, not a
tour, and not AI: the texts are fixed, appear only when summoned, and never
read the essay.

## ADDED Requirements

### Requirement: On-demand step-by-step guidance for the current place
Pressing `cmd-shift-h` SHALL open a guidance overlay showing the instruction
for the current context — Today (resting), Today (choosing a spark), Write
mode, Edit mode, the completion overlay, or the Shelf. Every context
reachable by the writer MUST have its own instruction, and each MUST consist
of a lead line saying what the place is for followed by numbered steps in the
order they are performed, ending with the step that takes the writer out of
that place. The steps MUST describe what the app actually does, and MUST be
consistent with the concept's method (sparks are one-line seeds; Write mode
pours out a deliberately bad draft; Edit mode cuts and rearranges; essays end
by being published or deliberately shelved). The overlay MUST follow the
innermost state.

#### Scenario: Guidance in Write mode is the drafting procedure
- **WHEN** the user presses `cmd-shift-h` in Write mode
- **THEN** the overlay shows the Write-mode instruction — start with any first sentence, keep going without rereading, leave gaps, stop when the session indicator says the stretch is done, move on to Edit mode when the draft is whole — and nothing from other contexts

#### Scenario: Guidance in Edit mode is the editing procedure
- **WHEN** the user presses `cmd-shift-h` in Edit mode
- **THEN** the overlay shows the Edit-mode instruction — read it whole first, cut the marked parts largest-first, rearrange, fill the gaps, then work the sentences, title last — and nothing from other contexts

#### Scenario: Guidance follows the innermost state
- **WHEN** the user presses `cmd-shift-h` while the completion overlay is open
- **THEN** the overlay shows the finishing instruction, not the underlying Edit-mode one

#### Scenario: Every step is a step the app supports
- **WHEN** a guidance step tells the writer to do something in the app
- **THEN** that action exists in that place, through the behavior another capability's spec defines

### Requirement: Any key dismisses without acting
While the guidance overlay is open, the next keypress SHALL only dismiss
the overlay; it MUST NOT also perform the action bound to that key, type
into any input, or change any state. In particular, the shortcut-help key
pressed over the guidance overlay MUST only dismiss it, and the two overlays
MUST NOT be open at the same time.

#### Scenario: The same key closes it
- **WHEN** the guidance overlay is open and the user presses `cmd-shift-h` or Escape
- **THEN** the overlay closes and nothing else changes

#### Scenario: A shortcut key closes without firing
- **WHEN** the guidance overlay is open in Write mode and the user presses `cmd-e`
- **THEN** the overlay closes, the app stays in Write mode, and no mode switch happens

#### Scenario: The help key does not swap sheets
- **WHEN** the guidance overlay is open and the user presses `cmd-h`
- **THEN** the guidance overlay closes and the shortcut-help overlay does not open

### Requirement: The overlay is presentation only
Opening and closing the guidance overlay SHALL NOT modify the essay text,
the session, focused input contents, or any persisted state; after dismissal
the screen behind it is exactly as it was. The instructions MUST be static:
they never read or reference the writer's essay, sparks, or history.

#### Scenario: Glancing at guidance mid-session changes nothing
- **WHEN** the user opens and dismisses the guidance overlay during a writing session
- **THEN** the text, cursor position, and running session are unchanged

### Requirement: Guidance appears only when summoned
The guidance overlay SHALL have no persistent or automatic surface: it MUST
NOT open on first launch, on entering any screen or mode, or on any trigger
other than its shortcut. First-run onboarding remains exactly what the
first-run-onboarding capability defines.

#### Scenario: Entering a mode shows no guidance
- **WHEN** the user enters Write mode, Edit mode, or the Shelf without pressing the guidance shortcut
- **THEN** no guidance text or hint is shown

#### Scenario: First launch is unchanged
- **WHEN** the app is launched for the first time
- **THEN** the guidance overlay is not shown until its shortcut is pressed
