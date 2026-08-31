//! What was answered when esse was set up.
//!
//! `esse-setup` asks three questions once, in a terminal, and records the two
//! answers the app has to know about — where the writing lives and which key
//! brings esse forward. The app only ever reads this file: there is no
//! settings screen, no preferences pane, and nothing in the three screens can
//! change what is in it (guided-setup spec, "The answers are recorded where
//! every launch reads them").
//!
//! It lives in `~/Library/Preferences`, not beside the app's own files in
//! `~/Library/Application Support/esse` — that directory is the one
//! [`crate::DataDir`] treats as an older installation to be carried across, and
//! a file there would send the config directory into `~/Documents/Esse` on the
//! first launch of a fresh machine (design.md, D3).
//!
//! Only departures from the defaults are written, so a machine that took every
//! default has no file at all and behaves exactly like one that never ran setup
//! (design.md, D4).

use std::path::{Path, PathBuf};

/// The file, under whichever directory the platform keeps preferences in.
const FILE: &str = "com.nullhtp.esse.conf";

/// The two answers the app reads back. Absent means "the default", which is
/// also what an unreadable or missing file means: refusing to start over a
/// preferences file would be worse than starting the way esse always has.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Setup {
    /// The folder named at setup, if it was not the default one.
    pub data_dir: Option<PathBuf>,
    /// The combination named at setup, in the app's own spelling.
    pub hotkey: Option<String>,
}

impl Setup {
    /// Reads the recorded answers. A missing file, an unreadable one, or one
    /// full of lines nobody understands all mean the same thing: defaults.
    pub fn read() -> Setup {
        match Setup::path() {
            Some(path) => Setup::read_at(&path),
            None => Setup::default(),
        }
    }

    /// Where the answers are kept, when the platform will say.
    pub fn path() -> Option<PathBuf> {
        dirs::preference_dir().map(|dir| dir.join(FILE))
    }

    /// The platform-free half of [`Setup::read`], so the tests never go near
    /// the real home directory.
    pub fn read_at(path: &Path) -> Setup {
        match std::fs::read_to_string(path) {
            Ok(text) => Setup::parse(&text),
            Err(_) => Setup::default(),
        }
    }

    /// `key=value` a line at a time. Blank lines, `#` comments and keys this
    /// version has no use for are skipped rather than complained about: an
    /// older esse must not choke on a file a newer setup wrote.
    fn parse(text: &str) -> Setup {
        let mut setup = Setup::default();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            match key.trim() {
                "data_dir" => setup.data_dir = Some(expand_home(value)),
                "hotkey" => setup.hotkey = Some(value.to_string()),
                _ => {}
            }
        }

        setup
    }
}

/// `~/Writing/Essays` as a person types it, into a path the machine can open.
fn expand_home(value: &str) -> PathBuf {
    let rest = match value.strip_prefix('~') {
        Some(rest) => rest.strip_prefix('/').unwrap_or(rest),
        None => return PathBuf::from(value),
    };
    match dirs::home_dir() {
        Some(home) if !rest.is_empty() => home.join(rest),
        Some(home) => home,
        None => PathBuf::from(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile::TempDir;

    #[test]
    fn both_answers_are_read_back() {
        let setup = Setup::parse("data_dir=/Users/w/Writing/Essays\nhotkey=cmd-shift-space\n");

        assert_eq!(setup.data_dir, Some(PathBuf::from("/Users/w/Writing/Essays")));
        assert_eq!(setup.hotkey.as_deref(), Some("cmd-shift-space"));
    }

    /// The file records departures, so one answer on its own is the ordinary
    /// case, not a broken file.
    #[test]
    fn one_answer_leaves_the_other_default() {
        let setup = Setup::parse("hotkey=cmd-shift-space\n");

        assert_eq!(setup.data_dir, None);
        assert_eq!(setup.hotkey.as_deref(), Some("cmd-shift-space"));
    }

    #[test]
    fn comments_blank_lines_and_strange_keys_are_skipped() {
        let setup = Setup::parse(
            "# written by esse-setup\n\n  hotkey = ctrl-alt-e  \nfavourite_colour=blue\nnonsense\n",
        );

        assert_eq!(setup.hotkey.as_deref(), Some("ctrl-alt-e"));
        assert_eq!(setup.data_dir, None);
    }

    #[test]
    fn an_empty_value_is_no_answer() {
        let setup = Setup::parse("data_dir=\nhotkey=   \n");

        assert_eq!(setup, Setup::default());
    }

    #[test]
    fn a_leading_tilde_becomes_the_home_directory() {
        let home = dirs::home_dir().expect("a home directory to write essays in");
        let setup = Setup::parse("data_dir=~/Writing/Essays\n");

        assert_eq!(setup.data_dir, Some(home.join("Writing/Essays")));
    }

    #[test]
    fn a_missing_file_means_every_default() {
        let temp = TempDir::new().unwrap();

        assert_eq!(Setup::read_at(&temp.path().join("nothing.conf")), Setup::default());
    }

    #[test]
    fn a_file_is_read_from_disk() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(FILE);
        fs::write(&path, "data_dir=/tmp/essays\n").unwrap();

        assert_eq!(
            Setup::read_at(&path).data_dir,
            Some(PathBuf::from("/tmp/essays"))
        );
    }
}
