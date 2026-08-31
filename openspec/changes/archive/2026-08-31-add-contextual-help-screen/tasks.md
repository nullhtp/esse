## 1. Keymap and contexts

- [x] 1.1 Add a `guidance::Toggle` action and register `cmd-shift-h` as a `Shortcut` with `Scope::Everywhere` and a Russian label («Как здесь работать»), so shortcut-help lists it in the global section automatically
- [x] 1.2 Add a `GUIDANCE` key context in `keymap.rs` and extend the `Scope::Everywhere` predicate to `"!Help && !Guidance"` so global keys bind in neither sheet's context
- [x] 1.3 Extend the keymap tests: `cmd-shift-h` appears in `everywhere()` with a non-empty label, nothing is bound in either sheet's context, and no global key is also claimed by a place

## 2. The instructions

- [x] 2.1 Write the six Russian instructions (Today, ChoosingSpark, Write, Edit, Finishing, Shelf): a lead line on what the place is for, then the steps in the order they are performed, ending with the step that leaves the place — sourced from CONCEPT.md and checked against the capability specs so no step promises behavior the app does not have
- [x] 2.2 Store them as a `Method { lead, steps }` per `keymap::Place` behind an exhaustive `match` beside the overlay module, so a new `Place` fails to compile without an instruction
- [x] 2.3 Tests: every `Place` has a lead and at least three steps, no step is blank, no two places say the same thing, and no instruction exceeds seven steps (the sheet cannot scroll)

## 3. Overlay

- [x] 3.1 Create `guidance.rs` mirroring `help.rs`: same veil, card, and typography family; title from the place, lead paragraph, then numbered steps with the number in a gutter so wrapped steps stay aligned
- [x] 3.2 The overlay occludes the pointer and dismisses on any key via `on_key_down` without performing the key's action

## 4. Root wiring

- [x] 4.1 Replace root's `helping: bool` with `sheet: Option<Sheet>` and a shared `sheet_focus`, keeping the existing focus capture and restoration (deviation from the planned second flag — see design.md D5)
- [x] 4.2 `toggle` puts a sheet up or takes down the one that is; the single field makes stacking unrepresentable
- [x] 4.3 Render the active sheet above the current screen, resolved from the same innermost `Place` the help sheet uses

## 5. Verification

- [x] 5.1 Manual pass per the spec scenarios, at the keyboard: guidance in Write/Edit shows the right instruction; over the completion overlay shows the finishing one; `cmd-e` over guidance only dismisses; `cmd-h` over guidance dismisses without opening help; text, cursor, and session unchanged after a glance. Done by hand — synthetic keystrokes are not a usable instrument here: with a Cyrillic layout active the system rewrites them before they reach the app (a scripted `z` arrived as `ф`), so neither a pass nor a failure driven that way means anything
- [x] 5.2 Confirm no automatic trigger: verified structurally — `sheet` is `None` at construction and assigned in exactly one place, inside `toggle`, which is reachable only from the two shortcut actions; nothing on first launch or on entering a screen can open it
- [x] 5.3 Run `cargo fmt --check`, `cargo clippy`, and `cargo test` across the workspace
