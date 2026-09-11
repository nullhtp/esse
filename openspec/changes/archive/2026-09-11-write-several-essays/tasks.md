## 1. The limit in the storage layer

- [x] 1.1 Add `pub const WIP_LIMIT: usize = 3` to `esse-core::model` with the comment explaining that three is a mechanic, not a capacity, and re-export it from `lib.rs`.
- [x] 1.2 Change `EssayStore::in_progress` to return `Result<Vec<Essay>>` — filter `load_all()` instead of `find`, keeping its `updated_at`-descending order so the most recently worked essay comes first.
- [x] 1.3 Change `EssayStore::create` to refuse when `in_progress()?.len() >= WIP_LIMIT`, leaving the slug validation and the `SlugTaken` check exactly where they are.
- [x] 1.4 Replace `Error::EssayInProgress { slug, status }` with `Error::TooManyInProgress { limit: usize }` and its message; fix the compile errors this raises in `start.rs` and anywhere else it is matched.
- [x] 1.5 Update the doc comments on `EssayStore::create`, `in_progress`, and `start_essay_from_spark` that spell out WIP = 1.

## 2. The storage tests

- [x] 2.1 Rewrite `a_second_essay_is_refused_while_one_is_in_progress` in `crates/esse-core/tests/storage.rs` as three-are-allowed plus a-fourth-is-refused, asserting the error is `TooManyInProgress` and that nothing was written.
- [x] 2.2 Fix every `in_progress()` assertion in `storage.rs` and `pipeline.rs` for the new `Vec` signature.
- [x] 2.3 Add a test that `in_progress()` puts the most recently saved essay first, and one that Published and Shelved essays never count toward the limit however many there are.
- [x] 2.4 Add a test that a refused start leaves the spark in the box (the `start.rs` path, with three essays already open).

## 3. The work chooser

- [x] 3.1 Add a `Choice` enum (continue an essay / start from a spark) and build the flat list on Today from the essays in progress followed by the sparks; keep `highlight` an index into it.
- [x] 3.2 Add `TodayEvent::Continue(String)` beside `Start(String)`, and make `enter` and a click emit the event matching the highlighted entry.
- [x] 3.3 Render the chooser's two sections — the open essays with their state, then the sparks — and carry the existing highlight styling across both.
- [x] 3.4 Show the quiet line about publishing or shelving to make room, and drop the sparks' start action, when three essays are in progress.
- [x] 3.5 Rename `offer_sparks` to `offer_choices` and stop it assuming the list is sparks; keep the focus handover to `choice_focus` and the `escape` path untouched.

## 4. Routing

- [x] 4.1 Rewrite `RootView::write_pressed` to always open the chooser, keeping only the empty-and-nothing-open case as a message and the disk error as a message.
- [x] 4.2 Handle `TodayEvent::Continue(slug)` by loading that essay and opening the editor in the mode matching its state.
- [x] 4.3 Update `start_from_spark`'s refusal arm for `TooManyInProgress`, with the copy that says three essays are open and how to make room.
- [x] 4.4 Give `ShelfEvent::Continue` a slug and load that essay in the handler, refreshing the Shelf when the slug is no longer in progress.

## 5. The Shelf

- [x] 5.1 Make `Contents::in_progress` a `Vec<Essay>` and update `read()`, `columns()`, and `startable()` (`len() < WIP_LIMIT`).
- [x] 5.2 Render the In-progress column as a list of cards, each showing Draft or Editing, most recently worked first, with no count anywhere.
- [x] 5.3 Make `activate` emit `Continue(slug)` for the highlighted card, and verify `up`/`down` walk the column with several cards in it.
- [x] 5.4 Show the "publish or shelve one to make room" line beside the Sparks column when three essays are open.

## 6. Copy, help and guidance

- [x] 6.1 Check `guidance.rs` and `help.rs` for text that assumes one essay at a time, and rewrite what is now false.
- [x] 6.2 Read the chooser's copy against the voice the UI already uses — step-by-step, human, inspiring — and settle the final wording of the full-conveyor line.

## 7. The documents that assert the invariant

- [x] 7.1 Rewrite the WIP paragraphs in `CONCEPT.md` — the conveyor section, the Shelf description, and the "how hard the WIP limit is" open question, which becomes a closed one pointing at this change.
- [x] 7.2 Rewrite the HARD INVARIANT block in `openspec/config.yaml` to state the limit of three, so future proposals are governed by the rule that now holds.
- [x] 7.3 Check `README.md` and `PLAN.md` for the same claim and fix what is stale.

## 8. Finishing

- [x] 8.1 Run `make test` and fix what falls over.
- [x] 8.2 Run `cargo clippy --all-targets` and `cargo fmt --check`. (Both were already dirty before this change and are no dirtier after: 59 rustfmt hunks across 20 files either way, and the four clippy warnings are all in code this change does not touch. Left alone rather than reformatting the repo inside this diff.)
- [x] 8.3 Hand Anton the scenarios to try by hand: open three essays, switch between them from the chooser and from the Shelf, hit the wall at the fourth, publish one and start another, and confirm `enter` alone still lands in the last essay.
- [x] 8.4 Run `openspec validate write-several-essays --strict` and archive the change once the scenarios pass.
