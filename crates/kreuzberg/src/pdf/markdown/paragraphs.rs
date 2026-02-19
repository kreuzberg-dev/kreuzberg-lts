//! Paragraph building from lines using vertical gaps and formatting changes.

use super::constants::{
    FONT_SIZE_CHANGE_THRESHOLD, LEFT_INDENT_CHANGE_THRESHOLD, MAX_LIST_ITEM_LINES, PARAGRAPH_GAP_MULTIPLIER,
};
use super::types::{PdfLine, PdfParagraph};

/// Group lines into paragraphs based on vertical gaps, font size changes, and indentation.
pub(super) fn lines_to_paragraphs(lines: Vec<PdfLine>) -> Vec<PdfParagraph> {
    if lines.is_empty() {
        return Vec::new();
    }

    if lines.len() == 1 {
        return vec![finalize_paragraph(lines)];
    }

    // Compute baseline line spacing for paragraph break detection.
    let avg_font_size = lines.iter().map(|l| l.dominant_font_size).sum::<f32>() / lines.len() as f32;

    let mut spacings: Vec<f32> = Vec::new();
    for pair in lines.windows(2) {
        let gap = (pair[1].baseline_y - pair[0].baseline_y).abs();
        if gap > avg_font_size * 0.4 {
            spacings.push(gap);
        }
    }

    let base_spacing = if spacings.is_empty() {
        avg_font_size
    } else {
        spacings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // Use 25th percentile (Q1) for robustness against outlier-tight spacings
        // from superscripts/subscripts, while staying conservative enough to work
        // with small sample sizes (unlike median which can pick a gap spacing).
        spacings[spacings.len() / 4]
    };

    let paragraph_gap_threshold = base_spacing * PARAGRAPH_GAP_MULTIPLIER;

    let mut paragraphs: Vec<PdfParagraph> = Vec::new();
    let mut current_lines: Vec<PdfLine> = vec![lines[0].clone()];

    for line in lines.into_iter().skip(1) {
        let prev = current_lines.last().unwrap();

        let vertical_gap = (line.baseline_y - prev.baseline_y).abs();
        let font_size_change = (line.dominant_font_size - prev.dominant_font_size).abs();

        let prev_left = prev.segments.first().map(|s| s.x).unwrap_or(0.0);
        let curr_left = line.segments.first().map(|s| s.x).unwrap_or(0.0);
        let indent_change = (curr_left - prev_left).abs();

        let has_significant_gap = vertical_gap > paragraph_gap_threshold;
        let has_some_gap = vertical_gap > base_spacing * 0.8;
        let has_font_change = font_size_change > FONT_SIZE_CHANGE_THRESHOLD;
        let has_indent_change = indent_change > LEFT_INDENT_CHANGE_THRESHOLD;

        // Force paragraph break if next line starts with a list prefix
        let next_starts_with_list = line
            .segments
            .first()
            .and_then(|s| s.text.split_whitespace().next())
            .map(is_list_prefix)
            .unwrap_or(false);

        let is_paragraph_break =
            has_significant_gap || (has_some_gap && (has_font_change || has_indent_change)) || next_starts_with_list;

        if is_paragraph_break {
            paragraphs.push(finalize_paragraph(current_lines));
            current_lines = vec![line];
        } else {
            current_lines.push(line);
        }
    }

    if !current_lines.is_empty() {
        paragraphs.push(finalize_paragraph(current_lines));
    }

    paragraphs
}

/// Build a PdfParagraph from a set of lines.
fn finalize_paragraph(lines: Vec<PdfLine>) -> PdfParagraph {
    let dominant_font_size = if lines.is_empty() {
        0.0
    } else {
        let mut fs_counts: Vec<(i32, usize)> = Vec::new();
        for l in &lines {
            let key = (l.dominant_font_size * 2.0).round() as i32;
            if let Some(entry) = fs_counts.iter_mut().find(|(k, _)| *k == key) {
                entry.1 += 1;
            } else {
                fs_counts.push((key, 1));
            }
        }
        fs_counts.sort_by(|a, b| b.1.cmp(&a.1));
        fs_counts[0].0 as f32 / 2.0
    };

    let bold_count = lines.iter().filter(|l| l.is_bold).count();
    let italic_count = lines.iter().filter(|l| l.is_italic).count();
    let majority = lines.len().div_ceil(2);

    // Detect list items: first segment of first line starts with bullet or number prefix
    let first_text = lines
        .first()
        .and_then(|l| l.segments.first())
        .map(|s| s.text.as_str())
        .unwrap_or("");
    let first_word = first_text.split_whitespace().next().unwrap_or("");
    let is_list_item = lines.len() <= MAX_LIST_ITEM_LINES && is_list_prefix(first_word);

    // Detect code blocks: all lines must be monospace (and there must be at least one line)
    let is_code_block = !lines.is_empty() && lines.iter().all(|l| l.is_monospace);

    PdfParagraph {
        dominant_font_size,
        heading_level: None,
        is_bold: bold_count >= majority,
        is_italic: italic_count >= majority,
        is_list_item,
        is_code_block,
        lines,
    }
}

