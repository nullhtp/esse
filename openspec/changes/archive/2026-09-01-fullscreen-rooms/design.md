# fullscreen-rooms — Design

## Context

The editor's two rooms have asked for fullscreen since the write-mode change
(D3 there: "entering Write mode requests native fullscreen for the window"),
and the install-and-summon change later made esse an Accessory application
(D5 there: the activation policy is set back to `Accessory` in the callback
after gpui has set it to `Regular`). The second decision silently cancelled
the first, and nothing said so: `toggleFullScreen:` on an Accessory app is not
an error, it is a no-op.

The write-mode design listed the opposite risk — that native fullscreen's
animation would make entering the room feel slow — and named the fallback:
"if it grates in practice, dropping to 'fills the window' is a one-line
change". Practice produced a different complaint, and the same fallback answers
it.

## Goals / Non-Goals

**Goals:**
- Entering either room covers the display; leaving restores the window.
- The room is reachable and leavable at the speed of the summon key, from
  whatever the writer was doing, without the desktop moving underneath them.
- The spec says something true, so the next reader does not re-implement the
  same no-op.

**Non-Goals:**
- Making native fullscreen work by giving up the Accessory policy. The app's
  presence in the machine is the summon key; a Dock icon is the larger loss.
- Any change to the rooms' content, layout, palettes, or keyboard map.
- A safe-area inset for the notch strip (see Risks).

## Decisions

### D1: Borderless fullscreen, driven by a flag rather than by the window
The window is put over the screen with gpui's `toggle_simple_fullscreen`:
the style mask goes borderless, the frame becomes the screen's frame, and the
app's presentation options hide the Dock and the menu bar while esse is the
active app. Leaving toggles it back, which restores the saved frame and style
mask.

Whether to toggle is read from a `filled_the_screen` flag on the router, not
from `Window::is_simple_fullscreen`. The toggle is carried out on a later turn
of the foreground executor, so a window asked immediately after entering still
answers for the frame before — a state that is right one frame and wrong the
next is not the thing to branch on. The flag records intent, which is exactly
what "give back what the editor took" needs, and a mode switch between the two
rooms leaves it alone, so the switch does not flicker (the constraint the
write-mode design already put on this code path).

*Rationale:* this is the only fullscreen an Accessory application can have.
Measured with a probe built on esse's own window and policy: after
`toggle_fullscreen` the window reports `fullscreen=false` and 680×752; after
`toggle_simple_fullscreen` it reports 1512×982 at (0,0).

*Invariant impact (required note):* none. WIP = 1 is untouched, and the
Write/Edit separation is strengthened rather than weakened — the rooms are now
the size the separation was specified to have.

### D2: Not a Space, on purpose
Even where native fullscreen is available, it would be the wrong shape here.
Native fullscreen moves the window to a Space of its own, and esse is summoned
from inside other work: pressing the key would swing the whole desktop across
to another Space, and pressing it again would swing it back. Borderless
fullscreen leaves the writing on the desktop the writer is already on. The
spec now says this outright, so that "use real fullscreen" cannot look like an
improvement to a later reader.

### D3: The Shelf still does not touch the window
Unchanged and worth restating, because the flag now lives next to it: the Shelf
is a plain screen. It is opened from Today and left back to Today with the
window exactly as it was, whatever size that is (shelf-screen spec).

## Risks / Trade-offs

- [The corner controls sit 22px from the top, and the top ~37px of a notched
  display is the menu-bar strip] → the "Editing" and "Finish" labels are to the
  right of the notch, so they are drawn, but they share a band with the menu
  bar when the pointer goes up there. Left as it is until it is looked at; a
  safe-area inset is a layout change, not a window change.
- [Being put away by the summon key while a room is open leaves the window
  borderless and screen-sized] → correct, and what coming back should look
  like: the summon key returns the writer to the same room. The presentation
  options that hide the Dock and menu bar are per-application and only apply
  while esse is the active app, so hiding esse gives the menu bar back on its
  own.
- [The window cannot be moved or resized while a room is open] → intended.
  There is nothing to arrange it against; Escape gives the window back.
