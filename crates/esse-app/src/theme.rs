//! Ink on paper. The app has no settings, so this is the whole appearance.
//!
//! Three rooms, one material. Today and the Shelf are paper; Write is that same
//! paper at night; Edit is daylight on it. Each keeps its own ramp below, and no
//! room borrows a colour from another — the rooms have to be told apart at a
//! glance, and a shared value is how that slowly stops being true.
//!
//! The whole app is set in one serif at three weights (`fonts.rs`), so hierarchy
//! is made of size, weight and space rather than of colour or rules. There is
//! one accent, and it is the caret: the place the writing is happening.

/// Colours, as `0xRRGGBB`.
pub const BACKGROUND: u32 = 0xfaf8f1;
pub const INK: u32 = 0x211d16;
/// The Write button under the pointer.
pub const INK_HOVER: u32 = 0x3a342a;
pub const MUTED: u32 = 0x8b8371;
pub const RULE: u32 = 0xe6dfcf;
/// A spark row under the pointer, while one is being chosen.
pub const HIGHLIGHT: u32 = 0xf1ead9;
/// The one accent in the app: sienna, and only ever the caret. Everything else
/// is ink, paper, or the space between them.
pub const CARET: u32 = 0xb0532c;
/// Text selected in the capture line: the caret's own colour, laid on the page
/// thinly enough to read through — the accent is still only ever the caret.
pub const SELECTION: u32 = 0xb0532c2b;
/// Trouble the writer has to know about — deeper and redder than the caret, so
/// the two never read as the same thing.
pub const ALARM: u32 = 0x9e2b1c;

/// The type scale. Six sizes for the whole app: anything that wants a seventh
/// wants one of these instead.
///
/// The size of the one thing a screen is built around — Today's Write button,
/// and the question the completion panel asks. Both are the reason their screen
/// is on, and neither is prose.
pub const LEAD_SIZE: f32 = 20.;
pub const INPUT_SIZE: f32 = 19.;
pub const BODY_SIZE: f32 = 17.;
pub const SMALL_SIZE: f32 = 13.;
/// Small capitals, letterspaced by [`label`] — the app's quietest register, for
/// naming a place or a column without adding a voice to the page.
pub const LABEL_SIZE: f32 = 11.;
pub const LINE_SPACING: f32 = 1.55;

/// The spacing rhythm. Every gap in the app is one of these, so the vertical
/// rhythm is a scale rather than a hundred separate decisions.
pub const SPACE_XS: f32 = 6.;
pub const SPACE_S: f32 = 10.;
pub const SPACE_M: f32 = 16.;
pub const SPACE_L: f32 = 24.;
pub const SPACE_XL: f32 = 36.;

/// Corners: one for anything sitting on the page, one for a sheet laid over it.
pub const RADIUS: f32 = 4.;
pub const RADIUS_PANEL: f32 = 8.;

pub const PAGE_PADDING: f32 = 44.;
/// Width of the text column — prose wants a measure, not the whole window.
pub const MEASURE: f32 = 620.;

/// Where the corner controls sit. Far enough into the corner to be out of the
/// way, close enough to the text to belong to the same page.
pub const CORNER_TOP: f32 = 22.;
pub const CORNER_SIDE: f32 = 26.;

/// A label in the app's quietest register: small capitals, letterspaced.
///
/// gpui has no letter-spacing, so the tracking is set with thin spaces. That is
/// deliberate rather than a workaround being tolerated: a thin space is
/// whitespace, so a face without the glyph costs an advance and draws nothing —
/// there is no way for this to fail visibly. Literata has it (`fonts.rs` tests
/// say so), and the spacing is what makes 11px capitals read as a label instead
/// of as shouting.
pub fn label(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for (index, character) in text.chars().enumerate() {
        if index > 0 {
            out.push('\u{2009}');
        }
        // `to_uppercase` is per-character and can yield more than one, which is
        // the right thing for both alphabets the app is written in.
        out.extend(character.to_uppercase());
    }
    out
}

