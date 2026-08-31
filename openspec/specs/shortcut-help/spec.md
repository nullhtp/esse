# shortcut-help Specification

## Purpose

The one surface on which shortcuts are visible: an overlay summoned with
`cmd-h`, listing exactly the keys that work where the writer is standing, and
gone again on the next keypress.

It exists because dogfooding disproved the original bet that shortcuts need no
discoverability at all — a shortcut nobody can find does not get used, which
costs the very thing the bindings were for (beginner problem #2, "cannot
start"). The anti-feature it avoids is the *persistent* one: a hint layered
onto a writing screen cannot be ignored, so this listing is visible only while
it is asked for. It is presentation only, and it never performs the key that
dismisses it — glancing at help mid-session can change nothing.

## Requirements
### Requirement: On-demand contextual help
Pressing `cmd-h` SHALL open a help overlay listing the shortcuts active in
the current context — Today (resting), Today (choosing a spark), Write mode,
Edit mode, the completion overlay, or the Shelf — each with its key and a
human-readable label, plus a short trailing section for the global keys
(`cmd-h`, `cmd-q`). The overlay MUST list only shortcuts that actually work
in that context and state; baseline text-editing conventions (arrows,
clipboard, undo) are not listed.

#### Scenario: Help on Today shows Today's shortcuts
- **WHEN** the user presses `cmd-h` on the resting Today screen
- **THEN** the overlay lists the Today shortcuts (write, shelf) and the global section, and nothing from other screens

#### Scenario: Help in the editor includes formatting
- **WHEN** the user presses `cmd-h` in Write mode or Edit mode
- **THEN** the overlay lists that mode's shortcuts including the formatting toggles and the mode's exit and switch keys

#### Scenario: Help follows the innermost state
- **WHEN** the user presses `cmd-h` while the completion overlay is open
- **THEN** the overlay lists the completion overlay's keys, not the underlying Edit mode set

### Requirement: Any key dismisses without acting
While the help overlay is open, the next keypress SHALL only dismiss the
overlay; it MUST NOT also perform the action bound to that key, type into
any input, or change any state.

#### Scenario: The same key closes it
- **WHEN** the help overlay is open and the user presses `cmd-h` or Escape
- **THEN** the overlay closes and nothing else changes

#### Scenario: A shortcut key closes without firing
- **WHEN** the help overlay is open in Edit mode and the user presses `cmd-e`
- **THEN** the overlay closes, the app stays in Edit mode, and no mode switch happens

### Requirement: The overlay is presentation only
Opening and closing the help overlay SHALL NOT modify the essay text, the
session, focus-visible input contents, or any persisted state; after
dismissal the screen behind it is exactly as it was.

#### Scenario: Glancing at help mid-session changes nothing
- **WHEN** the user opens and dismisses the help overlay during a writing session
- **THEN** the text, cursor position, and running session are unchanged

### Requirement: Help stays truthful to the bindings
The help overlay and the registered key bindings SHALL come from a single
declaration per shortcut, so a shortcut cannot be bound in a context without
appearing in that context's help with a label.

#### Scenario: A bound shortcut is a listed shortcut
- **WHEN** a shortcut is registered for a context (outside the baseline text-editing set)
- **THEN** the help overlay for that context lists it with a non-empty label

