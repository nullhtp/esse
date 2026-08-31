//! Ink on paper. The app has no settings, so this is the whole appearance.

/// Colours, as `0xRRGGBB`.
pub const BACKGROUND: u32 = 0xfbfaf7;
pub const INK: u32 = 0x23211e;
pub const MUTED: u32 = 0x8f8a80;
pub const RULE: u32 = 0xe2ddd3;
pub const CARET: u32 = 0x2563eb;
pub const ALARM: u32 = 0xb3261e;

pub const INPUT_SIZE: f32 = 19.;
pub const BODY_SIZE: f32 = 16.;
pub const SMALL_SIZE: f32 = 13.;
pub const LINE_SPACING: f32 = 1.5;

pub const PAGE_PADDING: f32 = 40.;
/// Width of the text column — prose wants a measure, not the whole window.
pub const MEASURE: f32 = 620.;
