## MODIFIED Requirements

### Requirement: Autosave on pause
Edit mode SHALL persist the essay body through the atomic essay save after
roughly a second's pause in editing, and always when leaving Edit mode and
whenever the app goes away — quitting, the window closing, or esse being put
away by the summon key. `updated_at` follows each save. There is no manual
save and no unsaved-changes state surfaced to the user.

#### Scenario: A pause saves the text
- **WHEN** the user stops editing for the pause interval
- **THEN** the essay file on disk contains the current body

#### Scenario: Leaving never loses text
- **WHEN** the user leaves Edit mode or quits the app immediately after editing
- **THEN** the essay file on disk contains the text as last edited

#### Scenario: Being put away never loses text
- **WHEN** the user hides esse with the summon key or closes the window immediately after editing
- **THEN** the essay file on disk contains the text as last edited
