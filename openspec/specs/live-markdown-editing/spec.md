# live-markdown-editing Specification

## Purpose

The editor-core capability: Typora-like in-place rendering of markdown-lite,
typewriter mode, and baseline text-editing correctness.

These requirements were validated by the stage-0 framework prototype, which is
what settled the framework choice (gpui — see the archived
`editor-framework-prototype` change). They now govern the production editor
built for Write mode: the prototype's `markdown-lite` parser carried over
unchanged as the executable form of the rendering rules below, while its view
layer was rewritten around wrapped layout.
## Requirements
### Requirement: In-place markdown-lite rendering
The editor SHALL render markdown-lite styling in place, in the same text flow
where the user types. There MUST be no split preview, no separate source mode,
and no raw markup symbols in finished (non-active) text.

#### Scenario: Bold renders as you type
- **WHEN** the user types `**bold**` and moves the cursor off that line
- **THEN** the text renders as bold and the `**` markers are not displayed

#### Scenario: Italic renders as you type
- **WHEN** the user types `*italic*` and moves the cursor off that line
- **THEN** the text renders as italic and the `*` markers are not displayed

#### Scenario: Heading renders as you type
- **WHEN** the user types `# ` followed by text at the start of a line and moves the cursor off that line
- **THEN** the line renders in a visibly larger heading style and the `# ` marker is not displayed

### Requirement: Raw markers visible only at the editing point
The editor SHALL reveal raw markdown markers only on the line containing the
cursor, so the source stays editable without a separate mode.

#### Scenario: Entering a styled line reveals markers
- **WHEN** the cursor moves onto a line containing rendered bold, italic, or heading text
- **THEN** the raw markers (`**`, `*`, `# `) become visible on that line and the text remains editable as plain characters

#### Scenario: Leaving a line hides markers again
- **WHEN** the cursor leaves a line containing valid markdown-lite markup
- **THEN** the line immediately re-renders with styling applied and markers hidden

### Requirement: Markdown-lite is the whole syntax
The editor SHALL style only `#` headings, `*italic*`, and `**bold**`. All other
markdown syntax MUST be treated as plain text.

#### Scenario: Other markdown syntax stays plain
- **WHEN** the user types other markdown constructs (e.g. `- list`, `[link](url)`, `` `code` ``)
- **THEN** the text is displayed literally with no styling applied

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

### Requirement: Typewriter mode
The editor SHALL support a typewriter mode in which the visual (wrapped) line
containing the cursor stays vertically centred in the viewport.

#### Scenario: Cursor stays centred while typing
- **WHEN** typewriter mode is active and typing moves the cursor to a new visual line — by wrapping past the viewport width or by pressing Enter
- **THEN** the view scrolls so the cursor's visual line remains vertically centred

### Requirement: Baseline editing correctness
The editor SHALL support the baseline text-editing operations a writer relies
on: selection, copy/cut/paste, undo/redo, and Cyrillic input including IME
composition.

#### Scenario: Selection and clipboard round-trip
- **WHEN** the user selects a passage, cuts it, and pastes it at another position
- **THEN** the text moves intact, including any markdown-lite markers it contained

#### Scenario: Undo restores prior text
- **WHEN** the user performs edits and presses undo
- **THEN** the document returns to its state before the last edit, and redo re-applies it

#### Scenario: Cyrillic and IME input
- **WHEN** the user types Cyrillic text or composes characters through an IME
- **THEN** the characters are inserted correctly with no dropped or reordered input

### Requirement: Responsive at essay size
The editor SHALL remain responsive on documents of typical essay size
(up to roughly 20,000 characters), with no perceptible lag between a keystroke
and the character appearing styled on screen.

#### Scenario: Typing into a full-length essay
- **WHEN** the user types continuously into a document of ~20,000 characters
- **THEN** every keystroke appears without perceptible delay and scrolling stays smooth

