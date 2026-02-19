//! Threshold constants for PDF-to-Markdown spatial analysis.

/// Baseline Y tolerance as a fraction of the smaller font size for same-line grouping.
pub(super) const BASELINE_Y_TOLERANCE_FRACTION: f32 = 0.5;
/// Multiplier for baseline line spacing (Q1) to detect paragraph breaks.
pub(super) const PARAGRAPH_GAP_MULTIPLIER: f32 = 1.5;
/// Font size change threshold (in points) to trigger a paragraph break.
pub(super) const FONT_SIZE_CHANGE_THRESHOLD: f32 = 1.5;
/// Left indent change threshold (in points) to trigger a paragraph break.
pub(super) const LEFT_INDENT_CHANGE_THRESHOLD: f32 = 10.0;
/// Maximum word count for a paragraph to qualify as a heading.
pub(super) const MAX_HEADING_WORD_COUNT: usize = 12;
/// Maximum number of lines for a paragraph to be classified as a list item.
pub(super) const MAX_LIST_ITEM_LINES: usize = 5;
/// Maximum distance multiplier relative to average inter-cluster gap for heading assignment.
pub(super) const MAX_HEADING_DISTANCE_MULTIPLIER: f32 = 2.0;
/// Minimum ratio of heading font size to body font size (heading must be this much larger).
pub const MIN_HEADING_FONT_RATIO: f32 = 1.25;
/// Minimum absolute font-size difference (in points) between heading and body.
pub const MIN_HEADING_FONT_GAP: f32 = 2.0;
/// Fraction of page height to exclude from top (page headers).
pub(super) const PAGE_TOP_MARGIN_FRACTION: f32 = 0.06;
/// Fraction of page height to exclude from bottom (page footers/numbers).
pub(super) const PAGE_BOTTOM_MARGIN_FRACTION: f32 = 0.05;
/// Minimum font size (in points) for a segment to be included in analysis.
/// Segments below this size are likely artifacts (embedded images, symbols, noise).
pub(super) const MIN_FONT_SIZE: f32 = 4.0;
/// Maximum word count for a bold paragraph to be promoted to a section heading.
pub(super) const MAX_BOLD_HEADING_WORD_COUNT: usize = 15;
