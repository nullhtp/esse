# visible-data-folder — Tasks

## 1. The folder moves

- [x] 1.1 Resolve the data directory as `document_dir()/Esse`, falling back to
      the home directory when the platform reports no documents directory;
      `ESSE_DATA_DIR` keeps overriding everything (D1).
- [x] 1.2 Migrate on open: when the resolved directory does not exist and the
      old platform directory does, `rename` it across, falling back to a
      copy-then-remove when the paths sit on different volumes (D2, D3).
- [x] 1.3 Give the failure a name in `Error` and let it stop the launch with
      both paths in the message.
- [x] 1.4 Skip the migration entirely when `ESSE_DATA_DIR` is set (D4).
- [x] 1.5 Tests in `esse-core`: an old directory moves and opens; no old
      directory means an empty new one; both existing leaves the old untouched;
      a named directory is never migrated into.

## 2. The Shelf says it

- [x] 2.1 Hand the Shelf the data directory's path (the app already carries the
      `DataDir` to its stores).
- [x] 2.2 Draw the line at the foot of the Shelf, under the drawer, with the
      home directory written as `~` (D5).
- [x] 2.3 Add the `OpenFolder` action, bind `cmd-o` in the Shelf's context, and
      put the row in the shortcut table so the help sheet lists it (D6).
- [x] 2.4 Open the folder through `App::open_with_system` from both the key and
      a click on the line.
- [x] 2.5 Tests: the path is rendered the short way; the key and the click
      reach the same action.

## 3. The docs catch up

- [x] 3.1 README: the Data section names `~/Documents/Esse`, mentions that an
      existing installation moves itself once, and keeps `ESSE_DATA_DIR` as the
      development affordance it is.
- [x] 3.2 README keyboard section: `cmd-o` on the Shelf.
