//! Ink on paper. The app has no settings, so this is the whole appearance.

/// Colours, as `0xRRGGBB`.
pub const BACKGROUND: u32 = 0xfbfaf7;
pub const INK: u32 = 0x23211e;
/// The Write button under the pointer.
pub const INK_HOVER: u32 = 0x3b3833;
pub const MUTED: u32 = 0x8f8a80;
pub const RULE: u32 = 0xe2ddd3;
/// A spark row under the pointer, while one is being chosen.
pub const HIGHLIGHT: u32 = 0xefebe2;
pub const CARET: u32 = 0x2563eb;
pub const ALARM: u32 = 0xb3261e;

pub const INPUT_SIZE: f32 = 19.;
pub const BODY_SIZE: f32 = 16.;
pub const SMALL_SIZE: f32 = 13.;
pub const LINE_SPACING: f32 = 1.5;

pub const PAGE_PADDING: f32 = 40.;
/// Width of the text column — prose wants a measure, not the whole window.
pub const MEASURE: f32 = 620.;

/// Write mode: the same ink, at night. Dark, quiet, and unmistakably a
/// different mode from Today — entering it should be felt (write-mode spec).
pub mod write {
    pub const BACKGROUND: u32 = 0x15171b;
    pub const INK: u32 = 0xe6e2da;
    /// Paragraphs above the one being written. Dim enough to stop the eye,
    /// light enough to still be text — the constant dogfooding tunes.
    pub const DIM_INK: u32 = 0x4e535c;
    /// Raw markers on the cursor's line: present, not loud.
    pub const MARKER_INK: u32 = 0x767d88;
    pub const SELECTION: u32 = 0x3b82f645;
    pub const CARET: u32 = 0x7aa2f7;
    /// The session indicator, before and after the target.
    pub const INDICATOR: u32 = 0x3f444d;
    pub const INDICATOR_DONE: u32 = 0x7d8794;
    /// The corner control that switches to the other mode: present under the
    /// eye, invisible to it (design.md, D3).
    pub const SWITCH: u32 = 0x3f444d;
    pub const SWITCH_HOVER: u32 = 0x9aa3b0;
}

/// Edit mode: daylight. Paper, ink at full strength, and nothing faded — the
/// whole text is there to be read and cut about. Deliberately as far from the
/// Write palette as the two can get: the change of room has to be felt at a
/// glance (edit-mode spec, design.md D6).
pub mod edit {
    pub const BACKGROUND: u32 = 0xf4f1ea;
    pub const INK: u32 = 0x1b1a17;
    /// Raw markers on the cursor's line: present, not loud.
    pub const MARKER_INK: u32 = 0x9c968a;
    pub const SELECTION: u32 = 0x2563eb2b;
    pub const CARET: u32 = 0x2563eb;
    /// The corner control that switches back to Write mode.
    pub const SWITCH: u32 = 0xb5aea1;
    pub const SWITCH_HOVER: u32 = 0x6d675c;
}

/// The editor's own measures. Bigger than the Today list: this is the text
/// being written, not a line about it.
pub const EDITOR_SIZE: f32 = 19.;
pub const EDITOR_LINE_SPACING: f32 = 1.65;
pub const EDITOR_MEASURE: f32 = 680.;
pub const EDITOR_PADDING: f32 = 48.;

/// Headings are larger, and stay larger while the cursor is on them —
/// otherwise entering a heading would make the page jump.
pub fn heading_size(level: u8) -> f32 {
    match level {
        1 => 33.,
        2 => 27.,
        3 => 23.,
        _ => 21.,
    }
}
