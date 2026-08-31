# fullscreen-rooms — Tasks

## 1. Find out what the window actually does

- [x] 1.1 Probe it: open esse's own window with esse's own activation policy,
      ask for native fullscreen, and report what the window says afterwards —
      `fullscreen=false`, 680×752, i.e. the call did nothing (design D1)
- [x] 1.2 Ask the same window for borderless fullscreen — 1512×982 at (0,0) —
      and delete the probe, keeping the numbers in the design

## 2. Take the whole screen

- [x] 2.1 `open_editor`: borderless fullscreen in place of the native toggle,
      with the reason it cannot be the native one written where the next reader
      will look for it
- [x] 2.2 `leave_editor`: give back what the editor took, decided by the
      router's own flag rather than by a window that answers a frame late
      (design D1)
- [x] 2.3 Leave the mode switch alone — entering takes the window once, and a
      switch between the rooms must not flicker

## 3. Say it in the specs

- [x] 3.1 write-mode and edit-mode deltas: the whole screen instead of native
      fullscreen, and the room is not a Space (design D2)

## 4. Checking it

- [x] 4.1 Whole suite green (191 tests)
- [x] 4.2 Look at it: Today → Write covers the display with no title bar;
      `cmd-e` to Edit and back does not flicker or resize; Escape gives back
      the 680×720 window where it was; the summon key puts a room away and
      brings it back the same size, with the menu bar behaving while esse is
      away — checked on the real machine, all as specified
- [x] 4.3 Decide about the notch strip: the corner controls at 22px read fine
      against the menu-bar band, so no safe-area inset (design, Risks)