/// Write mode: the same paper, at night. Dark, quiet, and unmistakably a
/// different room from Today — entering it should be felt (write-mode spec).
/// Warm rather than blue: it is the same ink and paper with the light off.
pub mod write {
    pub const BACKGROUND: u32 = 0x191613;
    pub const INK: u32 = 0xeae4d6;
    /// Paragraphs above the one being written. Dim enough to stop the eye,
    /// light enough to still be text — the constant dogfooding tunes.
    pub const DIM_INK: u32 = 0x5c554b;
    /// Raw markers on the cursor's line: present, not loud.
    pub const MARKER_INK: u32 = 0x837a6c;
    pub const SELECTION: u32 = 0xd98e4f38;
    pub const CARET: u32 = 0xd98e4f;
    /// The session indicator, before and after the target.
    pub const INDICATOR: u32 = 0x4a443b;
    pub const INDICATOR_DONE: u32 = 0x8a8071;
    /// The corner control that switches to the other mode: present under the
    /// eye, invisible to it (design.md, D3).
    pub const SWITCH: u32 = 0x4a443b;
    pub const SWITCH_HOVER: u32 = 0xa39887;
}

/// Edit mode: daylight. Paper, ink at full strength, and nothing faded — the
/// whole text is there to be read and cut about. Deliberately as far from the
/// Write palette as the two can get: the change of room has to be felt at a
/// glance (edit-mode spec, design.md D6).
///
/// The ramp is complete on purpose, down to its own muted ink and its own
/// button: the completion panel sits in this room, and a colour borrowed from
/// Today would put a piece of another screen in the middle of it.
pub mod edit {
    pub const BACKGROUND: u32 = 0xf6f1e6;
    pub const INK: u32 = 0x1d1a14;
    pub const MUTED: u32 = 0x8a8171;
    /// Raw markers on the cursor's line: present, not loud.
    pub const MARKER_INK: u32 = 0xb6ab97;
    pub const SELECTION: u32 = 0xb0532c2b;
    pub const CARET: u32 = 0xb0532c;
    /// The corner controls: back to Write mode, and on to finishing.
    pub const SWITCH: u32 = 0xb6ab97;
    pub const SWITCH_HOVER: u32 = 0x6b6355;

    /// The completion overlay: a sheet of paper laid on the page. The veil is
    /// thin on purpose — the text stays readable behind it, because deciding
    /// to finish is a decision about the text (essay-completion spec).
    pub const VEIL: u32 = 0xf6f1e6d6;
    pub const PANEL: u32 = 0xfbf8f1;
    pub const PANEL_BORDER: u32 = 0xe3dac7;
    /// The quiet actions inside the panel — copying, exporting, going back.
    pub const ACTION: u32 = 0xece5d4;
    pub const ACTION_HOVER: u32 = 0xe2d9c4;
    /// The one action that ends the essay, in this room's own ink.
    pub const BUTTON: u32 = 0x1d1a14;
    pub const BUTTON_HOVER: u32 = 0x393328;
    pub const BUTTON_INK: u32 = 0xfbf8f1;
}

/// The editor's own measures. Bigger than the Today list: this is the text
/// being written, not a line about it.
pub const EDITOR_SIZE: f32 = 20.;
pub const EDITOR_LINE_SPACING: f32 = 1.62;
pub const EDITOR_MEASURE: f32 = 680.;
pub const EDITOR_PADDING: f32 = 48.;

/// Headings are larger, and stay larger while the cursor is on them —
/// otherwise entering a heading would make the page jump. The steps are wide
/// enough to tell apart at a glance and stop short of display sizes: this is an
/// essay, and its title is still text on the same page.
pub fn heading_size(level: u8) -> f32 {
    match level {
        1 => 32.,
        2 => 26.,
        // Markdown-lite parses six levels, and an essay uses two. The rest stop
        // here rather than stepping down into the body size, so that every
        // heading still looks like one.
        _ => 22.,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_label_is_letterspaced_capitals() {
        assert_eq!(label("Shelf"), "S\u{2009}H\u{2009}E\u{2009}L\u{2009}F");
        // The gap between words widens by the same thin space on either side,
        // which is what keeps two words reading as two.
        assert_eq!(label("In progress"), "I\u{2009}N\u{2009} \u{2009}P\u{2009}R\u{2009}O\u{2009}G\u{2009}R\u{2009}E\u{2009}S\u{2009}S");
        assert_eq!(label(""), "");
    }

    /// Essays and their titles are written in Russian as often as in English.
    #[test]
    fn a_label_capitalises_both_alphabets() {
        assert_eq!(label("Искры"), "И\u{2009}С\u{2009}К\u{2009}Р\u{2009}Ы");
    }
}
