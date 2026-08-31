# visible-data-folder — Design

## Context

`DataDir::open` resolves one path and creates it: `ESSE_DATA_DIR` if set,
otherwise `dirs::data_dir()/esse`. Everything else in the core is handed paths
by it, so moving the folder is a change in one function — plus the question
nobody has had to answer until now: what happens to the files already sitting
in the old place.

The data has no schema version and no ceremony around it: three kinds of file,
all plain text, all written atomically. That is what makes a move cheap and a
merge unthinkable — two directories with the same file names are two truths,
and picking between them is not a decision an app should make behind the
writer's back.

## Goals / Non-Goals

**Goals:**

- The essays and sparks live where a person can find them without being told.
- An existing installation carries over silently and exactly once.
- The app can say the path out loud and open the folder.

**Non-Goals:**

- Choosing the folder, moving it later, or syncing it.
- Reading or listing files in the app beyond what the Shelf already shows.
- Any change to the file formats or to the WIP = 1 invariant — the storage
  layer keeps enforcing it exactly as before.

## Decisions

**D1. `~/Documents/Esse`, not `~/Esse` or the hidden directory.** The folder
has to be somewhere a person already looks and already backs up. Documents is
that place on every desktop, and the capitalised `Esse` reads as a name rather
than a dotfile. `~/Esse` was the alternative — it dodges iCloud's Documents
syncing, which can be a nuisance — but a folder in the home directory is one a
writer has to be taught about, and teaching is exactly what this change is
trying to stop doing. `dirs::document_dir()` resolves it; if the platform has
no documents directory, the home directory is the fallback, which keeps the
function total on machines that report nothing.

**D2. Move the whole directory, once, with `rename`.** Migration is
`fs::rename(old, new)` — atomic on one volume, and it leaves nothing behind to
go stale. If `rename` fails because the two paths are on different volumes,
the code falls back to copying the tree and removing the source only after
every file has landed. A failure at any point is an error that stops the
launch: starting on an empty directory while a full one exists nearby is the
one outcome a writer must never see.

**D3. Never merge.** The move happens only when the new location does not
exist. If both exist, the new one wins and the old one is left exactly as it
is — recoverable by hand, which is the whole point of plain files. This also
makes the migration idempotent by construction: after the first launch the new
directory exists, so no later launch can look at the old path again.

**D4. `ESSE_DATA_DIR` suppresses the migration.** A named path is a deliberate
one — a test's temporary directory, a second copy for development. Moving
someone's real data into it because it happened to be empty would be a
surprise, and tests would start racing over a shared source directory.

**D5. The path belongs on the Shelf, not on Today.** Today is the screen for
starting; every line on it competes with the Write button and the spark line.
The Shelf is already the answer to "what do I have", so "and here is where it
lives" is the same sentence continued. It sits under the drawer, in the same
muted register as the drawer's own label, and it says `~/Documents/Esse` with
the home directory shortened — a path a person can read, not a path a program
prints.

**D6. `cmd-o` opens the folder, through `open_with_system`.** The keyboard is
the app's ordinary way of doing anything, so the line gets a key and the key
gets a row in the shortcut table, which is what puts it in the help sheet
automatically. gpui's `App::open_with_system` hands the folder to the platform
— on macOS that is Finder — and needs no new dependency. A click on the line
does the same thing, because pointer and keyboard paths go through one action.

## Risks / Trade-offs

- **iCloud Drive syncs Documents when it is turned on** → the folder may
  appear on other machines, and a file being written on two of them at once is
  a conflict esse does not resolve. Accepted: the same is true of any folder a
  writer syncs, the essays are small text files written atomically, and the
  alternative (a hidden folder nobody syncs) is the problem this change exists
  to fix.
- **A move that half-happens across volumes** → the copy path removes the
  source only after the whole tree is written, so a failure leaves the old
  directory intact and the launch stops with both paths named.
- **Someone has already made `~/Documents/Esse` for something else** → their
  directory is opened as it is rather than merged into; nothing is deleted,
  and the Shelf shows the path so the mistake is visible on the first screen.

## Migration Plan

1. Ship the new resolution and the move together — a build that reads the new
   path without moving would look like data loss.
2. On the first launch the directory moves; the writer sees nothing except the
   path at the foot of the Shelf.
3. Rollback is `mv ~/Documents/Esse ~/Library/Application\ Support/esse` and
   the previous binary: the files themselves are unchanged by this change.

## Open Questions

None.
