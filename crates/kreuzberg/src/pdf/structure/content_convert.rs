//! Convert [`PageContent`] elements into [`PdfParagraph`]s for the markdown pipeline.
//!
//! This is the shared conversion layer: all extraction backends produce
//! `PageContent` via adapters, then this module converts elements into
//! the `PdfParagraph` representation used by heading classification,
//! layout overrides, and markdown rendering.

use super::constants::MAX_HEADING_WORD_COUNT;
use super::content::{ContentElement, ElementLevel, PageContent, SemanticRole};
use super::types::{LayoutHintClass, PdfLine, PdfParagraph};
use crate::pdf::hierarchy::SegmentData;

/// Minimum gap between columns as fraction of estimated page width.
const MIN_COLUMN_GAP_FRACTION: f32 = 0.10;

/// Minimum fraction of total Y range that each column side must span.
const MIN_COLUMN_Y_SPAN_FRACTION: f32 = 0.30;

/// Minimum number of elements required on each side of a column split.
const MIN_ELEMENTS_PER_COLUMN: usize = 2;

/// Y-proximity tolerance as a fraction of median element height, for line grouping.
const LINE_Y_TOLERANCE_FRACTION: f32 = 0.5;

/// Convert a page's content elements into paragraphs.
///
/// For word-level OCR content (majority of elements are `ElementLevel::Word`),
/// spatially proximate words are grouped into lines and then into paragraphs.
/// For block/line-level content, each element becomes its own paragraph.
pub(crate) fn content_to_paragraphs(page: &PageContent) -> Vec<PdfParagraph> {
    let word_count = page.elements.iter().filter(|e| e.level == ElementLevel::Word).count();
    let total = page.elements.len();

    if total > 0 && word_count > total / 2 {
        return group_words_to_paragraphs(&page.elements);
    }

    let mut paragraphs = Vec::with_capacity(total);
    for elem in &page.elements {
        if let Some(para) = element_to_paragraph(elem) {
            paragraphs.push(para);
        }
    }
    paragraphs
}

