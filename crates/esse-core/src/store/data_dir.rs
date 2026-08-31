//! Where the files live.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::setup::Setup;

/// The name of the folder, as a person reads it in Finder.
const FOLDER: &str = "Esse";

/// The directory holding every file esse owns. Resolving the path and creating
/// the directory happen once, here; the stores are handed paths.
#[derive(Debug, Clone)]
pub struct DataDir {
    root: PathBuf,
}

impl DataDir {
    /// Opens the writer's folder (`~/Documents/Esse` on macOS), creating it on
    /// first launch and carrying an older installation across from the hidden
    /// platform directory exactly once (design.md, D1–D3).
    ///
    /// `ESSE_DATA_DIR` overrides the location and suppresses the move: a named
    /// path is a deliberate one, and nobody's real essays should walk into it
    /// (design.md, D4). A folder chosen when esse was set up comes next, for
    /// the same reason and with the same effect on the move (guided-install
    /// design.md, D4–D5).
    pub fn open() -> Result<Self> {
        if let Some(named) = env::var_os("ESSE_DATA_DIR").filter(|path| !path.is_empty()) {
            return DataDir::at(named);
        }

        if let Some(chosen) = Setup::read().data_dir {
            return DataDir::chosen(chosen);
        }

        DataDir::open_in(visible_root()?, platform_root())
    }

    /// Opens the folder named at setup. It is created if it is missing — a
    /// folder can be named before anything is written to it — but never
    /// conjured under a parent that is not there, which is what an unmounted
    /// volume looks like from here (guided-install design.md, D9).
    fn chosen(root: PathBuf) -> Result<Self> {
        match root.parent() {
            Some(parent) if !parent.as_os_str().is_empty() && !parent.exists() => {
                Err(Error::MissingLocation { path: root })
            }
            _ => DataDir::at(root),
        }
    }

    /// The platform-free half of [`DataDir::open`]: the folder to end up in and
    /// the folder an older installation may still be sitting in. Two paths in,
    /// so the move can be tested without touching the process's environment.
    fn open_in(root: PathBuf, previous: Option<PathBuf>) -> Result<Self> {
        // Only ever into an empty place: two directories of the same files are
        // two truths, and merging them is not a decision to make behind the
        // writer's back (design.md, D3).
        if !root.exists() {
            if let Some(old) = previous.filter(|old| old.is_dir() && old != &root) {
                migrate(&old, &root)?;
            }
        }
        DataDir::at(root)
    }

    /// Opens a specific directory, creating it if it does not exist.
    pub fn at(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(&root).map_err(Error::io(&root))?;
        Ok(DataDir { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn essays_dir(&self) -> PathBuf {
        self.root.join("essays")
    }

    pub fn sparks_path(&self) -> PathBuf {
        self.root.join("sparks.jsonl")
    }

    pub fn sessions_path(&self) -> PathBuf {
        self.root.join("sessions.jsonl")
    }
}

/// The folder a person can find: `Esse` in the documents directory, or in the
/// home directory on a platform that reports no documents directory.
fn visible_root() -> Result<PathBuf> {
    let parent = dirs::document_dir()
        .or_else(dirs::home_dir)
        .ok_or(Error::NoDataDir)?;
    Ok(parent.join(FOLDER))
}

/// Where the files used to live, before they were made visible.
fn platform_root() -> Option<PathBuf> {
    dirs::data_dir().map(|dir| dir.join("esse"))
}

/// Moves the old directory to the new place, whole. A rename is atomic within
/// one volume; across volumes there is nothing for it but to copy and only
/// then let go of the source, so a failure always leaves the old directory
/// standing (design.md, D2).
fn migrate(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(Error::io(parent))?;
    }

    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(_) => {
            copy_tree(from, to).map_err(Error::migration(from, to))?;
            fs::remove_dir_all(from).map_err(Error::migration(from, to))
        }
    }
}

/// A directory, copied recursively. Only ever used on esse's own data: three
/// kinds of plain file and one level of nesting.
fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    /// An old installation: one spark file and one essay, the shape the move
    /// has to carry across whole.
    fn old_installation(at: &Path) {
        fs::create_dir_all(at.join("essays")).unwrap();
        fs::write(at.join("sparks.jsonl"), "{\"text\":\"why essays\"}\n").unwrap();
        fs::write(at.join("essays/why-essays.md"), "+++\n+++\nthe body\n").unwrap();
    }

