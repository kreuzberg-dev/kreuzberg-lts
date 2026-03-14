//! Adapters that convert extraction-source-specific types into the unified
//! [`PageContent`] DTO for the shared markdown pipeline.

use pdfium_render::prelude::{ContentRole, ExtractedBlock};

use super::content::{ContentElement, ElementLevel, ExtractionSource, PageContent, SemanticRole};
use super::geometry::Rect;
use crate::pdf::hierarchy::SegmentData;

// ── Structure tree adapter ──────────────────────────────────────────────

/// Convert structure-tree `ExtractedBlock`s into a [`PageContent`].
///
/// Flattens the block hierarchy into a flat list of `ContentElement`s,
/// mapping `ContentRole` to `SemanticRole` and extracting bounding boxes.
pub(super) fn from_structure_tree(
    blocks: &[ExtractedBlock],
    page_width: f32,
    page_height: f32,
    page_number: usize,
) -> PageContent {
    let mut elements = Vec::new();
    flatten_blocks(blocks, &mut elements);

    PageContent {
        page_number,
        page_width,
        page_height,
        elements,
        source: ExtractionSource::StructureTree,
    }
}

/// Recursively flatten `ExtractedBlock` hierarchy into `ContentElement`s.
fn flatten_blocks(blocks: &[ExtractedBlock], elements: &mut Vec<ContentElement>) {
    for block in blocks {
        if !block.children.is_empty() {
            flatten_blocks(&block.children, elements);
            continue;
        }

        if block.text.trim().is_empty() {
            continue;
        }

        let bbox = block
            .bounds
            .as_ref()
            .map(|b| Rect::from_lbrt(b.left().value, b.bottom().value, b.right().value, b.top().value));

        let (semantic_role, list_label) = map_content_role(&block.role);

        elements.push(ContentElement {
            text: block.text.clone(),
            bbox,
            font_size: block.font_size,
            is_bold: block.is_bold,
            is_italic: block.is_italic,
            is_monospace: block.is_monospace,
            confidence: None,
            semantic_role: Some(semantic_role),
            level: ElementLevel::Block,
            list_label,
        });
    }
}

/// Map `ContentRole` from pdfium to our `SemanticRole`.
fn map_content_role(role: &ContentRole) -> (SemanticRole, Option<String>) {
    match role {
        ContentRole::Heading { level } => (SemanticRole::Heading { level: *level }, None),
        ContentRole::Paragraph => (SemanticRole::Paragraph, None),
        ContentRole::ListItem { label } => (SemanticRole::ListItem, label.clone()),
        ContentRole::TableCell { .. } => (SemanticRole::TableCell, None),
        ContentRole::Figure { .. } => (SemanticRole::Figure, None),
        ContentRole::Caption => (SemanticRole::Caption, None),
        ContentRole::Code => (SemanticRole::Code, None),
        ContentRole::BlockQuote => (SemanticRole::BlockQuote, None),
        ContentRole::Link { .. } => (SemanticRole::Paragraph, None),
        ContentRole::Other(s) if s == "Formula" => (SemanticRole::Formula, None),
        ContentRole::Other(_) => (SemanticRole::Other, None),
    }
}

// ── Heuristic segments adapter ──────────────────────────────────────────

/// Convert pdfium heuristic `SegmentData`s into a [`PageContent`].
///
/// Each segment becomes a line-level `ContentElement` with its spatial data
/// mapped directly (already in PDF coordinate space).
pub(super) fn from_segments(
    segments: Vec<SegmentData>,
    page_width: f32,
    page_height: f32,
    page_number: usize,
) -> PageContent {
    let elements = segments
        .into_iter()
        .filter(|s| !s.text.trim().is_empty())
        .map(|s| {
            let bbox = if s.width > 0.0 || s.height > 0.0 || s.x != 0.0 || s.y != 0.0 {
                Some(Rect::from_lbrt(s.x, s.y, s.x + s.width, s.y + s.height))
            } else {
                None
            };

            ContentElement {
                text: s.text,
                bbox,
                font_size: Some(s.font_size),
                is_bold: s.is_bold,
                is_italic: s.is_italic,
                is_monospace: s.is_monospace,
                confidence: None,
                semantic_role: None,
                level: ElementLevel::Line,
                list_label: None,
            }
        })
        .collect();

    PageContent {
        page_number,
        page_width,
        page_height,
        elements,
        source: ExtractionSource::PdfiumHeuristic,
    }
}

// ── OCR adapter ─────────────────────────────────────────────────────────

