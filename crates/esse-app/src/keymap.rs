//! Every shortcut the app answers to, declared once.
//!
//! One row per shortcut: the keystroke, the action it fires, the key context it
//! lives in, and the words help says it in. `main` builds `bind_keys` from this
//! table and the help overlay reads the same rows, so a key that works cannot
//! go missing from help and a key help lists cannot be a fiction (design.md,
//! D1).
//!
//! Baseline text editing — arrows, clipboard, undo, the capture line's own keys
//! — is deliberately not here: it stays with the editor and the line input,
//! because those are platform conventions rather than this app's vocabulary,
//! and help does not list them.

use gpui::KeyBinding;

use crate::{edit, editor, help, shelf, today, write, Quit};

/// Where a shortcut lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// One key context, named as the screen names itself.
    In(&'static str),
    /// Bound wherever the help overlay is not. The overlay's own context has no
    /// bindings at all, which is what makes any key dismiss it and nothing else
    /// (design.md, D2).
    Everywhere,
}

impl Scope {
    /// The predicate gpui matches the binding against.
    fn predicate(self) -> &'static str {
        match self {
            Scope::In(context) => context,
            Scope::Everywhere => "!Help",
        }
    }
}

/// One shortcut, whole: what is pressed, what it does, where it works, and what
/// to call it.
pub struct Shortcut {
    pub keys: &'static str,
    pub scope: Scope,
    pub label: &'static str,
    /// Builds the binding. A function rather than a value because every action
    /// is its own type, and keeping the action beside its label is the point of
    /// the table.
    bind: fn(&'static str, Option<&'static str>) -> KeyBinding,
}

macro_rules! shortcut {
    ($keys:expr, $action:expr, $scope:expr, $label:expr) => {
        Shortcut {
            keys: $keys,
            scope: $scope,
            label: $label,
            bind: |keys, context| KeyBinding::new(keys, $action, context),
        }
    };
}

