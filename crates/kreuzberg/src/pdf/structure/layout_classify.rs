//! Layout-detection-based paragraph classification overrides.
//!
//! When layout detection is enabled, this module applies layout hints
//! to override or augment the font-size-based paragraph classification
//! from the standard markdown pipeline.

use super::geometry::Rect;
use super::types::{LayoutHint, LayoutHintClass, PdfParagraph};

/// Apply layout detection overrides to classified paragraphs.
///
/// Uses two matching strategies:
/// 1. **Spatial matching** (heuristic pages): computes bounding boxes from segment
///    positions and matches by containment overlap.
/// 2. **Proportional matching** (structure tree pages): paragraphs without positional
///    data are matched to hints by estimated vertical position, since both are in
///    reading order.
///
/// Structure-tree headings are preserved: only paragraphs without existing
/// heading classification receive heading overrides from layout detection.
pub(crate) fn apply_layout_overrides(
    paragraphs: &mut [PdfParagraph],
    hints: &[LayoutHint],
    min_confidence: f32,
    min_containment: f32,
    body_font_size: Option<f32>,
) {
    if hints.is_empty() {
        return;
    }

    let has_any_positions = paragraphs.iter().any(|p| compute_paragraph_bbox(p).is_some());

    if has_any_positions {
        apply_spatial_overrides(paragraphs, hints, min_confidence, min_containment, body_font_size);
    } else {
        apply_proportional_overrides(paragraphs, hints, min_confidence);
    }

    tracing::debug!(
        total = paragraphs.len(),
        headings = paragraphs.iter().filter(|p| p.heading_level.is_some()).count(),
        list_items = paragraphs.iter().filter(|p| p.is_list_item).count(),
        code_blocks = paragraphs.iter().filter(|p| p.is_code_block).count(),
        formulas = paragraphs.iter().filter(|p| p.is_formula).count(),
        furniture = paragraphs.iter().filter(|p| p.is_page_furniture).count(),
        "layout overrides applied"
    );
}

/// Spatial matching: match paragraphs to hints by bounding box overlap.
///
/// Uses a two-tier strategy:
/// 1. **2D containment** (intersection_area / paragraph_area): best for paragraphs
///    that horizontally overlap with the layout hint.
/// 2. **Vertical-only overlap** (vertical_intersection / paragraph_height): fallback
///    for paragraphs where horizontal alignment differs (e.g., centered text vs
///    left-aligned detection box).
///
/// The vertical fallback requires higher confidence to reduce false positives.
fn apply_spatial_overrides(
    paragraphs: &mut [PdfParagraph],
    hints: &[LayoutHint],
    min_confidence: f32,
    min_containment: f32,
    body_font_size: Option<f32>,
) {
    let confident_hints: Vec<&LayoutHint> = hints.iter().filter(|h| h.confidence >= min_confidence).collect();

    for (para_idx, para) in paragraphs.iter_mut().enumerate() {
        let para_bbox = match compute_paragraph_bbox(para) {
            Some(bbox) => bbox,
            None => continue,
        };

        if para_bbox.height() <= 0.0 {
            continue;
        }

        let best_2d = confident_hints
            .iter()
            .filter_map(|hint| {
                let hint_rect = Rect::from_lbrt(hint.left, hint.bottom, hint.right, hint.top);
                let containment = para_bbox.intersection_over_self(&hint_rect);
                if containment >= min_containment {
                    Some((*hint, containment))
                } else {
                    None
                }
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));

        if let Some((hint, containment)) = best_2d {
            tracing::trace!(
                para_idx,
                hint_class = ?hint.class,
                containment,
                "spatial hint match"
            );
            apply_hint_to_paragraph(para, hint, body_font_size);
        }
    }
}