/// Convert `OcrElement`s into a [`PageContent`].
///
/// Coordinates are flipped from image space (y=0 at top) to PDF space
/// (y=0 at bottom) using `page_height`.
#[cfg(feature = "layout-detection")]
pub(crate) fn from_ocr_elements(
    elements: &[crate::types::OcrElement],
    page_width: f32,
    page_height: f32,
    page_number: usize,
) -> PageContent {
    let content_elements = elements
        .iter()
        .filter(|e| !e.text.trim().is_empty())
        .map(|e| {
            let (left, top, width, height) = e.geometry.to_aabb();
            let left_f = left as f32;
            let top_f = top as f32;
            let w_f = width as f32;
            let h_f = height as f32;

            // Flip from image coords (y=0 top) to PDF coords (y=0 bottom).
            let pdf_bottom = page_height - (top_f + h_f);
            let pdf_top = page_height - top_f;

            let bbox = if w_f > 0.0 || h_f > 0.0 {
                Some(Rect::from_lbrt(left_f, pdf_bottom, left_f + w_f, pdf_top))
            } else {
                None
            };

            let level = match e.level {
                crate::types::OcrElementLevel::Word => ElementLevel::Word,
                crate::types::OcrElementLevel::Line => ElementLevel::Line,
                crate::types::OcrElementLevel::Block | crate::types::OcrElementLevel::Page => ElementLevel::Block,
            };

            ContentElement {
                text: e.text.clone(),
                bbox,
                font_size: None,
                is_bold: false,
                is_italic: false,
                is_monospace: false,
                confidence: Some(e.confidence.recognition as f32),
                semantic_role: None,
                level,
                list_label: None,
            }
        })
        .collect();

    PageContent {
        page_number,
        page_width,
        page_height,
        elements: content_elements,
        source: ExtractionSource::Ocr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdfium_render::prelude::PdfPoints;
    use pdfium_render::prelude::PdfRect;

    fn make_block(role: ContentRole, text: &str) -> ExtractedBlock {
        ExtractedBlock {
            role,
            text: text.to_string(),
            bounds: None,
            font_size: Some(12.0),
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            children: Vec::new(),
        }
    }

    fn make_block_with_bounds(role: ContentRole, text: &str) -> ExtractedBlock {
        ExtractedBlock {
            role,
            text: text.to_string(),
            bounds: Some(PdfRect::new(
                PdfPoints::new(100.0),
                PdfPoints::new(50.0),
                PdfPoints::new(200.0),
                PdfPoints::new(400.0),
            )),
            font_size: Some(12.0),
            is_bold: true,
            is_italic: false,
            is_monospace: false,
            children: Vec::new(),
        }
    }

    #[test]
    fn test_from_structure_tree_basic() {
        let blocks = vec![
            make_block(ContentRole::Heading { level: 1 }, "Title"),
            make_block(ContentRole::Paragraph, "Body text"),
        ];
        let page = from_structure_tree(&blocks, 612.0, 792.0, 1);
        assert_eq!(page.elements.len(), 2);
        assert_eq!(page.source, ExtractionSource::StructureTree);
        assert_eq!(page.elements[0].semantic_role, Some(SemanticRole::Heading { level: 1 }));
        assert_eq!(page.elements[1].semantic_role, Some(SemanticRole::Paragraph));
    }

    #[test]
    fn test_from_structure_tree_skips_empty() {
        let blocks = vec![
            make_block(ContentRole::Paragraph, ""),
            make_block(ContentRole::Paragraph, "   "),
            make_block(ContentRole::Paragraph, "Real text"),
        ];
        let page = from_structure_tree(&blocks, 612.0, 792.0, 1);
        assert_eq!(page.elements.len(), 1);
        assert_eq!(page.elements[0].text, "Real text");
    }

    #[test]
    fn test_from_structure_tree_flattens_children() {
        let blocks = vec![ExtractedBlock {
            role: ContentRole::Other("Table".to_string()),
            text: String::new(),
            bounds: None,
            font_size: None,
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            children: vec![
                make_block(ContentRole::Paragraph, "Cell 1"),
                make_block(ContentRole::Paragraph, "Cell 2"),
            ],
        }];
        let page = from_structure_tree(&blocks, 612.0, 792.0, 1);
        assert_eq!(page.elements.len(), 2);
    }

    #[test]
    fn test_from_structure_tree_maps_bounds() {
        let blocks = vec![make_block_with_bounds(ContentRole::Paragraph, "With bounds")];
        let page = from_structure_tree(&blocks, 612.0, 792.0, 1);
        let elem = &page.elements[0];
        assert!(elem.bbox.is_some());
        assert!(elem.is_bold);
    }

    #[test]
    fn test_from_structure_tree_list_item_label() {
        let blocks = vec![ExtractedBlock {
            role: ContentRole::ListItem {
                label: Some("1.".to_string()),
            },
            text: "First item".to_string(),
            bounds: None,
            font_size: Some(12.0),
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            children: Vec::new(),
        }];
        let page = from_structure_tree(&blocks, 612.0, 792.0, 1);
        assert_eq!(page.elements[0].semantic_role, Some(SemanticRole::ListItem));
        assert_eq!(page.elements[0].list_label, Some("1.".to_string()));
    }

    #[test]
    fn test_from_segments_basic() {
        let segments = vec![
            SegmentData {
                text: "Hello world".to_string(),
                x: 50.0,
                y: 700.0,
                width: 200.0,
                height: 12.0,
                font_size: 12.0,
                is_bold: false,
                is_italic: false,
                is_monospace: false,
                baseline_y: 700.0,
            },
            SegmentData {
                text: "Second line".to_string(),
                x: 50.0,
                y: 680.0,
                width: 180.0,
                height: 12.0,
                font_size: 12.0,
                is_bold: true,
                is_italic: false,
                is_monospace: false,
                baseline_y: 680.0,
            },
        ];
        let page = from_segments(segments, 612.0, 792.0, 1);
        assert_eq!(page.elements.len(), 2);
        assert_eq!(page.source, ExtractionSource::PdfiumHeuristic);
        assert!(page.elements[0].bbox.is_some());
        assert_eq!(page.elements[0].font_size, Some(12.0));
        assert!(page.elements[1].is_bold);
    }

    #[test]
    fn test_from_segments_skips_empty() {
        let segments = vec![SegmentData {
            text: "   ".to_string(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            font_size: 12.0,
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            baseline_y: 0.0,
        }];
        let page = from_segments(segments, 612.0, 792.0, 1);
        assert_eq!(page.elements.len(), 0);
    }
}
