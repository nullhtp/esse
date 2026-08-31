# typographic-polish — Tasks

## 1. The face

- [x] 1.1 Pick a face against the real constraints: a serif drawn for reading
      on screen, an open licence, and Cyrillic coverage — the last of which
      ruled out the first candidate (design D1)
- [x] 1.2 Cut five static instances from the upstream variable fonts with
      `scripts/fonts.py`, rewriting the name tables so all five report one
      family and carry their weight in OS/2 (design D2); keep the script and
      the licence in the tree so the cut is reproducible
- [x] 1.3 Embed and register them in `fonts.rs`, logging and stepping over a
      failure rather than refusing to start; name the family once, on the root
      element, so every screen and both hand-shaped elements inherit it
- [x] 1.4 Test what the screen would otherwise be the only witness to: every
      weight resolves to its own face through the same matcher gpui uses, and
      the faces cover both alphabets

## 2. The scale, the rhythm, the palettes

- [x] 2.1 Rewrite `theme.rs`: six sizes, five gaps, two radii, and the corner
      geometry, each with the reason it exists
- [x] 2.2 Give Write and Edit complete palettes of their own — including Edit's
      own button and muted ink, which the completion panel had been taking from
      Today (design D4)
- [x] 2.3 Replace the interface blue with one accent on the caret, and warm the
      night palette so Write mode is Today with the light off
- [x] 2.4 Add `theme::label` — letterspaced capitals via thin spaces — with
      tests for both alphabets and for the word gap (design D5)

## 3. The screens

- [x] 3.1 Today: the Write button, the capture line's italic prompt, the spark
      list, the published cards and the session dots onto the scale
- [x] 3.2 The Shelf: column headings into the label register, rows and drawer
      onto the rhythm
- [x] 3.3 Write and Edit: corner controls into the label register, the session
      indicator left as a plain fact rather than made into a label
- [x] 3.4 The completion panel: Edit's own colours throughout, and the question
      asked at reading size instead of captioned in grey
- [x] 3.5 The two sheets — shortcuts and guidance — onto the same panel
      treatment as the completion overlay
- [x] 3.6 The editor: headings at medium and `**bold**` a step above, so a bold
      word inside a heading is visible as one

## 4. Checking it

- [x] 4.1 Whole suite green (179 tests), and the app launches with no font
      trouble logged — 201 ms to first frame, against a one-second budget
- [ ] 4.2 Look at it: all four screens and both sheets, in both alphabets, with
      a real essay — the two constants most likely to want a second pass are
      the optical size across the 11-32px range and Write mode's dimmed ink
      (design, Risks)
- [ ] 4.3 Decide whether tracked capitals read well in Cyrillic («И С К Р Ы» on
      the Shelf); if not, the label register goes Latin-only or loses the
      tracking