/// Merge consecutive body-text paragraphs that are continuations of the same logical paragraph.
///
/// Two consecutive paragraphs are merged if:
/// - Both are body text (no heading_level, not is_list_item)
/// - The first paragraph doesn't end with sentence-ending punctuation
/// - Font sizes are within 2pt of each other
pub(super) fn merge_continuation_paragraphs(paragraphs: &mut Vec<PdfParagraph>) {
    if paragraphs.len() < 2 {
        return;
    }

    let mut i = 0;
    while i + 1 < paragraphs.len() {
        let should_merge = {
            let current = &paragraphs[i];
            let next = &paragraphs[i + 1];

            // Both must be body text
            current.heading_level.is_none()
                && next.heading_level.is_none()
                && !current.is_list_item
                && !next.is_list_item
                // Font sizes close enough
                && (current.dominant_font_size - next.dominant_font_size).abs() < 2.0
                // Current paragraph doesn't end with sentence-ending punctuation
                && !ends_with_sentence_terminator(current)
        };

        if should_merge {
            let next = paragraphs.remove(i + 1);
            paragraphs[i].lines.extend(next.lines);
        } else {
            i += 1;
        }
    }
}

/// Check if a paragraph's last line ends with sentence-terminating punctuation.
fn ends_with_sentence_terminator(para: &PdfParagraph) -> bool {
    let last_text = para
        .lines
        .last()
        .and_then(|l| l.segments.last())
        .map(|s| s.text.trim_end())
        .unwrap_or("");
    matches!(last_text.chars().last(), Some('.' | '?' | '!' | ':' | ';'))
}

/// Check if text looks like a list item prefix.
fn is_list_prefix(text: &str) -> bool {
    let trimmed = text.trim();
    // Bullet characters: hyphen, asterisk, bullet, en dash, em dash
    if matches!(trimmed, "-" | "*" | "\u{2022}" | "\u{2013}" | "\u{2014}") {
        return true;
    }
    let bytes = trimmed.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    // Numbered: 1. 2) etc.
    let digit_end = bytes.iter().position(|&b| !b.is_ascii_digit()).unwrap_or(bytes.len());
    if digit_end > 0 && digit_end < bytes.len() {
        let suffix = bytes[digit_end];
        if suffix == b'.' || suffix == b')' {
            return true;
        }
    }
    // Alphabetic: a. b) A. B) (single letter + period/paren)
    if bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && (bytes[1] == b'.' || bytes[1] == b')') {
        return true;
    }
    // Roman numerals: i. ii. iii. iv. v. vi. I. II. III. IV. V. VI. etc.
    if trimmed.ends_with('.') || trimmed.ends_with(')') {
        let prefix = &trimmed[..trimmed.len() - 1];
        if is_roman_numeral(prefix) {
            return true;
        }
    }
    false
}

/// Check if text is a roman numeral (i-xii or I-XII range).
fn is_roman_numeral(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let lower = s.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "i" | "ii" | "iii" | "iv" | "v" | "vi" | "vii" | "viii" | "ix" | "x" | "xi" | "xii"
    )
}

#[cfg(test)]
mod tests {
    use crate::pdf::hierarchy::SegmentData;

    use super::*;

    fn plain_segment(text: &str, x: f32, baseline_y: f32, width: f32, font_size: f32) -> SegmentData {
        SegmentData {
            text: text.to_string(),
            x,
            y: baseline_y,
            width,
            height: font_size,
            font_size,
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            baseline_y,
        }
    }

    fn make_line(segments: Vec<SegmentData>, baseline_y: f32, font_size: f32) -> PdfLine {
        PdfLine {
            segments,
            baseline_y,
            y_top: baseline_y - font_size,
            y_bottom: baseline_y,
            dominant_font_size: font_size,
            is_bold: false,
            is_italic: false,
            is_monospace: false,
        }
    }

    #[test]
    fn test_lines_to_paragraphs_single_line() {
        let lines = vec![make_line(
            vec![plain_segment("Hello world", 10.0, 700.0, 80.0, 12.0)],
            700.0,
            12.0,
        )];
        let paragraphs = lines_to_paragraphs(lines);
        assert_eq!(paragraphs.len(), 1);
    }

    #[test]
    fn test_lines_to_paragraphs_gap_detection() {
        let lines = vec![
            make_line(vec![plain_segment("Para 1", 10.0, 700.0, 60.0, 12.0)], 700.0, 12.0),
            make_line(
                vec![plain_segment("Still para 1", 10.0, 686.0, 80.0, 12.0)],
                686.0,
                12.0,
            ),
            // Big gap
            make_line(vec![plain_segment("Para 2", 10.0, 640.0, 60.0, 12.0)], 640.0, 12.0),
        ];
        let paragraphs = lines_to_paragraphs(lines);
        assert_eq!(paragraphs.len(), 2);
    }

    #[test]
    fn test_lines_to_paragraphs_empty() {
        let paragraphs = lines_to_paragraphs(vec![]);
        assert!(paragraphs.is_empty());
    }

    #[test]
    fn test_list_item_detection() {
        let lines = vec![make_line(
            vec![plain_segment("- Item text", 10.0, 700.0, 80.0, 12.0)],
            700.0,
            12.0,
        )];
        let paragraphs = lines_to_paragraphs(lines);
        assert_eq!(paragraphs.len(), 1);
        assert!(paragraphs[0].is_list_item);
    }

    #[test]
    fn test_numbered_list_detection() {
        let lines = vec![make_line(
            vec![plain_segment("1. First item", 10.0, 700.0, 80.0, 12.0)],
            700.0,
            12.0,
        )];
        let paragraphs = lines_to_paragraphs(lines);
        assert!(paragraphs[0].is_list_item);
    }

    #[test]
    fn test_not_list_item() {
        let lines = vec![make_line(
            vec![plain_segment("Normal text", 10.0, 700.0, 80.0, 12.0)],
            700.0,
            12.0,
        )];
        let paragraphs = lines_to_paragraphs(lines);
        assert!(!paragraphs[0].is_list_item);
    }
}