/// The whole keyboard vocabulary of the app. One table, in one place in
/// memory: `bind_keys` and the help overlay are reading the same rows.
pub static SHORTCUTS: &[Shortcut] = &[
    // Everywhere. `cmd-h` is the Hide key in most Mac apps; esse sets no
    // application menu, so nothing claims it, and hiding a one-window writing
    // app has no workflow anyway (design.md, D2).
    shortcut!(
        "cmd-h",
        help::Toggle,
        Scope::Everywhere,
        "Подсказка по клавишам"
    ),
    shortcut!("cmd-q", Quit, Scope::Everywhere, "Выйти из esse"),
    // Today, at rest. Both take a modifier, so plain typing keeps landing in
    // the capture line (keyboard-shortcuts spec).
    shortcut!("cmd-enter", today::Write, Scope::In(TODAY), "Писать"),
    shortcut!("cmd-l", today::Shelf, Scope::In(TODAY), "Полка"),
    // Today, choosing the spark to start from. The capture line does not hold
    // focus here, so the bare keys are free (design.md, D5).
    shortcut!("up", today::Previous, Scope::In(CHOOSING), "Искра выше"),
    shortcut!("down", today::Next, Scope::In(CHOOSING), "Искра ниже"),
    shortcut!(
        "enter",
        today::Choose,
        Scope::In(CHOOSING),
        "Начать с этой искры"
    ),
    shortcut!("escape", today::Cancel, Scope::In(CHOOSING), "Не начинать"),
    // The Shelf. Nothing here is typed into, so the arrows are free too.
    shortcut!("left", shelf::Left, Scope::In(SHELF), "Столбец левее"),
    shortcut!("right", shelf::Right, Scope::In(SHELF), "Столбец правее"),
    shortcut!("up", shelf::Up, Scope::In(SHELF), "Выше по столбцу"),
    shortcut!("down", shelf::Down, Scope::In(SHELF), "Ниже по столбцу"),
    shortcut!(
        "enter",
        shelf::Activate,
        Scope::In(SHELF),
        "Открыть выбранное"
    ),
    shortcut!(
        "cmd-d",
        shelf::Drawer,
        Scope::In(SHELF),
        "Стол: открыть и закрыть"
    ),
    shortcut!("escape", shelf::Leave, Scope::In(SHELF), "Назад, к сегодня"),
    shortcut!("cmd-l", shelf::Leave, Scope::In(SHELF), "Назад, к сегодня"),
    // Write mode.
    shortcut!(
        "escape",
        write::Leave,
        Scope::In(WRITE),
        "Выйти к сегодня"
    ),
    shortcut!("cmd-e", write::Switch, Scope::In(WRITE), "Перейти к правке"),
    // Edit mode.
    shortcut!("escape", edit::Leave, Scope::In(EDIT), "Выйти к сегодня"),
    shortcut!("cmd-e", edit::Switch, Scope::In(EDIT), "Вернуться к письму"),
    shortcut!("cmd-enter", edit::Finish, Scope::In(EDIT), "Закончить эссе"),
    // Markup, from inside the text. The same keys in both rooms: the editor is
    // one editor (design.md, D4).
    shortcut!("cmd-b", editor::ToggleBold, Scope::In(EDITOR), "Жирный"),
    shortcut!("cmd-i", editor::ToggleItalic, Scope::In(EDITOR), "Курсив"),
    shortcut!("cmd-1", editor::Heading1, Scope::In(EDITOR), "Заголовок 1"),
    shortcut!("cmd-2", editor::Heading2, Scope::In(EDITOR), "Заголовок 2"),
    shortcut!("cmd-3", editor::Heading3, Scope::In(EDITOR), "Заголовок 3"),
    // The completion overlay: a small key world of its own (design.md, D7).
    shortcut!(
        "tab",
        edit::NextAction,
        Scope::In(FINISHING),
        "Следующее действие"
    ),
    shortcut!(
        "right",
        edit::NextAction,
        Scope::In(FINISHING),
        "Следующее действие"
    ),
    shortcut!(
        "left",
        edit::PreviousAction,
        Scope::In(FINISHING),
        "Предыдущее действие"
    ),
    shortcut!("enter", edit::Activate, Scope::In(FINISHING), "Выбрать"),
    shortcut!("escape", edit::Leave, Scope::In(FINISHING), "Закрыть"),
];

/// The key contexts the screens declare. Named here so the table and the
/// `key_context` calls cannot disagree by a typo.
pub const TODAY: &str = "Today";
pub const CHOOSING: &str = "TodayChoosing";
pub const SHELF: &str = "Shelf";
pub const WRITE: &str = "Write";
pub const EDIT: &str = "Edit";
pub const EDITOR: &str = "Editor";
pub const FINISHING: &str = "Finishing";
pub const LINE_INPUT: &str = "LineInput";
/// The help overlay's own context, which nothing is bound in.
pub const HELP: &str = "Help";

/// Every binding, for `cx.bind_keys` at startup.
pub fn bindings() -> Vec<KeyBinding> {
    SHORTCUTS
        .iter()
        .map(|shortcut| (shortcut.bind)(shortcut.keys, Some(shortcut.scope.predicate())))
        .collect()
}

/// Where the writer is standing — the innermost state the eyes are on, and the
/// list help shows for it (design.md, D3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Today,
    ChoosingSpark,
    Write,
    Edit,
    Finishing,
    Shelf,
}

impl Place {
    /// What the place calls itself, at the top of the help sheet.
    pub fn title(self) -> &'static str {
        match self {
            Place::Today => "Сегодня",
            Place::ChoosingSpark => "С какой искры начать",
            Place::Write => "Пишу",
            Place::Edit => "Правлю",
            Place::Finishing => "Закончить эссе",
            Place::Shelf => "Полка",
        }
    }

    /// The key contexts active here, outermost first. The editor rooms carry
    /// the editor's own context inside them.
    fn contexts(self) -> &'static [&'static str] {
        match self {
            Place::Today => &[TODAY],
            Place::ChoosingSpark => &[CHOOSING],
            Place::Write => &[WRITE, EDITOR],
            Place::Edit => &[EDIT, EDITOR],
            Place::Finishing => &[FINISHING],
            Place::Shelf => &[SHELF],
        }
    }
}

