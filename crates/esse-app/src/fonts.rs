//! The faces the app is set in.
//!
//! Literata, embedded rather than installed: esse looks the same on any Mac it
//! is copied to, and the writing never falls back to a system sans mid-word.
//! The five files are static instances cut from the upstream variable fonts by
//! `scripts/fonts.py` — see `resources/fonts/README.md` for why static.
//!
//! One family at three weights, upright and italic. Everything the app draws
//! picks from that: body text at [`FontWeight::NORMAL`], headings and labels at
//! [`MEDIUM`], and the editor's `**bold**` at [`STRONG`].

use std::borrow::Cow;

use gpui::{App, FontWeight};

/// The family every face reports, and the only one the app ever asks for.
pub const FAMILY: &str = "Literata";

/// Headings, labels, and the Write button: present without being loud. A serif
/// at 600 shouts next to a 400 body; 500 is the weight that carries a heading.
pub const MEDIUM: FontWeight = FontWeight::MEDIUM;

/// `**bold**` inside the writing. One step above [`MEDIUM`], so a bold word
/// stands out from a heading rather than matching it.
pub const STRONG: FontWeight = FontWeight::SEMIBOLD;

const FACES: [&[u8]; 5] = [
    include_bytes!("../../../resources/fonts/Literata-Regular.ttf"),
    include_bytes!("../../../resources/fonts/Literata-Medium.ttf"),
    include_bytes!("../../../resources/fonts/Literata-SemiBold.ttf"),
    include_bytes!("../../../resources/fonts/Literata-Italic.ttf"),
    include_bytes!("../../../resources/fonts/Literata-SemiBoldItalic.ttf"),
];

/// Hand the faces to gpui, before anything is drawn.
///
/// A failure here is not worth refusing to start over — the app would still
/// run, set in the system font — so it is logged and stepped over, like every
/// other piece of platform trouble gpui reports.
pub fn load(cx: &App) {
    let faces = FACES.iter().map(|face| Cow::Borrowed(*face)).collect();
    if let Err(error) = cx.text_system().add_fonts(faces) {
        log::error!("could not load the fonts, falling back to the system face: {error}");
    }
}

#[cfg(test)]
mod tests {
    use font_kit::family_name::FamilyName;
    use font_kit::handle::Handle;
    use font_kit::properties::{Properties, Style, Weight};
    use font_kit::source::Source;
    use font_kit::sources::mem::MemSource;

    use super::*;

    /// The whole reason the faces are static and share one family name: gpui
    /// asks font-kit for "Literata" at a weight and a slant, and font-kit picks
    /// by the properties in the file. If a future re-cut lets the family names
    /// drift apart — one file calling itself "Literata Medium" — every weight
    /// would silently collapse onto whichever face was found first, and only
    /// the screen would tell. This asks the same question in a test.
    fn source() -> MemSource {
        let faces = FACES
            .iter()
            .map(|face| Handle::from_memory(std::sync::Arc::new(face.to_vec()), 0))
            .collect::<Vec<_>>();
        MemSource::from_fonts(faces.into_iter()).expect("the embedded faces load")
    }

    fn pick(source: &MemSource, weight: FontWeight, style: Style) -> String {
        let properties = Properties {
            weight: Weight(weight.0),
            style,
            ..Properties::default()
        };
        source
            .select_best_match(&[FamilyName::Title(FAMILY.to_string())], &properties)
            .expect("the family is there")
            .load()
            .expect("the face loads")
            .postscript_name()
            .expect("the face is named")
    }

    #[test]
    fn every_weight_the_app_asks_for_resolves_to_its_own_face() {
        let source = source();

        assert_eq!(
            pick(&source, FontWeight::NORMAL, Style::Normal),
            "Literata-Regular"
        );
        assert_eq!(pick(&source, MEDIUM, Style::Normal), "Literata-Medium");
        assert_eq!(pick(&source, STRONG, Style::Normal), "Literata-SemiBold");
        assert_eq!(
            pick(&source, FontWeight::NORMAL, Style::Italic),
            "Literata-Italic"
        );
        assert_eq!(
            pick(&source, STRONG, Style::Italic),
            "Literata-SemiBoldItalic"
        );
    }

    /// Essays here are written in Russian as often as in English, and the
    /// transliterated file names in `esse-core` say so. A face without Cyrillic
    /// would fall back mid-paragraph.
    #[test]
    fn the_faces_cover_both_alphabets_the_writing_uses() {
        let font = source()
            .select_best_match(
                &[FamilyName::Title(FAMILY.to_string())],
                &Properties::default(),
            )
            .expect("the family is there")
            .load()
            .expect("the face loads");

        for character in ['A', 'z', 'А', 'я', 'ё', '—', '…', '’', '\u{2009}'] {
            assert!(
                font.glyph_for_char(character).is_some(),
                "the face is missing {character:?}"
            );
        }
    }
}
