## MODIFIED Requirements

### Requirement: Autosave on pause
Write mode SHALL persist the essay body through the atomic essay save after
roughly a second's pause in editing, and always when leaving Write mode and
whenever the app goes away — quitting, the window closing, or esse being put
away by the summon key. `updated_at` follows each save. There is no manual
save and no unsaved-changes state surfaced to the user.

#### Scenario: A pause saves the draft
- **WHEN** the user stops typing for the pause interval
- **THEN** the essay file on disk contains the current body

#### Scenario: Leaving never loses text
- **WHEN** the user leaves Write mode or quits the app immediately after typing
- **THEN** the essay file on disk contains the text as last typed

#### Scenario: Being put away never loses text
- **WHEN** the user hides esse with the summon key or closes the window immediately after typing
- **THEN** the essay file on disk contains the text as last typed