/// The shortcuts that work in `place`, in table order.
pub fn shortcuts(place: Place) -> impl Iterator<Item = &'static Shortcut> {
    SHORTCUTS.iter().filter(move |shortcut| match shortcut.scope {
        Scope::In(context) => place.contexts().contains(&context),
        Scope::Everywhere => false,
    })
}

/// The shortcuts that work wherever you are — help's short trailing section.
pub fn everywhere() -> impl Iterator<Item = &'static Shortcut> {
    SHORTCUTS
        .iter()
        .filter(|shortcut| shortcut.scope == Scope::Everywhere)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLACES: [Place; 6] = [
        Place::Today,
        Place::ChoosingSpark,
        Place::Write,
        Place::Edit,
        Place::Finishing,
        Place::Shelf,
    ];

    /// The drift guard (design.md, D1): the table is what `bind_keys` consumes,
    /// so every binding that exists is a row — and every row has to be sayable
    /// out loud, or help would list a blank line.
    #[test]
    fn every_binding_is_a_labelled_row() {
        assert_eq!(bindings().len(), SHORTCUTS.len(), "one binding per row");

        for shortcut in SHORTCUTS {
            assert!(
                !shortcut.label.trim().is_empty(),
                "{} in {:?} has no label",
                shortcut.keys,
                shortcut.scope
            );
            assert!(
                !shortcut.keys.trim().is_empty(),
                "a row with the label {:?} has no keystroke",
                shortcut.label
            );
        }
    }

    /// Every row is reachable from some place's help, and the help overlay's
    /// own context is bound in by nothing.
    #[test]
    fn every_row_is_listed_somewhere() {
        for shortcut in SHORTCUTS {
            let same = |listed: &Shortcut| {
                listed.keys == shortcut.keys && listed.scope == shortcut.scope
            };
            let listed = PLACES
                .iter()
                .any(|place| shortcuts(*place).any(same))
                || everywhere().any(same);
            assert!(listed, "{} is bound but nowhere to be seen", shortcut.keys);
        }

        for shortcut in SHORTCUTS {
            assert_ne!(
                shortcut.scope,
                Scope::In(HELP),
                "nothing is bound while help is up"
            );
        }
    }

    /// Each place says something, and no place accidentally lists another's
    /// keys — the overlay follows the innermost state (shortcut-help spec).
    #[test]
    fn every_place_has_its_own_list() {
        for place in PLACES {
            assert!(
                shortcuts(place).next().is_some(),
                "{place:?} lists nothing at all"
            );
        }

        let today: Vec<_> = shortcuts(Place::Today).map(|s| s.keys).collect();
        assert_eq!(today, ["cmd-enter", "cmd-l"]);
        assert!(
            !shortcuts(Place::Finishing).any(|s| s.keys == "cmd-e"),
            "the completion overlay does not list Edit mode's keys"
        );
        assert!(
            shortcuts(Place::Write).any(|s| s.keys == "cmd-b"),
            "the editor's markup keys work in Write mode too"
        );
    }

    /// One keystroke means one thing in one context.
    #[test]
    fn no_context_binds_a_key_twice() {
        for (index, shortcut) in SHORTCUTS.iter().enumerate() {
            for other in &SHORTCUTS[index + 1..] {
                assert!(
                    shortcut.keys != other.keys || shortcut.scope != other.scope,
                    "{} is bound twice in {:?}",
                    shortcut.keys,
                    shortcut.scope
                );
            }
        }
    }

    /// gpui parses keystrokes and predicates at bind time and panics on a bad
    /// one; better to find out here than on the first launch.
    #[test]
    fn every_binding_parses() {
        let bindings = bindings();
        assert_eq!(bindings.len(), SHORTCUTS.len());
    }
}
