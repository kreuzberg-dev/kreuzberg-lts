//! Heuristic table extraction from layout-detected Table regions.

/// Upward margin (in PDF points) applied when tightening the table bbox top.
///
/// After identifying the first actual table content row (the topmost word row
/// that has a horizontal column gap), this margin is subtracted from that row's
/// image-y before converting to PDF coords, ensuring the first row's glyphs are
/// fully inside the filter bbox.
const TABLE_BBOX_TOP_TIGHTEN_MARGIN_PTS: u32 = 4;

use crate::pdf::structure::text_repair::repair_broken_word_spacing;
use crate::pdf::structure::types::{LayoutHint, LayoutHintClass};
use crate::pdf::table_reconstruct::{
    is_well_formed_table, looks_like_code_listing, post_process_table, reconstruct_table, table_to_markdown,
};
use crate::types::Table;

use super::table_recognition::word_hint_iow;

/// Extract tables from layout-detected Table regions using word-level data.
///
/// Filters the provided words by Table hint bboxes and reconstructs table
/// structure using heuristic column/row detection. The caller is responsible
/// for providing words (typically from `segments_to_words` for consistency
/// with region assembly, or from `extract_words_from_page` as fallback).
pub(in crate::pdf::structure) fn extract_tables_from_layout_hints(
    words: &[crate::pdf::table_reconstruct::HocrWord],
    hints: &[LayoutHint],
    page_index: usize,
    page_height: f32,
    min_confidence: f32,
    allow_single_column: bool,
) -> Vec<Table> {
    use crate::pdf::table_reconstruct::HocrWord;

    let table_hints: Vec<&LayoutHint> = hints
        .iter()
        .filter(|h| h.class_name == LayoutHintClass::Table && h.confidence >= min_confidence)
        .collect();

    if table_hints.is_empty() {
        return Vec::new();
    }

    let mut tables = Vec::new();

    for hint in &table_hints {
        // Filter words that overlap the table hint bbox (≥20% of word area).
        // HocrWord uses image coordinates (y=0 at top), while hint uses PDF
        // coordinates (y=0 at bottom). Convert hint bbox to image coords.
        let hint_img_top = (page_height - hint.top).max(0.0);
        let hint_img_bottom = (page_height - hint.bottom).max(0.0);

        tracing::trace!(
            hint_left = hint.left,
            hint_right = hint.right,
            hint_top = hint.top,
            hint_bottom = hint.bottom,
            hint_img_top,
            hint_img_bottom,
            page_height,
            total_words = words.len(),
            "table hint bbox (PDF→image coords)"
        );
        if let Some(first_word) = words.first() {
            tracing::trace!(
                word_text = %first_word.text,
                word_left = first_word.left,
                word_top = first_word.top,
                word_width = first_word.width,
                word_height = first_word.height,
                "first word coords (image coords)"
            );
        }

        let table_words: Vec<HocrWord> = words
            .iter()
            .filter(|w| {
                if w.text.trim().is_empty() {
                    return false;
                }
                word_hint_iow(w, hint.left, hint_img_top, hint.right, hint_img_bottom) >= 0.2
            })
            .cloned()
            .collect();

        tracing::trace!(matched_words = table_words.len(), "words overlapping table hint");

        // Need at least 4 words for a meaningful table
        if table_words.len() < 4 {
            continue;
        }

        // Adaptive column gap threshold based on word spacing statistics
        // within the table region. Use the median inter-word gap on the same
        // line as a baseline, then require a gap > 2x median to split columns.
        // Falls back to width-based scaling when not enough data.
        let table_width = hint.right - hint.left;
        let col_gap = compute_adaptive_column_gap(&table_words, table_width);
        let table_cells = reconstruct_table(&table_words, col_gap, 0.5);

        if table_cells.is_empty() || table_cells[0].is_empty() {
            continue;
        }

        // How many columns the initial reconstruction found.  Used below to
        // distinguish multi-column table rows from header paragraphs that happen
        // to have two text blocks.  A 4-column table requires 2 gaps per row to
        // qualify as "a table row"; a 2-column table only requires 1.
        let num_table_cols = table_cells[0].len();
        let min_column_gaps = (num_table_cols / 2).max(1);

        // Bounding box derived from the layout hint, but with the top edge tightened
        // to the actual topmost TABLE ROW — not merely the topmost word in the hint.
        // The raw hint top often extends above the visible table grid to cover an
        // adjacent paragraph (e.g., a bold metadata header on election-results pages).
        //
        // We detect pre-table paragraphs by scanning word rows top-to-bottom and
        // looking for the first row that has a horizontal gap ≥ col_gap between
        // consecutive words (column structure). Rows with no such gap are flowing
        // text (paragraphs), not table rows, and are excluded from the bbox, letting
        // them survive as ordinary paragraphs in the extraction pipeline.
        //
        // HocrWord uses image coordinates (u32, y=0 at top); convert to PDF coords
        // via pdf_y = page_height − img_y. A small upward margin is added so the
        // first row's top edge is fully covered.
        let tightened_y1: f64 = {
            const SAME_ROW_TOLERANCE_PTS: u32 = 5;

            // Sort words by image-y (ascending = topmost on page first).
            let mut sorted: Vec<&HocrWord> = table_words.iter().collect();
            sorted.sort_by_key(|w| w.top);

            // Walk the sorted words, grouping into rows (within SAME_ROW_TOLERANCE_PTS
            // of the row's anchor top).  For each row, check whether any consecutive
            // pair of words (sorted by left edge) has a gap ≥ col_gap — if so, this
            // row has column structure and is the actual first table content row.
            let mut first_table_row_top: Option<u32> = None;
            let mut row_start = 0_usize;
            while row_start < sorted.len() {
                let row_anchor = sorted[row_start].top;
                // Find the exclusive end of this row.
                let row_end = sorted[row_start..]
                    .iter()
                    .position(|w| w.top.saturating_sub(row_anchor) > SAME_ROW_TOLERANCE_PTS)
                    .map(|p| row_start + p)
                    .unwrap_or(sorted.len());

                // Sort this row's words by left edge and check for a column-sized gap.
                let mut left_rights: Vec<(u32, u32)> = sorted[row_start..row_end]
                    .iter()
                    .map(|w| (w.left, w.left + w.width))
                    .collect();
                left_rights.sort_by_key(|&(l, _)| l);
                let n_col_gaps = left_rights
                    .windows(2)
                    .filter(|pair| pair[1].0.saturating_sub(pair[0].1) >= col_gap)
                    .count();
                if n_col_gaps >= min_column_gaps {
                    first_table_row_top = Some(row_anchor);
                    break;
                }
                row_start = row_end;
            }

            let img_top = first_table_row_top.unwrap_or(hint_img_top as u32);
            let img_top_with_margin = img_top.saturating_sub(TABLE_BBOX_TOP_TIGHTEN_MARGIN_PTS);
            let pdf_top = page_height - img_top_with_margin as f32;
            // Never extend the bbox beyond the original hint top.
            (pdf_top as f64).min(hint.top as f64)
        };
        let bounding_box = Some(crate::types::BoundingBox {
            x0: hint.left as f64,
            y0: hint.bottom as f64,
            x1: hint.right as f64,
            y1: tightened_y1,
        });

        // Validate with layout_guided=true (relaxed thresholds)
        let table_cells = match post_process_table(table_cells, true, allow_single_column) {
            Some(cleaned) => cleaned,
            None => {
                // Table reconstruction failed — the Table hint was a false positive.
                // Do NOT emit a table with bounding_box: that would add the bbox to
                // extracted_table_bboxes_by_page, suppressing legitimate text segments
                // in assign_segments_to_regions (IoS >= 0.5 check). Instead, skip this
                // hint entirely and let the text fall through as unassigned segments
                // in the normal pipeline.
                tracing::trace!(
                    page = page_index,
                    hint_left = hint.left,
                    hint_right = hint.right,
                    words = table_words.len(),
                    "table reconstruction failed — skipping false-positive Table hint"
                );
                continue;
            }
        };

        // Reject single-row tables — these are almost always false positives
        // from the layout model (e.g., a line of text misclassified as Table).
        if table_cells.len() <= 1 {
            tracing::trace!(
                page = page_index,
                rows = table_cells.len(),
                "table has <=1 row — skipping likely false-positive Table hint"
            );
            continue;
        }

        // Reject tables with very few rows whose bbox covers most of the page.
        // This catches body text that the layout model misclassifies as a Table:
        // the table reconstructor splits prose into columns producing a wide,
        // page-spanning "table" with only 2–3 rows. Real tables with few rows
        // are compact and don't cover >50% of the page height.
        let hint_height = (hint.top - hint.bottom).abs();
        if table_cells.len() <= 3 && page_height > 0.0 && hint_height / page_height > 0.5 {
            tracing::trace!(
                page = page_index,
                rows = table_cells.len(),
                hint_height,
                page_height,
                ratio = hint_height / page_height,
                "table with <=3 rows spans >50% of page height — skipping likely false-positive"
            );
            continue;
        }

        // Reject degenerate tables with too many empty cells.
        // False-positive Table hints (e.g. in RTL documents) often produce
        // tables where most cells are empty because the content is not truly
        // tabular. Skip these to avoid polluting output with markdown table
        // formatting characters that hurt TF1.
        // Use 55% threshold: real tables often have sparse optional columns
        // (e.g., footnote markers, units), especially in scientific/financial docs.
        let total_cells: usize = table_cells.iter().map(|row| row.len()).sum();
        let empty_cells: usize = table_cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.trim().is_empty())
            .count();
        if total_cells > 0 && empty_cells as f64 / total_cells as f64 > 0.55 {
            tracing::trace!(
                page = page_index,
                total_cells,
                empty_cells,
                "table has >40% empty cells — skipping degenerate table"
            );
            continue;
        }

        // Reject tables where total text content is very short relative to
        // the number of cells. This catches false positives where a small
        // amount of text is spread across a table grid.
        let total_text_len: usize = table_cells
            .iter()
            .flat_map(|row| row.iter())
            .map(|cell| cell.trim().len())
            .sum();
        if total_cells > 6 && total_text_len < total_cells {
            tracing::trace!(
                page = page_index,
                total_cells,
                total_text_len,
                "table text content too sparse — skipping degenerate table"
            );
            continue;
        }

        // Reject tables where most rows have only 1 filled cell.
        // This pattern indicates non-tabular content forced into a grid
        // (e.g., RTL text where each line becomes a "row" with one cell).
        if table_cells.len() >= 3 {
            let single_cell_rows = table_cells
                .iter()
                .filter(|row| row.iter().filter(|c| !c.trim().is_empty()).count() <= 1)
                .count();
            if single_cell_rows as f64 / table_cells.len() as f64 > 0.5 {
                tracing::trace!(
                    page = page_index,
                    rows = table_cells.len(),
                    single_cell_rows,
                    "table has >50% single-cell rows — skipping likely false-positive"
                );
                continue;
            }
        }

        // Reject tables that look like code listings.
        // The layout model sometimes misclassifies monospace code blocks (JS,
        // Rust, Go, Java, C, …) as Table regions because the character-level
        // spacing in a fixed-width font creates apparent column positions that
        // fool the heuristic grid detector. Curly braces — especially isolated
        // `{` or `}` cells (from opening/closing lines) — are the most reliable
        // signal: they appear in virtually all C-family code but never in real
        // table data.
        if looks_like_code_listing(&table_cells) {
            tracing::trace!(
                page = page_index,
                rows = table_cells.len(),
                cols = table_cells.first().map_or(0, |r| r.len()),
                "table region looks like a code listing — skipping false-positive Table hint"
            );
            continue;
        }

        // Table quality validation: reject tables that are actually multi-column
        // prose, repeated page elements, or low-vocabulary repetitive content.
        if !is_well_formed_table(&table_cells) {
            tracing::trace!(
                page = page_index,
                rows = table_cells.len(),
                cols = table_cells.first().map_or(0, |r| r.len()),
                "table failed quality validation — skipping as prose"
            );
            continue;
        }

        // Repair broken word spacing per-cell before rendering to markdown
        let repaired_cells: Vec<Vec<String>> = table_cells
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| repair_broken_word_spacing(cell).into_owned())
                    .collect()
            })
            .collect();
        let markdown = table_to_markdown(&repaired_cells);

        tracing::trace!(
            page = page_index,
            rows = table_cells.len(),
            total_cells,
            empty_cells,
            total_text_len,
            markdown_len = markdown.len(),
            "table accepted"
        );

        tables.push(Table {
            cells: table_cells,
            markdown,
            page_number: (page_index + 1) as u32,
            bounding_box,
        });
    }

    tables
}

