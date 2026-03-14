//! Final markdown assembly from classified paragraphs, with optional table interleaving.

use super::render::{escape_html_entities, render_paragraph_to_output};
use super::types::PdfParagraph;

/// Assemble markdown with tables interleaved at their correct reading-order positions.
///
/// Tables are matched to pages by their `page_number` (1-indexed). Within a page,
/// tables with bounding boxes are placed at the correct vertical position relative to
/// paragraphs. Tables without bounding boxes are appended at the end of their page.
pub(super) fn assemble_markdown_with_tables(
    pages: Vec<Vec<PdfParagraph>>,
    tables: &[crate::types::Table],
    page_marker_format: Option<&str>,
) -> String {
    // Group tables by page number (1-indexed → 0-indexed)
    let mut tables_by_page: std::collections::BTreeMap<usize, Vec<&crate::types::Table>> =
        std::collections::BTreeMap::new();
    for table in tables {
        let page_idx = if table.page_number > 0 {
            table.page_number - 1
        } else {
            0
        };
        tables_by_page.entry(page_idx).or_default().push(table);
    }

    let mut output = String::new();

    for (page_idx, paragraphs) in pages.iter().enumerate() {
        if let Some(fmt) = page_marker_format {
            let marker = fmt.replace("{page_num}", &(page_idx + 1).to_string());
            output.push_str(&marker);
        } else if page_idx > 0 && !output.is_empty() {
            output.push_str("\n\n");
        }

        let page_tables = tables_by_page.remove(&page_idx);

        if let Some(tables) = page_tables {
            assemble_page_with_tables(&mut output, paragraphs, &tables);
        } else {
            for (para_idx, para) in paragraphs.iter().enumerate() {
                // Skip captions — they are rendered after their parent element
                if para.caption_for.is_some() {
                    continue;
                }
                if para_idx > 0 {
                    // Use single newline between consecutive list items
                    let prev_is_list = paragraphs[para_idx - 1].is_list_item;
                    if prev_is_list && para.is_list_item {
                        output.push('\n');
                    } else {
                        output.push_str("\n\n");
                    }
                }
                render_paragraph_to_output(para, &mut output);
                // Emit any captions associated with this paragraph
                emit_captions_for(&mut output, paragraphs, para_idx);
            }
        }
    }

    // Append tables for pages beyond what we have paragraphs for
    for tables in tables_by_page.values() {
        for table in tables {
            if !table.markdown.trim().is_empty() {
                if !output.is_empty() {
                    output.push_str("\n\n");
                }
                output.push_str(table.markdown.trim());
            }
        }
    }

    output
}

