//! Turning a spark into a file name.
//!
//! A slug is what the essay is called on disk, in a directory a person reads
//! and syncs — so it should still say what the essay is about. Sparks are
//! mostly Russian and file names are kept ASCII (design.md, D5), which leaves
//! transliteration as the only way to have both.

use crate::model::{now, Timestamp};

/// Longest slug we derive. Not a limit anything enforces — just the point
/// where a file name stops being readable and starts being a paragraph.
const MAX_LENGTH: usize = 48;

/// Derives an essay slug from spark text.
///
/// Cyrillic is transliterated, everything is lowercased, every other run of
/// non-alphanumerics becomes a single hyphen, and the result is cut at a word
/// boundary. A spark that leaves nothing usable — punctuation, emoji, a script
/// the table does not cover — falls back to `essay-<date>`.
pub fn derive_slug(text: &str) -> String {
    derive_slug_on(text, now())
}

/// The date only reaches the fallback, so tests can pin it.
fn derive_slug_on(text: &str, today: Timestamp) -> String {
    let hyphenated = hyphenate(text);
    let derived = truncate_at_word(&hyphenated);
    if derived.is_empty() {
        format!("essay-{}", today.format("%Y-%m-%d"))
    } else {
        derived.to_string()
    }
}

/// Transliterate, lowercase, and collapse everything else into single hyphens.
fn hyphenate(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    for ch in text.chars() {
        match transliterate(ch) {
            // A letter the table knows, or ASCII that already is one.
            Some(latin) => slug.push_str(latin),
            None if ch.is_ascii_alphanumeric() => slug.push(ch.to_ascii_lowercase()),
            // Anything else — punctuation, spaces, a script we cannot spell —
            // is a word break. Runs of them collapse into one hyphen.
            None => {
                if !slug.ends_with('-') {
                    slug.push('-');
                }
            }
        }
    }
    slug.trim_matches('-').to_string()
}

/// Cut to [`MAX_LENGTH`] at a hyphen, so a slug ends on a whole word. A single
/// word longer than the limit is cut mid-word: there is no boundary to use.
fn truncate_at_word(slug: &str) -> &str {
    if slug.len() <= MAX_LENGTH {
        return slug;
    }
    // One byte past the limit, so a cut landing exactly on a hyphen keeps the
    // word before it whole.
    let head = &slug[..=MAX_LENGTH];
    match head.rfind('-') {
        Some(boundary) => &slug[..boundary],
        None => &slug[..MAX_LENGTH],
    }
}

/// Russian letters, GOST-style: the spelling a reader would recognise, not a
/// reversible encoding. `ь` and `ъ` spell nothing at all.
fn transliterate(ch: char) -> Option<&'static str> {
    let latin = match ch.to_lowercase().next().unwrap_or(ch) {
        'а' => "a",
        'б' => "b",
        'в' => "v",
        'г' => "g",
        'д' => "d",
        'е' | 'ё' | 'э' => "e",
        'ж' => "zh",
        'з' => "z",
        'и' | 'й' => "i",
        'к' => "k",
        'л' => "l",
        'м' => "m",
        'н' => "n",
        'о' => "o",
        'п' => "p",
        'р' => "r",
        'с' => "s",
        'т' => "t",
        'у' => "u",
        'ф' => "f",
        'х' => "kh",
        'ц' => "ts",
        'ч' => "ch",
        'ш' => "sh",
        'щ' => "shch",
        'ы' => "y",
        'ъ' | 'ь' => "",
        'ю' => "yu",
        'я' => "ya",
        _ => return None,
    };
    Some(latin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(text: &str) -> String {
        derive_slug_on(text, "2026-08-31T09:00:00+03:00".parse().unwrap())
    }

    #[test]
    fn a_russian_spark_becomes_a_readable_ascii_slug() {
        assert_eq!(on("почему эссе"), "pochemu-esse");
        assert_eq!(on("Живёшь ещё?"), "zhivesh-eshche");
        assert_eq!(on("Цифры и Ъ"), "tsifry-i");
    }

    #[test]
    fn english_and_digits_pass_through_lowercased() {
        assert_eq!(on("Why Essays 2"), "why-essays-2");
    }

    #[test]
    fn punctuation_and_spacing_collapse_into_single_hyphens() {
        assert_eq!(on("  что — делать?!  "), "chto-delat");
        assert_eq!(on("а---б"), "a-b");
        assert_eq!(on("--слово--"), "slovo");
    }

    #[test]
    fn a_long_spark_is_cut_at_a_word_boundary() {
        // Reads as a sentence, so there are plenty of boundaries to cut at.
        let slug = on("почему черновик и правка это две разные работы которые нельзя смешивать");
        assert!(slug.len() <= MAX_LENGTH, "{slug} is {} long", slug.len());
        assert!(!slug.ends_with('-'));
        // Cut at a boundary means the last word survived whole.
        assert!(
            slug.split('-').next_back().is_some_and(|word| {
                "pochemu-chernovik-i-pravka-eto-dve-raznye-raboty"
                    .split('-')
                    .any(|whole| whole == word)
            }),
            "{slug}"
        );
    }

    #[test]
    fn one_very_long_word_is_cut_anyway() {
        let slug = on(&"а".repeat(100));
        assert_eq!(slug.len(), MAX_LENGTH);
    }

    #[test]
    fn a_spark_with_nothing_spellable_falls_back_to_the_date() {
        for text in ["", "  ", "!!!", "🔥🔥", "中文"] {
            assert_eq!(on(text), "essay-2026-08-31", "{text:?}");
        }
    }
}