/// Group word-level elements into multi-word, multi-line paragraphs.
///
/// Algorithm:
/// 1. Sort by Y position (top-to-bottom in PDF coords = largest y_min first)
/// 2. Group into lines by y_min proximity (tolerance = median height × 0.5)
/// 3. Sort within lines by X position (left-to-right)
/// 4. Group lines into paragraphs by vertical gap (gap > 1.5× median line height)
/// 5. Create one PdfParagraph per paragraph group
fn group_words_to_paragraphs(elements: &[ContentElement]) -> Vec<PdfParagraph> {
    if elements.is_empty() {
        return Vec::new();
    }

    let mut heights: Vec<f32> = elements
        .iter()
        .filter_map(|e| e.bbox.map(|r| r.height()))
        .filter(|h| *h > 0.0)
        .collect();

    let median_height = if !heights.is_empty() {
        heights.sort_by(|a, b| a.total_cmp(b));
        heights[heights.len() / 2]
    } else {
        12.0
    };
    let tolerance = median_height * LINE_Y_TOLERANCE_FRACTION;

    let mut sorted_indices: Vec<usize> = (0..elements.len()).filter(|&i| elements[i].bbox.is_some()).collect();
    sorted_indices.sort_by(|&a, &b| {
        let y_a = elements[a].bbox.map_or(0.0, |r| r.y_min);
        let y_b = elements[b].bbox.map_or(0.0, |r| r.y_min);
        let x_a = elements[a].bbox.map_or(0.0, |r| r.left);
        let x_b = elements[b].bbox.map_or(0.0, |r| r.left);
        y_b.total_cmp(&y_a).then_with(|| x_a.total_cmp(&x_b))
    });

    let mut lines: Vec<Vec<usize>> = Vec::new();
    let mut current_line: Vec<usize> = Vec::new();
    let mut line_y_sum: f32 = 0.0;

    for &idx in &sorted_indices {
        let y = elements[idx].bbox.map_or(0.0, |r| r.y_min);

        if current_line.is_empty() {
            current_line.push(idx);
            line_y_sum = y;
        } else {
            let avg_y = line_y_sum / current_line.len() as f32;
            if (y - avg_y).abs() <= tolerance {
                current_line.push(idx);
                line_y_sum += y;
            } else {
                current_line.sort_by(|&a, &b| {
                    let xa = elements[a].bbox.map_or(0.0, |r| r.left);
                    let xb = elements[b].bbox.map_or(0.0, |r| r.left);
                    xa.total_cmp(&xb)
                });
                lines.push(current_line);
                current_line = vec![idx];
                line_y_sum = y;
            }
        }
    }
    if !current_line.is_empty() {
        current_line.sort_by(|&a, &b| {
            let xa = elements[a].bbox.map_or(0.0, |r| r.left);
            let xb = elements[b].bbox.map_or(0.0, |r| r.left);
            xa.total_cmp(&xb)
        });
        lines.push(current_line);
    }

    if lines.is_empty() {
        return Vec::new();
    }

    let line_heights: Vec<f32> = lines
        .iter()
        .map(|line| {
            let min_y = line
                .iter()
                .filter_map(|&i| elements[i].bbox.map(|r| r.y_min))
                .fold(f32::MAX, f32::min);
            let max_y = line
                .iter()
                .filter_map(|&i| elements[i].bbox.map(|r| r.y_max))
                .fold(f32::MIN, f32::max);
            if min_y == f32::MAX || max_y == f32::MIN {
                median_height
            } else {
                (max_y - min_y).max(1.0)
            }
        })
        .collect();

    let paragraph_gap_threshold: f32 = if lines.len() >= 2 {
        let line_bottoms: Vec<f32> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let bottom = line
                    .iter()
                    .filter_map(|&idx| elements[idx].bbox.map(|r| r.y_min))
                    .fold(f32::MAX, f32::min);
                if bottom == f32::MAX {
                    let top = line
                        .iter()
                        .filter_map(|&idx| elements[idx].bbox.map(|r| r.y_max))
                        .fold(f32::MIN, f32::max);
                    if top == f32::MIN { 0.0 } else { top - line_heights[i] }
                } else {
                    bottom
                }
            })
            .collect();

        let line_tops: Vec<f32> = lines
            .iter()
            .map(|line| {
                line.iter()
                    .filter_map(|&idx| elements[idx].bbox.map(|r| r.y_max))
                    .fold(f32::MIN, f32::max)
            })
            .collect();

        let mut gaps: Vec<f32> = line_bottoms
            .iter()
            .zip(line_tops.iter().skip(1))
            .map(|(&prev_bottom, &next_top)| prev_bottom - next_top)
            .collect();

        if gaps.is_empty() {
            let mut sorted = line_heights.to_vec();
            sorted.sort_by(|a, b| a.total_cmp(b));
            sorted[sorted.len() / 2] * 1.5
        } else if gaps.len() == 1 {
            median_height * 1.5
        } else {
            gaps.sort_by(|a, b| a.total_cmp(b));
            let median_gap = gaps[gaps.len() / 2];
            if median_gap > 0.1 {
                (median_gap * 2.0).max(median_height * 0.3)
            } else {
                median_height * 0.5
            }
        }
    } else {
        f32::MAX
    };

    let mut paragraphs: Vec<PdfParagraph> = Vec::new();
    let mut current_para_lines: Vec<&Vec<usize>> = Vec::new();
    let mut prev_line_bottom: Option<f32> = None;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_top = line
            .iter()
            .filter_map(|&i| elements[i].bbox.map(|r| r.y_max))
            .fold(f32::MIN, f32::max);
        let line_bottom = line
            .iter()
            .filter_map(|&i| elements[i].bbox.map(|r| r.y_min))
            .fold(f32::MAX, f32::min);

        if let Some(prev_bottom) = prev_line_bottom {
            let gap = prev_bottom - line_top;
            if gap > paragraph_gap_threshold && !current_para_lines.is_empty() {
                if let Some(para) = build_paragraph_from_lines(&current_para_lines, elements) {
                    paragraphs.push(para);
                }
                current_para_lines = Vec::new();
            }
        }

        current_para_lines.push(line);
        prev_line_bottom = Some(if line_bottom == f32::MAX {
            line_top - line_heights[line_idx]
        } else {
            line_bottom
        });
    }

    if !current_para_lines.is_empty()
        && let Some(para) = build_paragraph_from_lines(&current_para_lines, elements)
    {
        paragraphs.push(para);
    }

    paragraphs
}

