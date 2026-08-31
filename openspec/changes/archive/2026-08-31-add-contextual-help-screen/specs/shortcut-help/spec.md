# shortcut-help Delta

## MODIFIED Requirements

### Requirement: On-demand contextual help
Pressing `cmd-h` SHALL open a help overlay listing the shortcuts active in
the current context — Today (resting), Today (choosing a spark), Write mode,
Edit mode, the completion overlay, or the Shelf — each with its key and a
human-readable label, plus a short trailing section for the global keys
(`cmd-h`, `cmd-shift-h`, `cmd-q`). The overlay MUST list only shortcuts that
actually work in that context and state; baseline text-editing conventions
(arrows, clipboard, undo) are not listed.

#### Scenario: Help on Today shows Today's shortcuts
- **WHEN** the user presses `cmd-h` on the resting Today screen
- **THEN** the overlay lists the Today shortcuts (write, shelf) and the global section, and nothing from other screens

#### Scenario: Help in the editor includes formatting
- **WHEN** the user presses `cmd-h` in Write mode or Edit mode
- **THEN** the overlay lists that mode's shortcuts including the formatting toggles and the mode's exit and switch keys

#### Scenario: Help follows the innermost state
- **WHEN** the user presses `cmd-h` while the completion overlay is open
- **THEN** the overlay lists the completion overlay's keys, not the underlying Edit mode set

#### Scenario: The global section lists the guidance key
- **WHEN** the user presses `cmd-h` in any context
- **THEN** the trailing global section lists `cmd-shift-h` with its guidance label alongside `cmd-h` and `cmd-q`