    /// The whole point of the change: someone who has been writing in the
    /// hidden directory finds their essays in the visible one, once.
    #[test]
    fn an_existing_installation_moves_itself() {
        let temp = TempDir::new().unwrap();
        let old = temp.path().join("Library/Application Support/esse");
        let new = temp.path().join("Documents/Esse");
        old_installation(&old);

        let dir = DataDir::open_in(new.clone(), Some(old.clone())).unwrap();

        assert_eq!(dir.root(), new);
        assert_eq!(
            fs::read_to_string(new.join("essays/why-essays.md")).unwrap(),
            "+++\n+++\nthe body\n"
        );
        assert!(new.join("sparks.jsonl").is_file());
        assert!(!old.exists(), "the old directory was left behind");
    }

    #[test]
    fn a_first_launch_just_creates_the_folder() {
        let temp = TempDir::new().unwrap();
        let new = temp.path().join("Documents/Esse");

        let dir = DataDir::open_in(new.clone(), Some(temp.path().join("nothing/here"))).unwrap();

        assert_eq!(dir.root(), new);
        assert!(new.is_dir());
        assert_eq!(fs::read_dir(&new).unwrap().count(), 0);
    }

    /// Two directories are never merged: the visible one is opened as it is,
    /// and the old one stays where a person can look at it.
    #[test]
    fn both_locations_leave_the_old_one_alone() {
        let temp = TempDir::new().unwrap();
        let old = temp.path().join("Library/Application Support/esse");
        let new = temp.path().join("Documents/Esse");
        old_installation(&old);
        fs::create_dir_all(&new).unwrap();
        fs::write(new.join("sparks.jsonl"), "{\"text\":\"newer\"}\n").unwrap();

        let dir = DataDir::open_in(new.clone(), Some(old.clone())).unwrap();

        assert_eq!(dir.root(), new);
        assert_eq!(
            fs::read_to_string(new.join("sparks.jsonl")).unwrap(),
            "{\"text\":\"newer\"}\n"
        );
        assert!(old.join("essays/why-essays.md").is_file());
    }

    /// A folder can be named at setup before a word has been written into it.
    #[test]
    fn a_folder_named_at_setup_is_created() {
        let temp = TempDir::new().unwrap();
        let named = temp.path().join("Writing/Essays");
        fs::create_dir_all(named.parent().unwrap()).unwrap();

        let dir = DataDir::chosen(named.clone()).unwrap();

        assert_eq!(dir.root(), named);
        assert!(named.is_dir());
    }

    /// The trap this guards: the folder is on a disk that is not plugged in,
    /// and esse quietly makes an empty one with the same name.
    #[test]
    fn a_folder_whose_volume_is_gone_stops_the_launch() {
        let temp = TempDir::new().unwrap();
        let named = temp.path().join("Volumes/Backup/Essays");

        let error = DataDir::chosen(named.clone()).unwrap_err();

        assert!(matches!(error, Error::MissingLocation { path } if path == named));
        assert!(!named.exists(), "an empty lookalike was created anyway");
    }

    /// Naming a folder is as deliberate as naming `ESSE_DATA_DIR`: nothing is
    /// carried into it behind the writer's back.
    #[test]
    fn a_folder_chosen_at_setup_is_never_migrated_into() {
        let temp = TempDir::new().unwrap();
        let old = temp.path().join("Library/Application Support/esse");
        let named = temp.path().join("Writing/Essays");
        old_installation(&old);
        fs::create_dir_all(named.parent().unwrap()).unwrap();

        let dir = DataDir::chosen(named.clone()).unwrap();

        assert_eq!(dir.root(), named);
        assert_eq!(fs::read_dir(&named).unwrap().count(), 0);
        assert!(old.join("essays/why-essays.md").is_file());
    }

    /// A copy across volumes cannot be provoked in a test, but the tree walk it
    /// leans on can: everything arrives, nesting included.
    #[test]
    fn the_copy_carries_the_whole_tree() {
        let temp = TempDir::new().unwrap();
        let old = temp.path().join("old");
        let new = temp.path().join("new");
        old_installation(&old);

        copy_tree(&old, &new).unwrap();

        assert_eq!(
            fs::read_to_string(new.join("essays/why-essays.md")).unwrap(),
            "+++\n+++\nthe body\n"
        );
        assert!(new.join("sparks.jsonl").is_file());
    }
}