/// Compute an adaptive column gap threshold based on word spacing within the
/// table region.
///
/// Sorts words into approximate rows (by y-center), then measures gaps between
/// consecutive words on each row. The median gap represents typical word spacing;
/// we use 2x that as the column threshold (columns have wider gaps than words).
///
/// Falls back to width-based scaling when there aren't enough same-row word
/// pairs to compute meaningful statistics.
pub(crate) fn compute_adaptive_column_gap(words: &[crate::pdf::table_reconstruct::HocrWord], table_width: f32) -> u32 {
    // Collect inter-word gaps on approximate rows
    let mut gaps: Vec<u32> = Vec::new();

    if words.len() >= 4 {
        // Group words by approximate y-center (within median height / 2)
        let mut heights: Vec<u32> = words.iter().map(|w| w.height).collect();
        heights.sort_unstable();
        let median_h = heights[heights.len() / 2];
        let row_tolerance = (median_h / 2).max(3);

        // Sort words by y-center then x
        let mut sorted: Vec<(u32, u32, u32)> = words
            .iter()
            .map(|w| {
                let yc = w.top + w.height / 2;
                (yc, w.left, w.left + w.width)
            })
            .collect();
        sorted.sort_by_key(|&(yc, x, _)| (yc, x));

        // Walk sorted words, group by approximate row
        let mut row_start = 0;
        while row_start < sorted.len() {
            let row_yc = sorted[row_start].0;
            let mut row_end = row_start + 1;
            while row_end < sorted.len() && sorted[row_end].0.abs_diff(row_yc) <= row_tolerance {
                row_end += 1;
            }

            // Measure gaps between consecutive words on this row
            for i in row_start + 1..row_end {
                let prev_right = sorted[i - 1].2;
                let curr_left = sorted[i].1;
                if curr_left > prev_right {
                    gaps.push(curr_left - prev_right);
                }
            }

            row_start = row_end;
        }
    }

    if gaps.len() >= 3 {
        gaps.sort_unstable();

        // Filter out intra-cell word gaps (typically <30px) to focus on inter-cell gaps.
        // Large gaps (>40px) are more likely to be column separators.
        // This prevents small intra-cell spacing from dominating the calculation.
        let large_gaps: Vec<u32> = gaps.iter().copied().filter(|&g| g >= 40).collect();

        if !large_gaps.is_empty() {
            // Use median of large gaps as the column boundary threshold.
            // `gaps` is already sorted, so the filtered subset stays sorted.
            let median_gap = large_gaps[large_gaps.len() / 2];
            let threshold = (median_gap / 2).clamp(20, 60);
            return threshold;
        } else {
            // Fallback: use all gaps but with safer bounds
            let median_gap = gaps[gaps.len() / 2];
            let threshold = (median_gap * 3).clamp(20, 60);
            return threshold;
        }
    }

    // Fallback: width-based scaling with tighter defaults for narrow tables
    if table_width < 200.0 {
        10
    } else if table_width < 400.0 {
        15
    } else if table_width < 600.0 {
        20
    } else {
        30
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::table_reconstruct::{HocrWord, looks_like_code_listing};

    fn make_word(text: &str, left: u32, top: u32, width: u32, height: u32) -> HocrWord {
        HocrWord {
            text: text.to_string(),
            left,
            top,
            width,
            height,
            confidence: 95.0,
        }
    }

    fn make_table_hint(confidence: f32, left: f32, bottom: f32, right: f32, top: f32) -> LayoutHint {
        LayoutHint {
            class_name: LayoutHintClass::Table,
            confidence,
            left,
            bottom,
            right,
            top,
        }
    }

    #[test]
    fn test_no_table_hints_returns_empty() {
        let words = vec![make_word("hello", 10, 10, 50, 12)];
        let hints = vec![LayoutHint {
            class_name: LayoutHintClass::Text,
            confidence: 0.9,
            left: 0.0,
            bottom: 0.0,
            right: 600.0,
            top: 800.0,
        }];
        let tables = extract_tables_from_layout_hints(&words, &hints, 0, 800.0, 0.5, false);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_low_confidence_table_hint_filtered() {
        let words = vec![
            make_word("A", 10, 10, 50, 12),
            make_word("B", 100, 10, 50, 12),
            make_word("C", 10, 30, 50, 12),
            make_word("D", 100, 30, 50, 12),
        ];
        let hints = vec![make_table_hint(0.3, 0.0, 0.0, 200.0, 800.0)];
        // min_confidence = 0.5, hint has 0.3 → filtered
        let tables = extract_tables_from_layout_hints(&words, &hints, 0, 800.0, 0.5, false);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_empty_region_too_few_words() {
        // Only 2 words in the region — below the 4-word minimum
        let words = vec![make_word("A", 10, 10, 50, 12), make_word("B", 100, 10, 50, 12)];
        let hints = vec![make_table_hint(0.9, 0.0, 0.0, 200.0, 800.0)];
        let tables = extract_tables_from_layout_hints(&words, &hints, 0, 800.0, 0.5, false);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_empty_words_returns_empty() {
        let hints = vec![make_table_hint(0.9, 0.0, 0.0, 200.0, 800.0)];
        let tables = extract_tables_from_layout_hints(&[], &hints, 0, 800.0, 0.5, false);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_no_hints_returns_empty() {
        let words = vec![
            make_word("A", 10, 10, 50, 12),
            make_word("B", 100, 10, 50, 12),
            make_word("C", 10, 30, 50, 12),
            make_word("D", 100, 30, 50, 12),
        ];
        let tables = extract_tables_from_layout_hints(&words, &[], 0, 800.0, 0.5, false);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_words_outside_hint_bbox_excluded() {
        // Words at (500, 500) are far from the hint bbox
        let words = vec![
            make_word("A", 500, 500, 50, 12),
            make_word("B", 560, 500, 50, 12),
            make_word("C", 500, 520, 50, 12),
            make_word("D", 560, 520, 50, 12),
        ];
        // Hint covers (0, 0) to (100, 100) in PDF coords → image y = 700..800
        let hints = vec![make_table_hint(0.9, 0.0, 700.0, 100.0, 800.0)];
        let tables = extract_tables_from_layout_hints(&words, &hints, 0, 800.0, 0.5, false);
        // Words at (500, 500) don't overlap the hint → too few words → empty
        assert!(tables.is_empty());
    }

    #[test]
    fn test_whitespace_only_words_filtered() {
        let words = vec![
            make_word("  ", 10, 10, 50, 12),
            make_word("A", 100, 10, 50, 12),
            make_word("B", 10, 30, 50, 12),
            make_word("C", 100, 30, 50, 12),
        ];
        // Only 3 non-empty words → below 4-word minimum
        let hints = vec![make_table_hint(0.9, 0.0, 0.0, 200.0, 800.0)];
        let tables = extract_tables_from_layout_hints(&words, &hints, 0, 800.0, 0.5, false);
        assert!(tables.is_empty());
    }

    #[test]
    fn test_page_number_is_one_indexed() {
        // Construct words that form a valid 2-column, multi-row table
        // Rows at y=10 and y=40 in image coords, columns at x=10 and x=200
        let words = vec![
            make_word("Header1", 10, 10, 80, 15),
            make_word("Header2", 200, 10, 80, 15),
            make_word("Cell1", 10, 40, 80, 15),
            make_word("Cell2", 200, 40, 80, 15),
            make_word("Cell3", 10, 70, 80, 15),
            make_word("Cell4", 200, 70, 80, 15),
        ];
        // Hint in PDF coords: bottom=700, top=800 → image top=0, image bottom=100
        let hints = vec![make_table_hint(0.9, 0.0, 700.0, 400.0, 800.0)];
        let tables = extract_tables_from_layout_hints(&words, &hints, 2, 800.0, 0.5, false);
        // If a valid table is produced, its page_number should be page_index + 1
        for table in &tables {
            assert_eq!(table.page_number, 3); // page_index=2 → page_number=3
        }
    }

    /// When a Table hint covers both a spanning paragraph above the table and the
    /// actual table grid below, the bbox top must be tightened to exclude the
    /// paragraph.  The paragraph row has no horizontal gap ≥ col_gap between its
    /// words; the first actual table row does.
    ///
    /// This mimics the la-precinct-bulletin-2014-p1 layout failure where a bold
    /// metadata header ("Precinct RUN 12/3/2014 11:57:01 AM ...") was being
    /// filtered by `filter_segments_by_table_bboxes` because the raw Table hint
    /// bbox extended above the actual table grid.
    #[test]
    fn test_bbox_top_tightened_to_first_table_row_skipping_paragraph_header() {
        // Page dimensions (PDF pts, y=0 at bottom).
        let page_height: f32 = 800.0;

        // Paragraph header row at image y=5..17 (PDF y ≈ 783..795).
        // Three words with small inter-word gaps (5 pts) — no column structure.
        let mut all_words: Vec<HocrWord> = vec![
            make_word("Pre", 50, 5, 30, 12),
            make_word("Cin", 85, 5, 30, 12),
            make_word("ct", 120, 5, 20, 12),
        ];

        // Three-column table at image y=40..112 (PDF y ≈ 688..760).
        // Columns are separated by ~160-pt gaps, far exceeding col_gap.
        // Cells are kept short (≤2 chars each; concatenated row ≤8 chars) so
        // that `is_well_formed_table`'s prose-row check (row concat ≥15 chars)
        // is never triggered, and col-uniformity check (mean cell len > 3 chars)
        // is also skipped.
        let table_rows: [(&str, &str, &str, u32); 5] = [
            ("AA", "DD", "GG", 40),
            ("BB", "EE", "HH", 60),
            ("CC", "FF", "II", 80),
            ("AB", "DE", "GH", 100),
            ("BC", "EF", "HI", 112),
        ];
        for (c1, c2, c3, y) in &table_rows {
            all_words.push(make_word(c1, 50, *y, 30, 12));
            all_words.push(make_word(c2, 250, *y, 30, 12));
            all_words.push(make_word(c3, 450, *y, 30, 12));
        }

        // The Table hint covers the entire area including the paragraph header.
        // PDF: left=40, bottom=690, right=510, top=800.
        // Image: top=0, bottom=110 → every word is inside the hint.
        let hints = vec![make_table_hint(0.9, 40.0, 690.0, 510.0, 800.0)];

        let tables = extract_tables_from_layout_hints(&all_words, &hints, 0, page_height, 0.5, true);

        // A table must be produced.
        assert!(!tables.is_empty(), "expected a table to be reconstructed");

        let bb = tables[0].bounding_box.as_ref().expect("table must have a bounding_box");

        // The tightened y1 (top of the filter bbox in PDF coords) must be below
        // the paragraph header (PDF y_bottom ≈ 800−17 = 783).
        // First table row at image top=40 → pdf_top = 800−(40−4) = 764.
        // So bb.y1 must be ≈ 764, well below 783.
        assert!(
            bb.y1 < 775.0,
            "bbox top (y1={:.1}) should be tightened below the paragraph header \
             (header PDF y_bottom ≈ 783)",
            bb.y1
        );

        // The bbox bottom must still encompass the full table grid.
        assert!(
            bb.y0 <= 695.0,
            "bbox bottom (y0={:.1}) should still cover the table area",
            bb.y0
        );
    }

    // --- Tests for looks_like_code_listing ---

    #[test]
    fn test_code_listing_with_isolated_closing_brace_is_rejected() {
        // Simulates a JS code block reconstructed into a table grid:
        //
        //   function | add(a, b) | {
        //   return   | a + b;    |
        //   }        |           |
        //
        // The isolated `}` cell (from the closing line of the code block)
        // is the hard-reject signal: no real table has a bare `}` cell.
        let table_cells = vec![
            vec!["function".to_string(), "add(a, b)".to_string(), "{".to_string()],
            vec!["return".to_string(), "a + b;".to_string(), "".to_string()],
            vec!["}".to_string(), "".to_string(), "".to_string()],
        ];
        assert!(
            looks_like_code_listing(&table_cells),
            "grid with isolated `}}` cell should be detected as code listing"
        );
    }

    #[test]
    fn test_code_listing_with_opening_brace_only_is_rejected() {
        // Opening brace `{` alone in a cell is equally specific to code.
        let table_cells = vec![
            vec!["if".to_string(), "(x > 0)".to_string(), "{".to_string()],
            vec!["".to_string(), "return".to_string(), "x".to_string()],
            vec!["".to_string(), "}".to_string(), "".to_string()],
        ];
        assert!(
            looks_like_code_listing(&table_cells),
            "grid with isolated `{{` or `}}` cell should be detected as code listing"
        );
    }

    #[test]
    fn test_code_listing_with_inline_braces_fraction_is_rejected() {
        // Code where braces appear mid-cell (not isolated):
        //   if (x) { | return x; }
        //   else {   | return y; }
        // Two out of four non-empty cells contain `{` or `}` → 50% ≥ 20% threshold.
        let table_cells = vec![
            vec!["if (x) {".to_string(), "return x; }".to_string()],
            vec!["else {".to_string(), "return y; }".to_string()],
        ];
        assert!(
            looks_like_code_listing(&table_cells),
            "grid with ≥20% of cells containing braces should be detected as code listing"
        );
    }

    #[test]
    fn test_genuine_data_table_is_not_rejected() {
        // A real 2-column table with header and data rows must NOT be suppressed.
        let table_cells = vec![
            vec!["Name".to_string(), "Score".to_string()],
            vec!["Alice".to_string(), "95".to_string()],
            vec!["Bob".to_string(), "87".to_string()],
            vec!["Carol".to_string(), "91".to_string()],
        ];
        assert!(
            !looks_like_code_listing(&table_cells),
            "genuine data table must not be classified as a code listing"
        );
    }

    #[test]
    fn test_table_with_parenthesised_values_is_not_rejected() {
        // Documentation tables sometimes have function-call notation in cells
        // like `to_string()` or `from_str(s)`. These contain `(` and `)` but
        // no curly braces, so they must not trigger the code-listing guard.
        let table_cells = vec![
            vec!["Function".to_string(), "Description".to_string()],
            vec!["to_string()".to_string(), "Converts to string".to_string()],
            vec!["from_str(s)".to_string(), "Creates from string".to_string()],
            vec!["parse()".to_string(), "Parses the value".to_string()],
        ];
        assert!(
            !looks_like_code_listing(&table_cells),
            "table with parenthesised function names must not be classified as code"
        );
    }

    #[test]
    fn test_empty_table_cells_is_not_rejected() {
        // A degenerate all-empty table should not crash or falsely fire the check.
        let table_cells: Vec<Vec<String>> = vec![
            vec!["".to_string(), "".to_string()],
            vec!["".to_string(), "".to_string()],
        ];
        assert!(
            !looks_like_code_listing(&table_cells),
            "all-empty table must not be classified as code listing"
        );
    }
}