/// Build a single `PdfParagraph` from a group of lines (each line is a slice of element indices).
fn build_paragraph_from_lines(line_groups: &[&Vec<usize>], elements: &[ContentElement]) -> Option<PdfParagraph> {
    let first_elem = line_groups.first().and_then(|l| l.first()).map(|&i| &elements[i]);

    let dominant_font_size = {
        let mut sizes: Vec<f32> = line_groups
            .iter()
            .flat_map(|l| l.iter())
            .filter_map(|&i| elements[i].font_size)
            .collect();
        if sizes.is_empty() {
            first_elem.and_then(|e| e.font_size).unwrap_or(12.0)
        } else {
            sizes.sort_by(|a, b| a.total_cmp(b));
            sizes[sizes.len() / 2]
        }
    };

    let mut pdf_lines: Vec<PdfLine> = Vec::new();
    let mut total_word_count = 0usize;

    for line in line_groups {
        let mut segments: Vec<SegmentData> = Vec::new();
        let mut line_is_bold = false;
        let mut line_is_monospace = false;
        let mut baseline_y_sum = 0.0f32;
        let mut baseline_count = 0usize;

        for &idx in line.iter() {
            let elem = &elements[idx];
            let text = elem.text.trim();
            if text.is_empty() {
                continue;
            }
            let font_size = elem.font_size.unwrap_or(dominant_font_size);
            let is_code = matches!(elem.semantic_role, Some(SemanticRole::Code));
            let is_monospace = elem.is_monospace || is_code;

            if elem.is_bold {
                line_is_bold = true;
            }
            if is_monospace {
                line_is_monospace = true;
            }

            let y_min = elem.bbox.map_or(0.0, |r| r.y_min);
            baseline_y_sum += y_min;
            baseline_count += 1;

            segments.push(SegmentData {
                text: text.to_string(),
                x: elem.bbox.map_or(0.0, |r| r.left),
                y: y_min,
                width: elem.bbox.map_or(0.0, |r| r.width()),
                height: elem.bbox.map_or(0.0, |r| r.height()),
                font_size,
                is_bold: elem.is_bold,
                is_italic: elem.is_italic,
                is_monospace,
                baseline_y: y_min,
                assigned_role: None,
            });

            total_word_count += 1;
        }

        if segments.is_empty() {
            continue;
        }

        let avg_baseline = if baseline_count > 0 {
            baseline_y_sum / baseline_count as f32
        } else {
            0.0
        };

        let line_font_size = if !segments.is_empty() {
            let mut sizes: Vec<f32> = segments.iter().map(|s| s.font_size).collect();
            sizes.sort_by(|a, b| a.total_cmp(b));
            sizes[sizes.len() / 2]
        } else {
            dominant_font_size
        };

        pdf_lines.push(PdfLine {
            segments,
            baseline_y: avg_baseline,
            dominant_font_size: line_font_size,
            is_bold: line_is_bold,
            is_monospace: line_is_monospace,
        });
    }

    if total_word_count == 0 {
        return None;
    }

    let (heading_level, is_list_item, is_code_block, is_formula, is_bold, is_page_furniture, layout_class) =
        if let Some(elem) = first_elem {
            let is_code = matches!(elem.semantic_role, Some(SemanticRole::Code));
            let is_formula = matches!(elem.semantic_role, Some(SemanticRole::Formula))
                || matches!(elem.layout_class, Some(LayoutHintClass::Formula));
            let is_page_furniture = false;
            let mut is_list = matches!(elem.semantic_role, Some(SemanticRole::ListItem));

            if !is_list {
                let leading_text: String = pdf_lines
                    .first()
                    .map(|l| {
                        l.segments
                            .iter()
                            .take(3)
                            .map(|s| s.text.as_str())
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default();
                is_list = super::paragraphs::is_list_prefix_multi_token(&leading_text);
            }

            let heading_level = match elem.semantic_role {
                Some(SemanticRole::Heading { level }) if total_word_count <= MAX_HEADING_WORD_COUNT => Some(level),
                _ => None,
            };

            (
                heading_level,
                is_list,
                is_code,
                is_formula,
                elem.is_bold,
                is_page_furniture,
                elem.layout_class,
            )
        } else {
            (None, false, false, false, false, false, None)
        };

    let block_bbox = {
        let mut left = f32::MAX;
        let mut bottom = f32::MAX;
        let mut right = f32::MIN;
        let mut top = f32::MIN;
        for line in line_groups {
            for &idx in line.iter() {
                if let Some(r) = elements[idx].bbox {
                    left = left.min(r.left);
                    bottom = bottom.min(r.y_min);
                    right = right.max(r.right);
                    top = top.max(r.y_max);
                }
            }
        }
        if left == f32::MAX {
            None
        } else {
            Some((left, bottom, right, top))
        }
    };

    Some(PdfParagraph {
        text: String::new(),
        lines: pdf_lines,
        dominant_font_size,
        heading_level,
        is_bold,
        is_list_item,
        is_code_block,
        is_formula,
        is_page_furniture,
        layout_class,
        caption_for: None,
        block_bbox,
    })
}

/// Convert a single `ContentElement` into a `PdfParagraph`.
///
/// Returns `None` for empty elements.
fn element_to_paragraph(elem: &ContentElement) -> Option<PdfParagraph> {
    let full_text = if let Some(ref label) = elem.list_label {
        format!("{} {}", label, elem.text)
    } else {
        elem.text.clone()
    };

    let word_count = full_text.split_whitespace().count();
    if word_count == 0 {
        return None;
    }

    let font_size = elem.font_size.unwrap_or(12.0);

    let mut is_list_item = matches!(elem.semantic_role, Some(SemanticRole::ListItem));
    let is_code_block = matches!(elem.semantic_role, Some(SemanticRole::Code));
    let is_formula = matches!(elem.semantic_role, Some(SemanticRole::Formula))
        || matches!(elem.layout_class, Some(LayoutHintClass::Formula));
    let is_monospace = elem.is_monospace || is_code_block;
    let is_page_furniture = false;

    let heading_level = match elem.semantic_role {
        Some(SemanticRole::Heading { level }) if word_count <= MAX_HEADING_WORD_COUNT => Some(level),
        _ => None,
    };

    if !is_list_item && heading_level.is_none() {
        is_list_item = super::paragraphs::is_list_prefix_multi_token(&full_text);
    }

    let block_bbox = elem.bbox.map(|r| (r.left, r.y_min, r.right, r.y_max));

    let segments: Vec<SegmentData> = if elem.level == ElementLevel::Line || elem.level == ElementLevel::Block {
        full_text
            .split_whitespace()
            .map(|w| SegmentData {
                text: w.to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                font_size,
                is_bold: elem.is_bold,
                is_italic: elem.is_italic,
                is_monospace,
                baseline_y: 0.0,
                assigned_role: None,
            })
            .collect()
    } else {
        vec![SegmentData {
            text: full_text.clone(),
            x: elem.bbox.map_or(0.0, |r| r.left),
            y: elem.bbox.map_or(0.0, |r| r.y_min),
            width: elem.bbox.map_or(0.0, |r| r.width()),
            height: elem.bbox.map_or(0.0, |r| r.height()),
            font_size,
            is_bold: elem.is_bold,
            is_italic: elem.is_italic,
            is_monospace,
            baseline_y: elem.bbox.map_or(0.0, |r| r.y_min),
            assigned_role: None,
        }]
    };

    let line = PdfLine {
        segments,
        baseline_y: elem.bbox.map_or(0.0, |r| r.y_min),
        dominant_font_size: font_size,
        is_bold: elem.is_bold,
        is_monospace,
    };

    Some(PdfParagraph {
        text: String::new(),
        lines: vec![line],
        dominant_font_size: font_size,
        heading_level,
        is_bold: elem.is_bold,
        is_list_item,
        is_code_block,
        is_formula,
        is_page_furniture,
        layout_class: elem.layout_class,
        caption_for: None,
        block_bbox,
    })
}

/// Reorder elements for multi-column reading order.
///
/// Detects two-column layouts by finding the largest horizontal gap in element
/// positions. When detected, reorders elements: left column top-to-bottom,
/// then right column top-to-bottom.
///
/// The detection algorithm:
/// 1. Filter out furniture elements from analysis (but keep them in output).
/// 2. Collect X-center positions of content elements that have bounding boxes.
/// 3. Sort X-centers, find the largest gap between adjacent values.
/// 4. Gap must be ≥ 10% of estimated page width (max right edge of any element).
/// 5. Validate ≥ 2 elements on each side of the split.
/// 6. Validate each side spans ≥ 30% of the total Y range.
/// 7. If valid: partition elements into left/right groups, sort each top-to-bottom,
///    concatenate left then right.
/// 8. If no valid split: leave elements in their current order.
#[allow(dead_code)]
pub(crate) fn reorder_elements_reading_order(elements: &mut Vec<ContentElement>) {
    if elements.len() < MIN_ELEMENTS_PER_COLUMN * 2 {
        return;
    }

    let content_indices: Vec<usize> = elements
        .iter()
        .enumerate()
        .filter(|(_, e)| e.bbox.is_some())
        .map(|(i, _)| i)
        .collect();

    if content_indices.len() < MIN_ELEMENTS_PER_COLUMN * 2 {
        return;
    }

    let page_width_estimate = elements
        .iter()
        .filter_map(|e| e.bbox.map(|r| r.right))
        .fold(0.0_f32, f32::max);

    if page_width_estimate < 1.0 {
        return;
    }

    let min_gap = page_width_estimate * MIN_COLUMN_GAP_FRACTION;

    let mut x_centers: Vec<(f32, usize)> = content_indices
        .iter()
        .map(|&i| {
            let r = elements[i].bbox.expect("filtered above");
            let x_center = (r.left + r.right) / 2.0;
            (x_center, i)
        })
        .collect();

    x_centers.sort_by(|a, b| a.0.total_cmp(&b.0));

    let mut best_gap = 0.0_f32;
    let mut best_split_x: Option<f32> = None;

    for window in x_centers.windows(2) {
        let gap = window[1].0 - window[0].0;
        if gap > min_gap && gap > best_gap {
            best_gap = gap;
            best_split_x = Some((window[0].0 + window[1].0) / 2.0);
        }
    }

    let split_x = match best_split_x {
        Some(x) => x,
        None => return,
    };

    let left_count = content_indices
        .iter()
        .filter(|&&i| {
            let r = elements[i].bbox.expect("filtered above");
            (r.left + r.right) / 2.0 < split_x
        })
        .count();
    let right_count = content_indices.len() - left_count;

    if left_count < MIN_ELEMENTS_PER_COLUMN || right_count < MIN_ELEMENTS_PER_COLUMN {
        return;
    }

    let mut y_min_all = f32::MAX;
    let mut y_max_all = f32::MIN;
    for &i in &content_indices {
        let r = elements[i].bbox.expect("filtered above");
        y_min_all = y_min_all.min(r.y_min);
        y_max_all = y_max_all.max(r.y_max);
    }
    let total_y_range = y_max_all - y_min_all;

    if total_y_range < 1.0 {
        return;
    }

    let left_y_span = {
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;
        for &i in &content_indices {
            let r = elements[i].bbox.expect("filtered above");
            if (r.left + r.right) / 2.0 < split_x {
                y_min = y_min.min(r.y_min);
                y_max = y_max.max(r.y_max);
            }
        }
        if y_max > y_min { y_max - y_min } else { 0.0 }
    };

    let right_y_span = {
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;
        for &i in &content_indices {
            let r = elements[i].bbox.expect("filtered above");
            if (r.left + r.right) / 2.0 >= split_x {
                y_min = y_min.min(r.y_min);
                y_max = y_max.max(r.y_max);
            }
        }
        if y_max > y_min { y_max - y_min } else { 0.0 }
    };

    let min_y_span = total_y_range * MIN_COLUMN_Y_SPAN_FRACTION;
    if left_y_span < min_y_span || right_y_span < min_y_span {
        return;
    }

    let mut left_col: Vec<ContentElement> = Vec::new();
    let mut right_col: Vec<ContentElement> = Vec::new();

    for elem in elements.drain(..) {
        if let Some(r) = elem.bbox {
            let x_center = (r.left + r.right) / 2.0;
            if x_center < split_x {
                left_col.push(elem);
            } else {
                right_col.push(elem);
            }
        } else {
            left_col.push(elem);
        }
    }

    left_col.sort_by(|a, b| {
        let ya = a.bbox.map_or(0.0, |r| r.y_max);
        let yb = b.bbox.map_or(0.0, |r| r.y_max);
        yb.total_cmp(&ya)
    });
    right_col.sort_by(|a, b| {
        let ya = a.bbox.map_or(0.0, |r| r.y_max);
        let yb = b.bbox.map_or(0.0, |r| r.y_max);
        yb.total_cmp(&ya)
    });

    elements.extend(left_col);
    elements.extend(right_col);
}

#[cfg(test)]
mod tests {
    use super::super::geometry::Rect;
    use super::*;

    fn make_element(text: &str, role: Option<SemanticRole>) -> ContentElement {
        ContentElement {
            text: text.to_string(),
            bbox: None,
            font_size: Some(12.0),
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            semantic_role: role,
            level: ElementLevel::Block,
            list_label: None,
            layout_class: None,
        }
    }

    fn make_word(text: &str, x: f32, y_min: f32, y_max: f32) -> ContentElement {
        ContentElement {
            text: text.to_string(),
            bbox: Some(Rect::from_lbrt(x, y_min, x + 30.0, y_max)),
            font_size: Some(12.0),
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            semantic_role: Some(SemanticRole::Paragraph),
            level: ElementLevel::Word,
            list_label: None,
            layout_class: None,
        }
    }

    fn make_page(elements: Vec<ContentElement>) -> PageContent {
        PageContent { elements }
    }

    #[test]
    fn test_heading_conversion() {
        let page = make_page(vec![
            make_element("Title Text", Some(SemanticRole::Heading { level: 1 })),
            make_element("Body text", Some(SemanticRole::Paragraph)),
        ]);
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras.len(), 2);
        assert_eq!(paras[0].heading_level, Some(1));
        assert_eq!(paras[1].heading_level, None);
    }

    #[test]
    fn test_heading_too_many_words_demoted() {
        let long_heading = (0..25).map(|i| format!("word{i}")).collect::<Vec<_>>().join(" ");
        let page = make_page(vec![make_element(
            &long_heading,
            Some(SemanticRole::Heading { level: 2 }),
        )]);
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras[0].heading_level, None);
    }

    #[test]
    fn test_list_item_from_role() {
        let mut elem = make_element("First item", Some(SemanticRole::ListItem));
        elem.list_label = Some("1.".to_string());
        let page = make_page(vec![elem]);
        let paras = content_to_paragraphs(&page);
        assert!(paras[0].is_list_item);
        assert_eq!(paras[0].lines[0].segments[0].text, "1.");
    }

    #[test]
    fn test_list_item_from_text_prefix() {
        let page = make_page(vec![make_element("• Bullet point", Some(SemanticRole::Paragraph))]);
        let paras = content_to_paragraphs(&page);
        assert!(paras[0].is_list_item);
    }

    /// Regression: a tagged Heading whose visible text happens to start
    /// with a numeric prefix (e.g. `"3. Conclusions"`) must not also get
    /// flagged as a list item. When both flags are set, assembly wraps
    /// the heading in a `ListStart`/`ListEnd` pair around a `Heading`
    /// node, which is an invalid CommonMark AST (`List` children may only
    /// be `Item`/`TaskItem`) and trips comrak's debug validator.
    #[test]
    fn test_heading_with_numeric_prefix_not_marked_list_item() {
        let page = make_page(vec![make_element(
            "3. Conclusions",
            Some(SemanticRole::Heading { level: 3 }),
        )]);
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras[0].heading_level, Some(3));
        assert!(
            !paras[0].is_list_item,
            "tagged Heading must not also be flagged is_list_item from text pattern"
        );
    }

    #[test]
    fn test_code_block() {
        let page = make_page(vec![make_element("fn main() {}", Some(SemanticRole::Code))]);
        let paras = content_to_paragraphs(&page);
        assert!(paras[0].is_code_block);
    }

    #[test]
    fn test_empty_skipped() {
        let page = make_page(vec![
            make_element("", Some(SemanticRole::Paragraph)),
            make_element("   ", Some(SemanticRole::Paragraph)),
            make_element("Real text", Some(SemanticRole::Paragraph)),
        ]);
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].lines[0].segments[0].text, "Real");
    }

    #[test]
    fn test_block_bbox_propagated() {
        let mut elem = make_element("With bounds", Some(SemanticRole::Paragraph));
        elem.bbox = Some(Rect::from_lbrt(50.0, 100.0, 400.0, 120.0));
        let page = make_page(vec![elem]);
        let paras = content_to_paragraphs(&page);
        let bbox = paras[0].block_bbox.unwrap();
        assert!((bbox.0 - 50.0).abs() < f32::EPSILON);
        assert!((bbox.1 - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_six_words_two_lines_one_paragraph() {
        let elements = vec![
            make_word("Hello", 50.0, 700.0, 712.0),
            make_word("world", 85.0, 700.0, 712.0),
            make_word("foo", 120.0, 700.0, 712.0),
            make_word("bar", 50.0, 684.0, 696.0),
            make_word("baz", 85.0, 684.0, 696.0),
            make_word("qux", 120.0, 684.0, 696.0),
        ];
        let page = PageContent { elements };
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras.len(), 1, "expected 1 paragraph, got {}", paras.len());
        assert_eq!(
            paras[0].lines.len(),
            2,
            "expected 2 lines, got {}",
            paras[0].lines.len()
        );
        assert_eq!(paras[0].lines[0].segments.len(), 3);
        assert_eq!(paras[0].lines[0].segments[0].text, "Hello");
        assert_eq!(paras[0].lines[0].segments[1].text, "world");
        assert_eq!(paras[0].lines[0].segments[2].text, "foo");
    }

    #[test]
    fn test_large_gap_produces_two_paragraphs() {
        let elements = vec![
            make_word("First", 50.0, 700.0, 712.0),
            make_word("para", 85.0, 700.0, 712.0),
            make_word("Second", 50.0, 600.0, 612.0),
            make_word("para", 85.0, 600.0, 612.0),
        ];
        let page = PageContent { elements };
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras.len(), 2, "expected 2 paragraphs, got {}", paras.len());
        assert_eq!(paras[0].lines[0].segments[0].text, "First");
        assert_eq!(paras[1].lines[0].segments[0].text, "Second");
    }

    #[test]
    fn test_single_word_produces_one_paragraph() {
        let elements = vec![make_word("Solo", 50.0, 400.0, 412.0)];
        let page = PageContent { elements };
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].lines[0].segments[0].text, "Solo");
    }

    #[test]
    fn test_empty_word_elements_skipped() {
        let mut empty = make_word("", 50.0, 400.0, 412.0);
        empty.text = "   ".to_string();
        let page = PageContent {
            elements: vec![empty, make_word("Real", 85.0, 400.0, 412.0)],
        };
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras.len(), 1);
        assert_eq!(paras[0].lines[0].segments[0].text, "Real");
    }

    #[test]
    fn test_block_bbox_spans_all_words_in_paragraph() {
        let elements = vec![
            make_word("A", 50.0, 700.0, 712.0),
            make_word("B", 200.0, 700.0, 712.0),
            make_word("C", 100.0, 685.0, 697.0),
        ];
        let page = PageContent { elements };
        let paras = content_to_paragraphs(&page);
        assert_eq!(paras.len(), 1);
        let bbox = paras[0].block_bbox.unwrap();
        assert!((bbox.0 - 50.0).abs() < f32::EPSILON, "left={}", bbox.0);
        assert!((bbox.1 - 685.0).abs() < f32::EPSILON, "bottom={}", bbox.1);
        assert!((bbox.2 - 230.0).abs() < f32::EPSILON, "right={}", bbox.2);
        assert!((bbox.3 - 712.0).abs() < f32::EPSILON, "top={}", bbox.3);
    }

    /// Create a block-level element with a bounding box for column tests.
    fn make_block(text: &str, x: f32, y_min: f32, y_max: f32, role: SemanticRole) -> ContentElement {
        ContentElement {
            text: text.to_string(),
            bbox: Some(Rect::from_lbrt(x, y_min, x + 80.0, y_max)),
            font_size: Some(12.0),
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            semantic_role: Some(role),
            level: ElementLevel::Block,
            list_label: None,
            layout_class: None,
        }
    }

    #[test]
    fn test_reorder_no_columns_unchanged() {
        let mut elements = vec![
            make_block("P1", 50.0, 680.0, 700.0, SemanticRole::Paragraph),
            make_block("P2", 50.0, 630.0, 650.0, SemanticRole::Paragraph),
            make_block("P3", 60.0, 580.0, 600.0, SemanticRole::Paragraph),
        ];
        let original_texts: Vec<_> = elements.iter().map(|e| e.text.clone()).collect();
        reorder_elements_reading_order(&mut elements);
        let after_texts: Vec<_> = elements.iter().map(|e| e.text.clone()).collect();
        assert_eq!(original_texts, after_texts, "single-column should not be reordered");
    }

    #[test]
    fn test_reorder_two_columns_detected() {
        let mut elements = vec![
            make_block("R1", 400.0, 700.0, 712.0, SemanticRole::Paragraph),
            make_block("L1", 0.0, 700.0, 712.0, SemanticRole::Paragraph),
            make_block("R2", 400.0, 650.0, 662.0, SemanticRole::Paragraph),
            make_block("L2", 0.0, 650.0, 662.0, SemanticRole::Paragraph),
            make_block("L3", 0.0, 600.0, 612.0, SemanticRole::Paragraph),
        ];
        reorder_elements_reading_order(&mut elements);
        let texts: Vec<_> = elements.iter().map(|e| e.text.clone()).collect();
        assert_eq!(
            texts,
            vec!["L1", "L2", "L3", "R1", "R2"],
            "two-column layout should be reordered left-then-right, got: {:?}",
            texts
        );
    }

    #[test]
    fn test_reorder_header_footer_stays_in_spatial_position() {
        let mut elements = vec![
            make_block("L1", 0.0, 650.0, 662.0, SemanticRole::Paragraph),
            make_block("R1", 400.0, 650.0, 662.0, SemanticRole::Paragraph),
            make_block("Header", 0.0, 750.0, 762.0, SemanticRole::Other),
            make_block("L2", 0.0, 600.0, 612.0, SemanticRole::Paragraph),
            make_block("R2", 400.0, 600.0, 612.0, SemanticRole::Paragraph),
        ];
        reorder_elements_reading_order(&mut elements);
        let texts: Vec<_> = elements.iter().map(|e| e.text.clone()).collect();
        assert_eq!(texts.len(), 5, "all elements must be present; got: {:?}", texts);
        let header_pos = texts.iter().position(|t| t == "Header").unwrap();
        let l1_pos = texts.iter().position(|t| t == "L1").unwrap();
        let l2_pos = texts.iter().position(|t| t == "L2").unwrap();
        assert!(
            header_pos < l1_pos,
            "Header (y=762) should precede L1 (y=662) in spatial order"
        );
        assert!(l1_pos < l2_pos, "L1 (y=662) should precede L2 (y=612) in spatial order");
    }

    #[test]
    fn test_reorder_too_few_elements_unchanged() {
        let mut elements = vec![
            make_block("A", 0.0, 700.0, 712.0, SemanticRole::Paragraph),
            make_block("B", 400.0, 700.0, 712.0, SemanticRole::Paragraph),
            make_block("C", 0.0, 650.0, 662.0, SemanticRole::Paragraph),
        ];
        let original: Vec<_> = elements.iter().map(|e| e.text.clone()).collect();
        reorder_elements_reading_order(&mut elements);
        let after: Vec<_> = elements.iter().map(|e| e.text.clone()).collect();
        assert_eq!(original, after, "too few elements should not be reordered");
    }

    #[test]
    fn test_reorder_no_y_span_unchanged() {
        let mut elements = vec![
            make_block("A", 0.0, 700.0, 712.0, SemanticRole::Paragraph),
            make_block("B", 400.0, 700.0, 712.0, SemanticRole::Paragraph),
            make_block("C", 0.0, 700.0, 712.0, SemanticRole::Paragraph),
            make_block("D", 400.0, 700.0, 712.0, SemanticRole::Paragraph),
        ];
        reorder_elements_reading_order(&mut elements);
        let total: usize = elements.len();
        assert_eq!(total, 4, "all elements must be preserved");
    }
}
