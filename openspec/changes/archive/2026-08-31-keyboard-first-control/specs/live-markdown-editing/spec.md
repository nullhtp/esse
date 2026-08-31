# live-markdown-editing Delta

## ADDED Requirements

### Requirement: Emphasis toggles from the keyboard
The editor SHALL toggle `**bold**` with `cmd-b` and `*italic*` with `cmd-i`
as buffer edits. With a selection contained in one source line, the
selection is wrapped in the markers, or unwrapped when the selection or the
span it sits in already carries that style — span boundaries come from the
markdown-lite parse, not string scanning. With no selection, the toggle
applies to the word at the cursor; on an empty line or with a selection
spanning multiple source lines it does nothing. Each toggle SHALL be a
single undo step, and the cursor or selection SHALL cover the same text
afterwards, markers excluded. Ambiguous cases (cursor on a marker, mixed
styles inside the selection) SHALL resolve to wrapping.

#### Scenario: Bold a selection
- **WHEN** the user selects plain text within one line and presses `cmd-b`
- **THEN** the selection is wrapped in `**` markers, renders bold, and remains selected without the markers

#### Scenario: Toggle bold off
- **WHEN** the user places the selection on text already inside a `**bold**` span and presses `cmd-b`
- **THEN** the `**` markers around that span are removed and the text renders plain

#### Scenario: No selection styles the word at the cursor
- **WHEN** the cursor sits inside a word with no selection and the user presses `cmd-i`
- **THEN** that word is wrapped in `*` markers and renders italic

#### Scenario: One undo step
- **WHEN** the user toggles emphasis and presses undo once
- **THEN** the text is exactly as it was before the toggle

#### Scenario: Multi-line selection is a no-op
- **WHEN** the selection spans more than one source line and the user presses `cmd-b`
- **THEN** the text is unchanged

### Requirement: Heading toggles from the keyboard
The editor SHALL set the current line's heading level with `cmd-1`, `cmd-2`,
and `cmd-3`: the line's existing heading marker (any level) is replaced with
`#{level} `, and pressing the key matching the line's current level strips
the marker, returning the line to body text. The edit SHALL be a single undo
step and the cursor SHALL keep its position within the line's text. Levels
4–6 remain available by typing only.

#### Scenario: A body line becomes a heading
- **WHEN** the cursor is on a plain line and the user presses `cmd-2`
- **THEN** the line gains a `## ` marker and renders as a level-2 heading

#### Scenario: Switching levels replaces the marker
- **WHEN** the cursor is on a `# ` heading line and the user presses `cmd-3`
- **THEN** the line's marker becomes `### ` with the text unchanged

#### Scenario: The same level toggles off
- **WHEN** the cursor is on a `## ` heading line and the user presses `cmd-2`
- **THEN** the marker is removed and the line renders as body text
