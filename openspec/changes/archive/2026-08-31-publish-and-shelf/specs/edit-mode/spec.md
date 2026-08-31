# edit-mode Delta

## MODIFIED Requirements

### Requirement: Calm full-text screen
Edit mode SHALL fill the window with the editor and nothing else — no panels,
toolbars, or navigation chrome; the only overlays are the quiet mode-switch
control, the quiet finish control that opens the essay-completion flow, and,
when needed, the save-trouble notice. The whole text renders at full strength
with no dimming, in a light, calm treatment that MUST be visibly distinct
from Write mode's dark flow treatment. Entering Edit mode from the Today
screen SHALL request native fullscreen; leaving to Today SHALL restore the
previous window state.

#### Scenario: Entering Edit mode shows the whole essay
- **WHEN** the user opens an essay that is in Editing state
- **THEN** the window enters fullscreen and shows the full text at full strength in the light Edit treatment

#### Scenario: The two modes look different at a glance
- **WHEN** the user switches between Write mode and Edit mode
- **THEN** the background and text treatment change between the dark flow look and the light calm look

#### Scenario: The finish control stays quiet
- **WHEN** the user is editing in Edit mode without touching the finish control
- **THEN** the finish control remains a small unobtrusive overlay and nothing else is added to the screen