/// Proportional matching: match paragraphs to hints using range-overlap.
///
/// Structure tree paragraphs have no positional data but are in reading order
/// (top-to-bottom). Layout hints have PDF coordinates with known bounding boxes.
///
/// Strategy:
/// 1. Sort hints by vertical position (top-to-bottom in reading order).
/// 2. Each paragraph occupies a fractional range `[i/n, (i+1)/n]` of the page.
/// 3. Each hint occupies a fractional range `[(page_height - top)/page_height, (page_height - bottom)/page_height]`.
/// 4. Match each paragraph to the hint with the most fractional overlap.
///
/// Structure tree paragraphs lack positional data so we cannot do spatial matching.
/// We use conservative fractional-overlap matching: each paragraph is assigned a
/// fraction of the page [i/n, (i+1)/n] and matched against hint bounding boxes
/// converted to page fractions. Only high-confidence, high-overlap matches are applied.
fn apply_proportional_overrides(paragraphs: &mut [PdfParagraph], hints: &[LayoutHint], min_confidence: f32) {
    let n = paragraphs.len();
    if n == 0 {
        return;
    }

    let confident_hints: Vec<&LayoutHint> = hints.iter().filter(|h| h.confidence >= min_confidence).collect();
    if confident_hints.is_empty() {
        return;
    }

    let page_height = hints.iter().map(|h| h.top).fold(0.0_f32, f32::max);
    if page_height <= 0.0 {
        return;
    }

    tracing::debug!(
        paragraph_count = n,
        hint_count = confident_hints.len(),
        page_height,
        "Proportional matching: structure tree paragraphs without positions"
    );

    let hint_ranges: Vec<(f32, f32, &LayoutHint)> = confident_hints
        .iter()
        .map(|h| {
            let frac_start = (page_height - h.top) / page_height;
            let frac_end = (page_height - h.bottom) / page_height;
            (frac_start.max(0.0), frac_end.min(1.0), *h)
        })
        .collect();

    for (i, para) in paragraphs.iter_mut().enumerate() {
        let para_start = i as f32 / n as f32;
        let para_end = (i as f32 + 1.0) / n as f32;

        let best = hint_ranges
            .iter()
            .filter_map(|&(h_start, h_end, hint)| {
                let overlap_start = para_start.max(h_start);
                let overlap_end = para_end.min(h_end);
                let overlap = (overlap_end - overlap_start).max(0.0);
                if overlap > 0.0 { Some((hint, overlap)) } else { None }
            })
            .max_by(|a, b| a.1.total_cmp(&b.1));

        if let Some((hint, overlap)) = best {
            let para_span = para_end - para_start;
            let overlap_frac = if para_span > 0.0 { overlap / para_span } else { 0.0 };

            match hint.class {
                LayoutHintClass::PageHeader if i == 0 && overlap_frac > 0.25 => {
                    apply_hint_to_paragraph(para, hint, None);
                }
                LayoutHintClass::PageFooter if i == n - 1 && overlap_frac > 0.25 => {
                    apply_hint_to_paragraph(para, hint, None);
                }
                LayoutHintClass::SectionHeader | LayoutHintClass::Title
                    if para.heading_level.is_none() && !para.is_code_block && overlap_frac > 0.3 =>
                {
                    para.is_list_item = false;
                    let text: String = if !para.text.is_empty() {
                        para.text.clone()
                    } else {
                        para.lines
                            .iter()
                            .flat_map(|l| l.segments.iter())
                            .map(|s| s.text.as_str())
                            .collect::<Vec<_>>()
                            .join(" ")
                    };
                    let word_count = text.split_whitespace().count();
                    if word_count <= super::constants::MAX_HEADING_WORD_COUNT && !is_separator_text(&text) {
                        let level = infer_heading_level_from_text(&text, hint.class);
                        para.heading_level = Some(level);
                        para.layout_class = Some(hint.class);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Check if text is a separator/filler line (dashes, underscores, tildes, etc.)
/// that should never be classified as a heading.
pub(super) fn is_separator_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    let total = trimmed.chars().count();
    let alnum = trimmed.chars().filter(|c| c.is_alphanumeric()).count();
    if alnum == 0 {
        return true;
    }
    total >= 6 && (alnum as f64 / total as f64) < 0.15
}

/// Infer heading level from section numbering in the text.
///
/// Academic papers use numbering to indicate heading depth:
/// - "1 Introduction" → H2 (top-level section)
/// - "3.2 AI models" → H3 (sub-section)
/// - "3.2.1 Details" → H4 (sub-sub-section)
/// - "Layout Analysis Model" (no number) → H2 (default for SectionHeader)
pub(super) fn infer_heading_level_from_text(text: &str, hint_class: LayoutHintClass) -> u8 {
    if hint_class == LayoutHintClass::Title {
        return 1;
    }

    let trimmed = text.trim();

    let first_char = trimmed.chars().next().unwrap_or(' ');
    let is_alpha_prefix = first_char.is_ascii_alphabetic()
        && trimmed.len() >= 2
        && matches!(trimmed.as_bytes().get(1), Some(b'.' | b')' | b' '));

    let numbering_end = if is_alpha_prefix {
        let after_letter = &trimmed[1..];
        let rest_end = after_letter
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(0);
        1 + rest_end
    } else {
        trimmed.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(0)
    };

    if numbering_end == 0 {
        return 2;
    }

    let numbering = &trimmed[..numbering_end];
    let dot_count = numbering.chars().filter(|&c| c == '.').count();

    let effective_dots = if numbering.ends_with('.') {
        dot_count.saturating_sub(1)
    } else {
        dot_count
    };

    match effective_dots {
        0 => 2,
        1 => 3,
        _ => 4,
    }
}

/// Apply a single hint's classification to a paragraph.
///
/// `body_font_size`: when provided, used to guard against promoting body-text-sized
/// paragraphs to headings (unnumbered SectionHeader at body font size is likely a
/// false positive from the layout model).
pub(super) fn apply_hint_to_paragraph(para: &mut PdfParagraph, hint: &LayoutHint, body_font_size: Option<f32>) {
    tracing::debug!(
        hint_class = ?hint.class,
        confidence = hint.confidence,
        old_heading = ?para.heading_level,
        "applying layout hint"
    );

    para.layout_class = Some(hint.class);

    let para_text: String = if !para.text.is_empty() {
        para.text.clone()
    } else {
        para.lines
            .iter()
            .flat_map(|l| l.segments.iter())
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    };
    let word_count = para_text.split_whitespace().count();
    let is_sep = is_separator_text(&para_text);

    match hint.class {
        LayoutHintClass::Title
            if !is_sep
                && (para.heading_level.is_none() || hint.confidence >= 0.7)
                && word_count <= super::constants::MAX_HEADING_WORD_COUNT =>
        {
            para.heading_level = Some(1);
        }
        LayoutHintClass::SectionHeader if !is_sep && (para.heading_level.is_none() || hint.confidence >= 0.7) => {
            let trimmed = para_text.trim();
            let too_long = word_count > super::constants::MAX_HEADING_WORD_COUNT;
            let ends_period = trimmed.ends_with('.') && !super::classify::is_section_pattern(trimmed);
            let ends_colon = trimmed.ends_with(':');
            let is_figure = super::regions::looks_like_figure_label(trimmed);
            let is_monospace = if !para.text.is_empty() {
                para.is_monospace_hint()
            } else {
                para.lines.iter().all(|l| l.is_monospace)
            };
            let text_level = infer_heading_level_from_text(&para_text, hint.class);
            let near_body = body_font_size.is_some_and(|body| {
                body > 0.0 && para.dominant_font_size >= body - 1.5 && para.dominant_font_size <= body + 0.5
            });
            let is_unnumbered = text_level == 2;
            let high_confidence_bold = hint.confidence >= 0.7 && para.is_bold;
            let looks_like_sentence = trimmed.ends_with('.') && word_count > 8;
            let body_size_guard = near_body && is_unnumbered && (!high_confidence_bold || looks_like_sentence);
            if !too_long && !ends_period && !ends_colon && !is_figure && !is_monospace && !body_size_guard {
                para.heading_level = Some(text_level);
            }
        }
        LayoutHintClass::Code => {
            let is_prose = {
                let sentence_endings = para_text
                    .chars()
                    .filter(|&c| c == '.' || c == '!' || c == '?' || c == ',')
                    .count();
                let syntax_chars = para_text
                    .chars()
                    .filter(|c| {
                        matches!(
                            c,
                            '{' | '}' | '(' | ')' | '[' | ']' | ';' | '=' | '<' | '>' | '|' | '@' | '#' | '$'
                        )
                    })
                    .count();
                let syntax_ratio = if para_text.is_empty() {
                    0.0
                } else {
                    syntax_chars as f64 / para_text.len() as f64
                };
                sentence_endings >= 2 && syntax_ratio < 0.03 && word_count > 15
            };
            if !is_prose {
                para.is_code_block = true;
                para.heading_level = None;
            }
        }
        LayoutHintClass::Formula => {
            para.is_formula = true;
            para.heading_level = None;
        }
        LayoutHintClass::ListItem => {
            para.is_list_item = true;
        }
        LayoutHintClass::PageHeader | LayoutHintClass::PageFooter => {
            para.is_page_furniture = true;
        }
        LayoutHintClass::Picture => {
            para.is_page_furniture = true;
            para.heading_level = None;
        }
        LayoutHintClass::Text | LayoutHintClass::Caption | LayoutHintClass::Footnote
            if para.heading_level.is_some() && hint.confidence >= 0.7 =>
        {
            tracing::trace!(
                ?hint.class,
                hint_confidence = hint.confidence,
                old_heading_level = ?para.heading_level,
                "Demoting heading: layout model classifies as body text"
            );
            para.heading_level = None;
        }
        _ => {}
    }
}

/// Compute a paragraph's bounding box from its line segments' positional data.
///
/// Returns `None` if the paragraph has no segments with valid positional data.
///
/// In PDF coordinates (y=0 at bottom, y increases upward):
/// - `seg.y` / `seg.baseline_y` is the text baseline (near the bottom of glyphs).
/// - Text extends UPWARD from the baseline by roughly the ascent (~80% of font size).
/// - Text extends DOWNWARD from the baseline by the descent (~20% of font size).
///
/// For layout detection matching, we approximate the visual text extent as:
/// - top = baseline + height (covers ascenders)
/// - bottom = baseline (descent is small and usually within the layout hint's margin)
fn compute_paragraph_bbox(para: &PdfParagraph) -> Option<Rect> {
    if let Some((left, bottom, right, top)) = para.block_bbox
        && right > left
        && top > bottom
    {
        return Some(Rect::from_lbrt(left, bottom, right, top));
    }

    let mut left = f32::MAX;
    let mut right = f32::MIN;
    let mut bottom = f32::MAX;
    let mut top = f32::MIN;
    let mut has_data = false;

    for line in &para.lines {
        for seg in &line.segments {
            if seg.x == 0.0 && seg.width == 0.0 && seg.y == 0.0 && seg.height == 0.0 {
                continue;
            }
            has_data = true;
            left = left.min(seg.x);
            right = right.max(seg.x + seg.width);
            top = top.max(seg.y + seg.height);
            bottom = bottom.min(seg.y);
        }
    }

    if has_data {
        Some(Rect::from_lbrt(left, bottom, right, top))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::hierarchy::SegmentData;
    use crate::pdf::structure::types::PdfLine;

    fn make_segment(text: &str, x: f32, y: f32, width: f32, height: f32) -> SegmentData {
        SegmentData {
            text: text.to_string(),
            x,
            y,
            width,
            height,
            font_size: 12.0,
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            baseline_y: y,
            assigned_role: None,
        }
    }

    fn make_line_at(segments: Vec<SegmentData>, baseline_y: f32) -> PdfLine {
        PdfLine {
            segments,
            baseline_y,
            dominant_font_size: 12.0,
            is_bold: false,
            is_monospace: false,
        }
    }

    fn make_line(segments: Vec<SegmentData>) -> PdfLine {
        make_line_at(segments, 700.0)
    }

    fn make_para(x: f32, y: f32, width: f32, height: f32) -> PdfParagraph {
        PdfParagraph {
            text: String::new(),
            lines: vec![make_line(vec![make_segment("text", x, y, width, height)])],
            dominant_font_size: 12.0,
            heading_level: None,
            is_bold: false,
            is_list_item: false,
            is_code_block: false,
            is_formula: false,
            is_page_furniture: false,
            layout_class: None,
            caption_for: None,
            block_bbox: None,
        }
    }

    fn make_hint(class: LayoutHintClass, confidence: f32, left: f32, bottom: f32, right: f32, top: f32) -> LayoutHint {
        LayoutHint {
            class,
            confidence,
            left,
            bottom,
            right,
            top,
        }
    }

    #[test]
    fn test_title_override() {
        let mut paragraphs = vec![make_para(50.0, 750.0, 500.0, 20.0)];
        let hints = vec![make_hint(LayoutHintClass::Title, 0.9, 40.0, 745.0, 560.0, 775.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, Some(1));
        assert_eq!(paragraphs[0].layout_class, Some(LayoutHintClass::Title));
    }

    #[test]
    fn test_section_header_override() {
        let mut paragraphs = vec![make_para(50.0, 600.0, 300.0, 16.0)];
        let hints = vec![make_hint(
            LayoutHintClass::SectionHeader,
            0.85,
            40.0,
            598.0,
            400.0,
            620.0,
        )];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, Some(2));
    }

    #[test]
    fn test_low_confidence_ignored() {
        let mut paragraphs = vec![make_para(50.0, 750.0, 500.0, 20.0)];
        let hints = vec![make_hint(LayoutHintClass::Title, 0.3, 40.0, 745.0, 560.0, 775.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, None);
        assert_eq!(paragraphs[0].layout_class, None);
    }

    #[test]
    fn test_existing_heading_overridden_by_high_confidence() {
        let mut paragraphs = vec![make_para(50.0, 750.0, 500.0, 20.0)];
        paragraphs[0].heading_level = Some(3);
        let hints = vec![make_hint(
            LayoutHintClass::SectionHeader,
            0.9,
            40.0,
            745.0,
            560.0,
            775.0,
        )];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, Some(2));
    }

    #[test]
    fn test_existing_heading_preserved_low_confidence() {
        let mut paragraphs = vec![make_para(50.0, 750.0, 500.0, 20.0)];
        paragraphs[0].heading_level = Some(3);
        let hints = vec![make_hint(
            LayoutHintClass::SectionHeader,
            0.6,
            40.0,
            745.0,
            560.0,
            775.0,
        )];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, Some(3));
    }

    #[test]
    fn test_empty_hints() {
        let mut paragraphs = vec![make_para(50.0, 750.0, 500.0, 20.0)];
        apply_layout_overrides(&mut paragraphs, &[], 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, None);
    }

    #[test]
    fn test_intersection_over_self_full() {
        let hint = Rect::from_lbrt(0.0, 0.0, 612.0, 792.0);
        let para = Rect::from_lbrt(50.0, 100.0, 550.0, 200.0);
        let containment = para.intersection_over_self(&hint);
        assert!(
            (containment - 1.0).abs() < 0.01,
            "Full containment expected: {}",
            containment
        );
    }

    #[test]
    fn test_intersection_over_self_none() {
        let hint = Rect::from_lbrt(0.0, 500.0, 100.0, 600.0);
        let para = Rect::from_lbrt(200.0, 100.0, 500.0, 200.0);
        let containment = para.intersection_over_self(&hint);
        assert!(
            (containment - 0.0).abs() < 0.01,
            "No containment expected: {}",
            containment
        );
    }

    #[test]
    fn test_infer_heading_level_title() {
        assert_eq!(
            infer_heading_level_from_text("Docling Report", LayoutHintClass::Title),
            1
        );
    }

    #[test]
    fn test_infer_heading_level_top_section() {
        assert_eq!(
            infer_heading_level_from_text("3 Processing pipeline", LayoutHintClass::SectionHeader),
            2
        );
    }

    #[test]
    fn test_infer_heading_level_subsection() {
        assert_eq!(
            infer_heading_level_from_text("3.2 AI models", LayoutHintClass::SectionHeader),
            3
        );
    }

    #[test]
    fn test_infer_heading_level_subsubsection() {
        assert_eq!(
            infer_heading_level_from_text("3.2.1 Details", LayoutHintClass::SectionHeader),
            4
        );
    }

    #[test]
    fn test_infer_heading_level_trailing_dot() {
        assert_eq!(
            infer_heading_level_from_text("3. Processing", LayoutHintClass::SectionHeader),
            2
        );
    }

    #[test]
    fn test_infer_heading_level_no_number() {
        assert_eq!(
            infer_heading_level_from_text("Layout Analysis Model", LayoutHintClass::SectionHeader),
            2
        );
    }

    #[test]
    fn test_no_positional_data_proportional_applies_page_furniture() {
        let mut paragraphs = vec![PdfParagraph {
            text: String::new(),
            lines: vec![make_line(vec![make_segment("text", 0.0, 0.0, 0.0, 0.0)])],
            dominant_font_size: 12.0,
            heading_level: None,
            is_bold: false,
            is_list_item: false,
            is_code_block: false,
            is_formula: false,
            is_page_furniture: false,
            layout_class: None,
            caption_for: None,
            block_bbox: None,
        }];

        let hints = vec![make_hint(LayoutHintClass::Title, 0.9, 40.0, 0.0, 560.0, 760.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, Some(1));
        assert_eq!(paragraphs[0].layout_class, Some(LayoutHintClass::Title));

        paragraphs[0].heading_level = None;
        paragraphs[0].layout_class = None;

        let hints = vec![make_hint(LayoutHintClass::PageHeader, 0.9, 40.0, 0.0, 560.0, 760.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert!(paragraphs[0].is_page_furniture);
        assert_eq!(paragraphs[0].layout_class, Some(LayoutHintClass::PageHeader));
    }

    #[test]
    fn test_separator_pure_dashes() {
        assert!(is_separator_text("----------"));
    }

    #[test]
    fn test_separator_underscores() {
        assert!(is_separator_text("___________"));
    }

    #[test]
    fn test_separator_mixed_with_few_alnum() {
        assert!(is_separator_text("------- M ---------"));
    }

    #[test]
    fn test_separator_empty_string() {
        assert!(!is_separator_text(""));
        assert!(!is_separator_text("   "));
    }

    #[test]
    fn test_separator_normal_text() {
        assert!(!is_separator_text("Hello World"));
    }

    #[test]
    fn test_separator_short_symbols() {
        assert!(is_separator_text("---"));
    }

    #[test]
    fn test_infer_heading_level_alpha_prefix() {
        assert_eq!(
            infer_heading_level_from_text("A. Proofs", LayoutHintClass::SectionHeader),
            2
        );
    }

    #[test]
    fn test_infer_heading_level_alpha_subsection() {
        assert_eq!(
            infer_heading_level_from_text("A.1 Details", LayoutHintClass::SectionHeader),
            3
        );
    }

    #[test]
    fn test_infer_heading_level_deep_subsection() {
        assert_eq!(
            infer_heading_level_from_text("1.2.3.4 Very deep", LayoutHintClass::SectionHeader),
            4
        );
    }

    #[test]
    fn test_code_override() {
        let mut paragraphs = vec![make_para(50.0, 600.0, 300.0, 16.0)];
        paragraphs[0].heading_level = Some(2);
        let hints = vec![make_hint(LayoutHintClass::Code, 0.9, 40.0, 598.0, 400.0, 620.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert!(paragraphs[0].is_code_block);
        assert_eq!(paragraphs[0].heading_level, None);
    }

    #[test]
    fn test_code_override_rejects_prose() {
        let mut para = make_para(50.0, 600.0, 300.0, 16.0);
        para.text = "Duis autem vel eum iriure dolor in hendrerit in vulputate velit esse molestie consequat. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore.".to_string();
        let mut paragraphs = vec![para];
        let hints = vec![make_hint(LayoutHintClass::Code, 0.9, 40.0, 598.0, 400.0, 620.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert!(
            !paragraphs[0].is_code_block,
            "Prose text should not be classified as code"
        );
    }

    #[test]
    fn test_code_override_accepts_real_code() {
        let mut para = make_para(50.0, 600.0, 300.0, 16.0);
        para.text = "function add(a, b) { return a + b; }".to_string();
        let mut paragraphs = vec![para];
        let hints = vec![make_hint(LayoutHintClass::Code, 0.9, 40.0, 598.0, 400.0, 620.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert!(
            paragraphs[0].is_code_block,
            "Code-like text should be classified as code"
        );
    }

    #[test]
    fn test_formula_override() {
        let mut paragraphs = vec![make_para(50.0, 600.0, 300.0, 16.0)];
        let hints = vec![make_hint(LayoutHintClass::Formula, 0.9, 40.0, 598.0, 400.0, 620.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert!(paragraphs[0].is_formula);
    }

    #[test]
    fn test_list_item_override() {
        let mut paragraphs = vec![make_para(50.0, 600.0, 300.0, 16.0)];
        let hints = vec![make_hint(LayoutHintClass::ListItem, 0.9, 40.0, 598.0, 400.0, 620.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert!(paragraphs[0].is_list_item);
    }

    #[test]
    fn test_body_text_demotes_heading() {
        let mut paragraphs = vec![make_para(50.0, 600.0, 300.0, 16.0)];
        paragraphs[0].heading_level = Some(2);
        let hints = vec![make_hint(LayoutHintClass::Text, 0.9, 40.0, 598.0, 400.0, 620.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, None);
    }

    #[test]
    fn test_body_text_low_confidence_preserves_heading() {
        let mut paragraphs = vec![make_para(50.0, 600.0, 300.0, 16.0)];
        paragraphs[0].heading_level = Some(2);
        let hints = vec![make_hint(LayoutHintClass::Text, 0.6, 40.0, 598.0, 400.0, 620.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert_eq!(paragraphs[0].heading_level, Some(2));
    }

    #[test]
    fn test_page_footer_override() {
        let mut paragraphs = vec![make_para(50.0, 600.0, 300.0, 16.0)];
        let hints = vec![make_hint(LayoutHintClass::PageFooter, 0.9, 40.0, 598.0, 400.0, 620.0)];
        apply_layout_overrides(&mut paragraphs, &hints, 0.5, 0.5, None);
        assert!(paragraphs[0].is_page_furniture);
    }

    #[test]
    fn test_separator_text_not_promoted_to_heading() {
        let mut para = PdfParagraph {
            text: String::new(),
            lines: vec![make_line(vec![make_segment("----------", 50.0, 600.0, 300.0, 16.0)])],
            dominant_font_size: 12.0,
            heading_level: None,
            is_bold: false,
            is_list_item: false,
            is_code_block: false,
            is_formula: false,
            is_page_furniture: false,
            layout_class: None,
            caption_for: None,
            block_bbox: None,
        };
        let hint = make_hint(LayoutHintClass::SectionHeader, 0.9, 40.0, 598.0, 400.0, 620.0);
        apply_hint_to_paragraph(&mut para, &hint, None);
        assert_eq!(para.heading_level, None);
    }

    #[test]
    fn test_compute_paragraph_bbox_no_positional_data() {
        let para = PdfParagraph {
            text: String::new(),
            lines: vec![make_line(vec![make_segment("text", 0.0, 0.0, 0.0, 0.0)])],
            dominant_font_size: 12.0,
            heading_level: None,
            is_bold: false,
            is_list_item: false,
            is_code_block: false,
            is_formula: false,
            is_page_furniture: false,
            layout_class: None,
            caption_for: None,
            block_bbox: None,
        };
        assert!(compute_paragraph_bbox(&para).is_none());
    }

    #[test]
    fn test_compute_paragraph_bbox_with_block_bbox() {
        let para = PdfParagraph {
            text: String::new(),
            lines: vec![make_line(vec![make_segment("text", 0.0, 0.0, 0.0, 0.0)])],
            dominant_font_size: 12.0,
            heading_level: None,
            is_bold: false,
            is_list_item: false,
            is_code_block: false,
            is_formula: false,
            is_page_furniture: false,
            layout_class: None,
            caption_for: None,
            block_bbox: Some((50.0, 100.0, 400.0, 120.0)),
        };
        let bbox = compute_paragraph_bbox(&para).unwrap();
        assert!((bbox.left - 50.0).abs() < f32::EPSILON);
        assert!((bbox.y_min - 100.0).abs() < f32::EPSILON);
        assert!((bbox.right - 400.0).abs() < f32::EPSILON);
        assert!((bbox.y_max - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_compute_paragraph_bbox_from_segments() {
        let para = PdfParagraph {
            text: String::new(),
            lines: vec![
                make_line_at(vec![make_segment("A", 50.0, 700.0, 100.0, 12.0)], 700.0),
                make_line_at(vec![make_segment("B", 60.0, 680.0, 120.0, 14.0)], 680.0),
            ],
            dominant_font_size: 12.0,
            heading_level: None,
            is_bold: false,
            is_list_item: false,
            is_code_block: false,
            is_formula: false,
            is_page_furniture: false,
            layout_class: None,
            caption_for: None,
            block_bbox: None,
        };
        let bbox = compute_paragraph_bbox(&para).unwrap();
        assert!((bbox.left - 50.0).abs() < f32::EPSILON);
        assert!((bbox.y_min - 680.0).abs() < f32::EPSILON);
        assert!((bbox.right - 180.0).abs() < f32::EPSILON);
        assert!((bbox.y_max - 712.0).abs() < f32::EPSILON);
    }
}