/// Assemble a single page's paragraphs with tables interleaved by vertical position.
fn assemble_page_with_tables(output: &mut String, paragraphs: &[PdfParagraph], tables: &[&crate::types::Table]) {
    // Split tables into positioned (have bounding box) and unpositioned
    let mut positioned: Vec<(f32, &str)> = Vec::new();
    let mut unpositioned: Vec<&str> = Vec::new();

    for table in tables {
        let md = table.markdown.trim();
        if md.is_empty() {
            continue;
        }
        if let Some(ref bbox) = table.bounding_box {
            // In PDF coordinates, y1 is the top of the table (higher = earlier in reading order)
            // Use y1 as the position reference
            positioned.push((bbox.y1 as f32, md));
        } else {
            unpositioned.push(md);
        }
    }

    // Sort positioned tables by y-position descending (top of page first in PDF coords)
    positioned.sort_by(|a, b| b.0.total_cmp(&a.0));

    // Build interleaved output: paragraphs and tables sorted by vertical position
    // Each paragraph's position is the baseline_y of its first line
    // In PDF coords, higher y = higher on page = earlier in reading order

    struct Element<'a> {
        y_pos: f32,
        content: ElementContent<'a>,
        para_idx: Option<usize>,
    }
    enum ElementContent<'a> {
        Paragraph(&'a PdfParagraph),
        Table(&'a str),
    }

    let mut elements: Vec<Element> = Vec::new();

    for (idx, para) in paragraphs.iter().enumerate() {
        // Skip captions — they are emitted after their parent
        if para.caption_for.is_some() {
            continue;
        }
        let y_pos = para.lines.first().map(|l| l.baseline_y).unwrap_or(0.0);
        elements.push(Element {
            y_pos,
            content: ElementContent::Paragraph(para),
            para_idx: Some(idx),
        });
    }

    for (y_pos, md) in &positioned {
        elements.push(Element {
            y_pos: *y_pos,
            content: ElementContent::Table(md),
            para_idx: None,
        });
    }

    // Sort by y descending (top of page first in PDF coordinates)
    elements.sort_by(|a, b| b.y_pos.total_cmp(&a.y_pos));

    let start_len = output.len();
    let mut prev_was_list_item = false;
    for elem in &elements {
        if output.len() > start_len {
            let curr_is_list_item = matches!(&elem.content, ElementContent::Paragraph(p) if p.is_list_item);
            if prev_was_list_item && curr_is_list_item {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }
        match &elem.content {
            ElementContent::Paragraph(para) => {
                prev_was_list_item = para.is_list_item;
                render_paragraph_to_output(para, output);
                // Emit captions associated with this paragraph
                if let Some(idx) = elem.para_idx {
                    emit_captions_for(output, paragraphs, idx);
                }
            }
            ElementContent::Table(md) => {
                prev_was_list_item = false;
                output.push_str(md);
            }
        }
    }

    // Append unpositioned tables at end of page
    for md in &unpositioned {
        if output.len() > start_len {
            output.push_str("\n\n");
        }
        output.push_str(md);
    }
}

/// Emit any caption paragraphs associated with the parent at `parent_idx`.
///
/// Captions are rendered in italics. The text is HTML-escaped before wrapping
/// to prevent `&`, `<`, `>`, and `_` from appearing raw in markdown output.
fn emit_captions_for(output: &mut String, paragraphs: &[PdfParagraph], parent_idx: usize) {
    for para in paragraphs {
        if para.caption_for == Some(parent_idx) {
            output.push('\n');
            // Collect raw text, escape HTML entities, then wrap in italics
            let text: String = para
                .lines
                .iter()
                .flat_map(|l| l.segments.iter())
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let escaped = escape_html_entities(trimmed);
                output.push('*');
                output.push_str(&escaped);
                output.push('*');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pdf::hierarchy::SegmentData;

    use super::super::types::PdfLine;
    use super::*;

    fn plain_segment(text: &str) -> SegmentData {
        SegmentData {
            text: text.to_string(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 12.0,
            font_size: 12.0,
            is_bold: false,
            is_italic: false,
            is_monospace: false,
            baseline_y: 700.0,
        }
    }

    fn make_paragraph(text: &str, heading_level: Option<u8>) -> PdfParagraph {
        make_paragraph_at(text, heading_level, 700.0)
    }

    fn make_paragraph_at(text: &str, heading_level: Option<u8>, baseline_y: f32) -> PdfParagraph {
        PdfParagraph {
            lines: vec![PdfLine {
                segments: vec![SegmentData {
                    baseline_y,
                    ..plain_segment(text)
                }],
                baseline_y,
                dominant_font_size: 12.0,
                is_bold: false,
                is_monospace: false,
            }],
            dominant_font_size: 12.0,
            heading_level,
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

    #[test]
    fn test_assemble_markdown_basic() {
        let pages = vec![vec![
            make_paragraph("Title", Some(1)),
            make_paragraph("Body text", None),
        ]];
        let result = assemble_markdown_with_tables(pages, &[], None);
        assert_eq!(result, "# Title\n\nBody text");
    }

    #[test]
    fn test_assemble_markdown_empty() {
        let result = assemble_markdown_with_tables(vec![], &[], None);
        assert_eq!(result, "");
    }

    #[test]
    fn test_assemble_markdown_multiple_pages() {
        let pages = vec![
            vec![make_paragraph("Page 1", None)],
            vec![make_paragraph("Page 2", None)],
        ];
        let result = assemble_markdown_with_tables(pages, &[], None);
        assert_eq!(result, "Page 1\n\nPage 2");
    }

    #[test]
    fn test_assemble_with_tables_no_tables() {
        let pages = vec![vec![make_paragraph("Body", None)]];
        let result = assemble_markdown_with_tables(pages, &[], None);
        assert_eq!(result, "Body");
    }

    #[test]
    fn test_assemble_with_tables_no_bbox() {
        let pages = vec![vec![make_paragraph("Before", None)]];
        let tables = vec![crate::types::Table {
            cells: vec![],
            markdown: "| A | B |\n|---|---|\n| 1 | 2 |".to_string(),
            page_number: 1,
            bounding_box: None,
        }];
        let result = assemble_markdown_with_tables(pages, &tables, None);
        assert!(result.starts_with("Before"));
        assert!(result.contains("| A | B |"));
    }

    #[test]
    fn test_assemble_with_tables_positioned() {
        // Paragraph at y=700 (top), table at y=500 (middle), paragraph at y=300 (bottom)
        let pages = vec![vec![
            make_paragraph_at("Top text", None, 700.0),
            make_paragraph_at("Bottom text", None, 300.0),
        ]];
        let tables = vec![crate::types::Table {
            cells: vec![],
            markdown: "| Col1 | Col2 |".to_string(),
            page_number: 1,
            bounding_box: Some(crate::types::BoundingBox {
                x0: 50.0,
                y0: 400.0,
                x1: 500.0,
                y1: 500.0,
            }),
        }];
        let result = assemble_markdown_with_tables(pages, &tables, None);
        let parts: Vec<&str> = result.split("\n\n").collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "Top text");
        assert_eq!(parts[1], "| Col1 | Col2 |");
        assert_eq!(parts[2], "Bottom text");
    }

    #[test]
    fn test_assemble_with_tables_multipage() {
        let pages = vec![
            vec![make_paragraph("Page 1", None)],
            vec![make_paragraph("Page 2", None)],
        ];
        let tables = vec![crate::types::Table {
            cells: vec![],
            markdown: "| Table |".to_string(),
            page_number: 2,
            bounding_box: None,
        }];
        let result = assemble_markdown_with_tables(pages, &tables, None);
        assert!(result.contains("Page 1"));
        assert!(result.contains("Page 2"));
        assert!(result.contains("| Table |"));
        // Table should be on page 2
        let page2_start = result.find("Page 2").unwrap();
        let table_pos = result.find("| Table |").unwrap();
        assert!(table_pos > page2_start);
    }

    #[test]
    fn test_page_markers_inserted_for_all_pages() {
        let pages = vec![
            vec![make_paragraph("Page 1 content", None)],
            vec![make_paragraph("Page 2 content", None)],
            vec![make_paragraph("Page 3 content", None)],
        ];
        let marker_fmt = "\n\n<!-- PAGE {page_num} -->\n\n";
        let result = assemble_markdown_with_tables(pages, &[], Some(marker_fmt));
        assert!(result.contains("<!-- PAGE 1 -->"));
        assert!(result.contains("<!-- PAGE 2 -->"));
        assert!(result.contains("<!-- PAGE 3 -->"));
        // Markers appear before page content
        let m1 = result.find("<!-- PAGE 1 -->").unwrap();
        let c1 = result.find("Page 1 content").unwrap();
        assert!(m1 < c1);
        let m2 = result.find("<!-- PAGE 2 -->").unwrap();
        let c2 = result.find("Page 2 content").unwrap();
        assert!(m2 < c2);
    }

    #[test]
    fn test_page_markers_custom_format() {
        let pages = vec![
            vec![make_paragraph("First", None)],
            vec![make_paragraph("Second", None)],
        ];
        let marker_fmt = "<page number=\"{page_num}\">";
        let result = assemble_markdown_with_tables(pages, &[], Some(marker_fmt));
        assert!(result.contains("<page number=\"1\">"));
        assert!(result.contains("<page number=\"2\">"));
    }

    #[test]
    fn test_no_markers_when_none() {
        let pages = vec![vec![make_paragraph("A", None)], vec![make_paragraph("B", None)]];
        let result = assemble_markdown_with_tables(pages, &[], None);
        assert!(!result.contains("PAGE"));
        assert!(!result.contains("page"));
        assert_eq!(result, "A\n\nB");
    }

    #[test]
    fn test_page_markers_with_tables() {
        let pages = vec![
            vec![make_paragraph("Page 1", None)],
            vec![make_paragraph("Page 2", None)],
        ];
        let tables = vec![crate::types::Table {
            cells: vec![],
            markdown: "| T |".to_string(),
            page_number: 2,
            bounding_box: None,
        }];
        let marker_fmt = "\n\n<!-- PAGE {page_num} -->\n\n";
        let result = assemble_markdown_with_tables(pages, &tables, Some(marker_fmt));
        assert!(result.contains("<!-- PAGE 1 -->"));
        assert!(result.contains("<!-- PAGE 2 -->"));
        assert!(result.contains("| T |"));
        // Table appears after page 2 marker
        let m2 = result.find("<!-- PAGE 2 -->").unwrap();
        let t = result.find("| T |").unwrap();
        assert!(t > m2);
    }
}
