//! Table structure recognition for native PDF pages (TATR + SLANeXT backends).

use super::super::geometry::Rect;

#[cfg(feature = "layout-detection")]
use crate::pdf::structure::types::{LayoutHint, LayoutHintClass};
#[cfg(feature = "layout-detection")]
use crate::types::Table;
#[cfg(feature = "layout-detection")]
use crate::utils::escape_html_entities;

/// Compute intersection-over-word-area between an HocrWord and a rectangular region.
///
/// Both word and region must be in the same coordinate space (image coords).
pub(in crate::pdf::structure) fn word_hint_iow(
    w: &crate::pdf::table_reconstruct::HocrWord,
    region_left: f32,
    region_top: f32,
    region_right: f32,
    region_bottom: f32,
) -> f32 {
    let word_rect = Rect::from_xywh(w.left as f32, w.top as f32, w.width as f32, w.height as f32);
    let region_rect = Rect::from_ltrb(region_left, region_top, region_right, region_bottom);
    if word_rect.area() <= 0.0 {
        // Zero-area word: fall back to center-point containment (0 or 1)
        return if region_rect.contains_point(word_rect.center_x(), word_rect.center_y()) {
            1.0
        } else {
            0.0
        };
    }
    word_rect.intersection_over_self(&region_rect)
}

/// Recognize tables on a native PDF page using TATR structure prediction.
///
/// Crops table regions from the rendered layout detection image, runs TATR
/// inference, then matches predicted cell bboxes against native PDF words.
///
/// # Coordinate conversion
///
/// Three coordinate spaces are involved:
/// - **PDF coords**: LayoutHint bboxes and HocrWord positions (y=0 at bottom for hints;
///   HocrWord uses image-coords with y=0 at top, converted via `page_height - pdf_top`).
/// - **Rendered image pixels**: The ~640px image used for layout detection.
/// - **TATR crop pixels**: Cell bboxes relative to the cropped table region.
#[cfg(feature = "layout-detection")]
pub(in crate::pdf::structure) fn recognize_tables_for_native_page(
    page_image: &image::RgbImage,
    hints: &[LayoutHint],
    words: &[crate::pdf::table_reconstruct::HocrWord],
    page_result: &crate::pdf::structure::types::PageLayoutResult,
    page_height: f32,
    page_index: usize,
    tatr_model: &mut crate::layout::models::tatr::TatrModel,
) -> Vec<Table> {
    let rgb_image = page_image;
    let img_w = rgb_image.width();
    let img_h = rgb_image.height();

    // Scale factors: PDF points → rendered image pixels
    let sx = img_w as f32 / page_result.page_width_pts;
    let sy = img_h as f32 / page_result.page_height_pts;

    let table_hints: Vec<&LayoutHint> = hints
        .iter()
        .filter(|h| {
            if h.class_name != LayoutHintClass::Table || h.confidence < 0.5 {
                return false;
            }
            // Structural hint guard relaxed: region assignment now handles
            // text/table overlap correctly by assigning segments to Table
            // regions instead of suppressing them. Small tables on structured
            // pages are now allowed through since double-counting is prevented
            // by the region-first assembly approach.
            true
        })
        .collect();

    let mut tables = Vec::new();

    for hint in &table_hints {
        // Convert hint bbox from PDF coords to rendered image pixel coords.
        // PDF: y=0 at bottom, increases upward.
        // Image: y=0 at top, increases downward.
        let px_left = (hint.left * sx).round().max(0.0) as u32;
        let px_top = ((page_height - hint.top) * sy).round().max(0.0) as u32;
        let px_right = (hint.right * sx).round().min(img_w as f32) as u32;
        let px_bottom = ((page_height - hint.bottom) * sy).round().min(img_h as f32) as u32;

        let crop_w = px_right.saturating_sub(px_left);
        let crop_h = px_bottom.saturating_sub(px_top);

        if crop_w < 10 || crop_h < 10 {
            continue;
        }

        // Guard: skip TATR on extremely large crops that would slow inference.
        // DETR preprocessing resizes the crop (shortest edge → 800, cap 1333),
        // so even large crops are feasible; 4M pixels (~2000x2000) is generous
        // enough for tables rendered from the ~640px layout image.
        if (crop_w as u64) * (crop_h as u64) > 4_000_000 {
            tracing::debug!(
                page = page_index,
                crop_w,
                crop_h,
                "Skipping TATR for oversized table crop"
            );
            continue;
        }

        // Crop table region from rendered image
        let cropped = image::imageops::crop_imm(rgb_image, px_left, px_top, crop_w, crop_h).to_image();

        // Run TATR inference
        let tatr_result = match tatr_model.recognize(&cropped) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("TATR inference failed for table on page {}: {e}", page_index);
                continue;
            }
        };

        // Check if TATR detected any rows and columns
        if tatr_result.rows.is_empty() || tatr_result.columns.is_empty() {
            tracing::debug!(
                page = page_index,
                rows = tatr_result.rows.len(),
                columns = tatr_result.columns.len(),
                "TATR: no rows or columns detected"
            );
            continue;
        }

        // Build cell grid from row × column intersections.
        // Pass the table hint bbox converted to crop-relative pixel coords
        // so that rows are widened to the full table extent.
        let table_bbox_crop = [0.0_f32, 0.0, crop_w as f32, crop_h as f32];
        let cell_grid = crate::layout::models::tatr::build_cell_grid(&tatr_result, Some(table_bbox_crop));
        let num_rows = cell_grid.len();
        let num_cols = if num_rows > 0 { cell_grid[0].len() } else { 0 };

        tracing::debug!(
            page = page_index,
            detected_rows = tatr_result.rows.len(),
            detected_columns = tatr_result.columns.len(),
            grid_rows = num_rows,
            grid_cols = num_cols,
            crop = format!("{}x{}", crop_w, crop_h),
            "TATR inference result"
        );

        if num_rows == 0 || num_cols == 0 {
            continue;
        }

        // Filter words that overlap the table hint bbox (≥20% of word area).
        // HocrWord uses image coordinates (y=0 at top).
        // Pad the hint bbox slightly (3% width, 2% height) so edge words
        // (e.g. row numbers at the left margin) are not excluded by a
        // tight-fitting RT-DETR bbox.
        let hint_width = hint.right - hint.left;
        let hint_height = hint.top - hint.bottom;
        let pad_x = hint_width * 0.03;
        let pad_y = hint_height * 0.02;
        let padded_left = (hint.left - pad_x).max(0.0);
        let padded_right = hint.right + pad_x;
        let padded_top_pdf = hint.top + pad_y;
        let padded_bottom_pdf = (hint.bottom - pad_y).max(0.0);

        let hint_img_top = (page_height - padded_top_pdf).max(0.0);
        let hint_img_bottom = (page_height - padded_bottom_pdf).max(0.0);

        let table_words: Vec<&crate::pdf::table_reconstruct::HocrWord> = words
            .iter()
            .filter(|w| {
                if w.text.trim().is_empty() {
                    return false;
                }
                word_hint_iow(w, padded_left, hint_img_top, padded_right, hint_img_bottom) >= 0.2
            })
            .collect();

        // Match words to cells and build markdown table.
        // Cell bboxes are in crop-pixel space; words are in PDF coords.
        // Convert cell bboxes to PDF coords for matching.
        let (grid, markdown) = build_tatr_grid_table(&cell_grid, &table_words, px_left as f32, px_top as f32, sx, sy);

        tracing::debug!(
            page = page_index,
            table_words = table_words.len(),
            grid_rows = grid.len(),
            grid_cols = grid.first().map_or(0, |r| r.len()),
            markdown_len = markdown.len(),
            "TATR: word matching and markdown generation"
        );
        if markdown.is_empty() {
            tracing::debug!(page = page_index, "TATR: empty markdown output");
            continue;
        }

        // Validate: reject TATR output if too few cells have content.
        let total_cells = num_rows * num_cols;
        let filled_cells = grid
            .iter()
            .flat_map(|r| r.iter())
            .filter(|c| !c.trim().is_empty())
            .count();
        if total_cells > 4 && filled_cells < total_cells / 4 {
            tracing::debug!(
                page = page_index,
                total_cells,
                filled_cells,
                "TATR table rejected: too few filled cells"
            );
            continue;
        }

        // Tighten the top edge of the bbox to the first row that has genuine
        // column structure.  The hint top sometimes covers a metadata header
        // above the table (e.g. ballot headers on election pages), causing
        // filter_segments_by_table_bboxes to suppress those paragraphs.
        let table_width = hint.right - hint.left;
        let col_gap_for_tighten = compute_col_gap_for_word_refs(&table_words, table_width);
        let tatr_num_cols = grid.first().map_or(0, |r| r.len());
        // Require at least half the table's column gaps per row: header rows with
        // 1–2 text blocks have fewer gaps than rows inside the actual table.
        let min_column_gaps = (tatr_num_cols / 2).max(1);
        let tightened_y1 = tighten_table_bbox_top(
            &table_words,
            (page_height - hint.top).max(0.0),
            hint.top,
            col_gap_for_tighten,
            min_column_gaps,
            page_height,
        );

        let bounding_box = Some(crate::types::BoundingBox {
            x0: hint.left as f64,
            y0: hint.bottom as f64,
            x1: hint.right as f64,
            y1: tightened_y1,
        });

        tables.push(Table {
            cells: grid,
            markdown,
            page_number: (page_index + 1) as u32,
            bounding_box,
        });
    }

    tables
}

/// Build markdown table from TATR cell grid + PDF words.
///
/// Cell bboxes are in crop-pixel space. Words are in PDF image-coord space
/// (HocrWord: left in PDF x-units, top = page_height - pdf_top).
/// Converts cell coords to word space via crop offset + scale factors.
///
/// Uses best-match assignment: each word is assigned to the single cell with
/// the highest IoW overlap, preventing duplication across cells.
#[cfg(feature = "layout-detection")]
fn build_tatr_grid_table(
    cell_grid: &[Vec<crate::layout::models::tatr::CellBBox>],
    words: &[&crate::pdf::table_reconstruct::HocrWord],
    crop_offset_px_x: f32,
    crop_offset_px_y: f32,
    sx: f32,
    sy: f32,
) -> (Vec<Vec<String>>, String) {
    if cell_grid.is_empty() {
        return (Vec::new(), String::new());
    }

    let num_rows = cell_grid.len();
    let num_cols = cell_grid[0].len();
    if num_cols == 0 {
        return (Vec::new(), String::new());
    }

    // Convert all cell bboxes from crop-pixel space to HocrWord coordinate
    // space (PDF point units, image-oriented y).
    let mut converted_cells: Vec<Vec<(f32, f32, f32, f32)>> = Vec::with_capacity(num_rows);
    for row in cell_grid {
        let mut conv_row = Vec::with_capacity(num_cols);
        for cell in row {
            let cell_left = (cell.x1 + crop_offset_px_x) / sx;
            let cell_right = (cell.x2 + crop_offset_px_x) / sx;
            let cell_top = (cell.y1 + crop_offset_px_y) / sy;
            let cell_bottom = (cell.y2 + crop_offset_px_y) / sy;
            conv_row.push((cell_left, cell_top, cell_right, cell_bottom));
        }
        converted_cells.push(conv_row);
    }

    // Best-match assignment: assign each word to the single cell with the
    // highest IoW, preventing the same word from appearing in multiple cells.
    // Store (word_index, cx, cy) per cell for reading-order sorting.
    let mut cell_words: Vec<Vec<Vec<(usize, f32, f32)>>> = (0..num_rows)
        .map(|_| (0..num_cols).map(|_| Vec::new()).collect())
        .collect();

    for (wi, &word) in words.iter().enumerate() {
        let mut best_iow: f32 = 0.0;
        let mut best_row: usize = 0;
        let mut best_col: usize = 0;

        for (ri, conv_row) in converted_cells.iter().enumerate() {
            for (ci, &(cl, ct, cr, cb)) in conv_row.iter().enumerate() {
                let iow = word_hint_iow(word, cl, ct, cr, cb);
                if iow > best_iow {
                    best_iow = iow;
                    best_row = ri;
                    best_col = ci;
                }
            }
        }

        if best_iow >= 0.2 {
            let cx = word.left as f32 + word.width as f32 / 2.0;
            let cy = word.top as f32 + word.height as f32 / 2.0;
            cell_words[best_row][best_col].push((wi, cx, cy));
        }
    }

    // Build the text grid from the assigned words.
    let mut grid: Vec<Vec<String>> = Vec::with_capacity(num_rows);
    for row_cells in &cell_words {
        let mut grid_row = vec![String::new(); num_cols];
        for (ci, cell_word_indices) in row_cells.iter().enumerate() {
            if cell_word_indices.is_empty() {
                continue;
            }
            // Sort words within the cell by reading order (y then x).
            let mut sorted = cell_word_indices.clone();
            sorted.sort_by(|a, b| a.2.total_cmp(&b.2).then_with(|| a.1.total_cmp(&b.1)));
            let text: String = sorted
                .iter()
                .map(|(wi, _, _)| words[*wi].text.trim())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            grid_row[ci] = text;
        }
        grid.push(grid_row);
    }

    let markdown = render_grid_as_markdown(&grid);
    (grid, markdown)
}

// Word-to-cell matching is now handled inline in build_tatr_grid_table
// using best-match assignment (each word assigned to exactly one cell).

/// Detect and fix vertically-oriented table header text.
///
/// PDFs with rotated column headers (common in wide tables) produce garbled
/// text when the PDF extractor extracts characters individually: "y t i r o h t u A o N"
/// instead of "No Authority". Detected by: ≥3 tokens, >70% single characters.
/// Fixed by joining characters and reversing (the chars are in bottom-to-top order).
#[cfg(feature = "layout-detection")]
fn fix_vertical_header_text(text: &str) -> String {
    let trimmed = text.trim();
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.len() < 3 {
        return text.to_string();
    }
    let single_chars = tokens.iter().filter(|t| t.len() == 1).count();
    let ratio = single_chars as f32 / tokens.len() as f32;
    if ratio > 0.7 {
        // Join all tokens and reverse to get original reading order.
        let joined: String = tokens.concat();
        joined.chars().rev().collect()
    } else {
        text.to_string()
    }
}

/// Render a grid of cell text strings as a markdown table.
#[cfg(feature = "layout-detection")]
fn render_grid_as_markdown(grid: &[Vec<String>]) -> String {
    if grid.is_empty() {
        return String::new();
    }

    let max_cols = grid.iter().map(|r| r.len()).max().unwrap_or(0);
    if max_cols == 0 {
        return String::new();
    }

    let mut md = String::new();

    for (row_idx, row) in grid.iter().enumerate() {
        md.push('|');
        for col in 0..max_cols {
            let raw_cell = row.get(col).map(|s| s.as_str()).unwrap_or("");
            // Fix vertically-oriented header text (spaced single chars in reverse).
            let cell = fix_vertical_header_text(raw_cell);
            // Escape pipe characters first, then HTML entities
            let pipe_escaped = cell.replace('|', "\\|");
            let escaped = escape_html_entities(&pipe_escaped);
            md.push(' ');
            md.push_str(escaped.trim());
            md.push_str(" |");
        }
        md.push('\n');

        if row_idx == 0 {
            md.push('|');
            for _ in 0..max_cols {
                md.push_str(" --- |");
            }
            md.push('\n');
        }
    }

    if md.ends_with('\n') {
        md.pop();
    }
    md
}

// ---------------------------------------------------------------------------
// SLANeXT-based table recognition
// ---------------------------------------------------------------------------

/// Recognize tables on a native PDF page using SLANeXT structure prediction.
///
/// Unlike TATR (which works on cropped table regions), SLANeXT requires the
/// **full page image** to detect table structure. We run inference once per page,
/// then filter detected cells by RT-DETR table region bounding boxes.
///
/// Cell bboxes from SLANeXT are in full-page image coordinates. We match them
/// to RT-DETR table hint regions, then match words to cells within each table.
///
/// When `classifier` is provided, each table region is classified as wired or
/// wireless and the appropriate SLANeXT variant is used. The classifier runs on
/// the cropped table region (works on crops), then we run full-page inference
/// with the selected model.
#[cfg(feature = "layout-detection")]
#[allow(clippy::too_many_arguments)]
pub(in crate::pdf::structure) fn recognize_tables_slanet(
    page_image: &image::RgbImage,
    hints: &[LayoutHint],
    words: &[crate::pdf::table_reconstruct::HocrWord],
    page_result: &crate::pdf::structure::types::PageLayoutResult,
    page_height: f32,
    page_index: usize,
    slanet_model: &mut crate::layout::models::slanet::SlanetModel,
    classifier: Option<(
        &mut crate::layout::models::table_classifier::TableClassifier,
        &mut crate::layout::models::slanet::SlanetModel,
    )>,
) -> Vec<Table> {
    let rgb_image = page_image;
    let img_w = rgb_image.width();
    let img_h = rgb_image.height();

    let sx = img_w as f32 / page_result.page_width_pts;
    let sy = img_h as f32 / page_result.page_height_pts;

    let table_hints: Vec<&LayoutHint> = hints
        .iter()
        .filter(|h| h.class_name == LayoutHintClass::Table && h.confidence >= 0.5)
        .collect();

    if table_hints.is_empty() {
        return Vec::new();
    }

    // When a classifier is provided, classify the first table region on this page
    // to decide between wired and wireless SLANeXT variants.
    // `slanet_model` is the primary (wired or forced variant).
    // `classifier` provides (classifier, alternative_model) for auto-selection.
    let active_model: &mut crate::layout::models::slanet::SlanetModel = if let Some((cls, alt_model)) = classifier {
        // Crop the first table hint for classification
        let first_hint = table_hints[0];
        let px_left = (first_hint.left * sx).round().max(0.0) as u32;
        let px_top = ((page_height - first_hint.top) * sy).round().max(0.0) as u32;
        let px_right = (first_hint.right * sx).round().min(img_w as f32) as u32;
        let px_bottom = ((page_height - first_hint.bottom) * sy).round().min(img_h as f32) as u32;
        let crop_w = px_right.saturating_sub(px_left).max(10);
        let crop_h = px_bottom.saturating_sub(px_top).max(10);
        let crop = image::imageops::crop_imm(rgb_image, px_left, px_top, crop_w, crop_h).to_image();

        match cls.classify(&crop) {
            Ok(crate::layout::models::table_classifier::TableType::Wireless) => {
                tracing::debug!(
                    page = page_index,
                    "TableClassifier: page classified as wireless, using wireless SLANeXT"
                );
                alt_model // alt_model is wireless
            }
            Ok(crate::layout::models::table_classifier::TableType::Wired) => {
                tracing::debug!(
                    page = page_index,
                    "TableClassifier: page classified as wired, using wired SLANeXT"
                );
                slanet_model // slanet_model is wired
            }
            Err(e) => {
                tracing::warn!(page = page_index, "TableClassifier failed: {e}, defaulting to wired");
                slanet_model
            }
        }
    } else {
        slanet_model
    };

    tracing::trace!(
        page = page_index,
        page_image_w = img_w,
        page_image_h = img_h,
        table_hints = table_hints.len(),
        "SLANeXT: running full-page inference"
    );

    // Run SLANeXT on the FULL page image (not a crop).
    // SLANeXT expects complete table context to detect structure.
    let slanet_result = match active_model.recognize(rgb_image) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("SLANeXT inference failed on page {}: {e}", page_index);
            return Vec::new();
        }
    };

    if slanet_result.cells.is_empty() {
        tracing::debug!(
            page = page_index,
            tokens = slanet_result.structure_tokens.len(),
            confidence = format!("{:.3}", slanet_result.confidence),
            "SLANeXT: no cells detected on full page"
        );
        return Vec::new();
    }

    tracing::debug!(
        page = page_index,
        cells = slanet_result.cells.len(),
        rows = slanet_result.num_rows,
        cols = slanet_result.num_cols,
        confidence = format!("{:.3}", slanet_result.confidence),
        "SLANeXT: full-page inference result"
    );

    // For each RT-DETR table hint, find SLANeXT cells that overlap it,
    // then match words and build a markdown table.
    let mut tables = Vec::new();

    for hint in &table_hints {
        // Convert hint bbox to image coordinates (for cell matching)
        let hint_img_left = hint.left * sx;
        let hint_img_top = (page_height - hint.top) * sy;
        let hint_img_right = hint.right * sx;
        let hint_img_bottom = (page_height - hint.bottom) * sy;

        // Find SLANeXT cells whose center falls within this table region.
        // Cell bboxes are in original image pixel coords (from SLANeXT decode).
        let mut matching_cells: Vec<&crate::layout::models::slanet::SlanetCell> = Vec::new();
        for cell in &slanet_result.cells {
            let cx = (cell.bbox[0] + cell.bbox[2]) / 2.0;
            let cy = (cell.bbox[1] + cell.bbox[3]) / 2.0;
            if cx >= hint_img_left && cx <= hint_img_right && cy >= hint_img_top && cy <= hint_img_bottom {
                matching_cells.push(cell);
            }
        }

        if matching_cells.is_empty() {
            tracing::trace!(
                page = page_index,
                hint_left = format!("{:.0}", hint.left),
                hint_top = format!("{:.0}", hint.top),
                "SLANeXT: no cells overlap this table hint"
            );
            continue;
        }

        // Determine grid dimensions from matching cells
        let max_row = matching_cells.iter().map(|c| c.row).max().unwrap_or(0);
        let max_col = matching_cells.iter().map(|c| c.col).max().unwrap_or(0);
        let num_rows = max_row + 1;
        let num_cols = max_col + 1;

        tracing::trace!(
            page = page_index,
            matching_cells = matching_cells.len(),
            num_rows,
            num_cols,
            "SLANeXT: cells matched to table hint"
        );

        // Filter words overlapping the table hint bbox.
        // HocrWord uses image coordinates (y=0 at top), so flip the hint's PDF y-coords.
        let hint_img_top = (page_height - hint.top).max(0.0);
        let hint_img_bottom = (page_height - hint.bottom).max(0.0);

        let table_words: Vec<&crate::pdf::table_reconstruct::HocrWord> = words
            .iter()
            .filter(|w| {
                if w.text.trim().is_empty() {
                    return false;
                }
                word_hint_iow(w, hint.left, hint_img_top, hint.right, hint_img_bottom) >= 0.2
            })
            .collect();

        // Build markdown by matching words to SLANeXT cells.
        // Cell bboxes are in image pixel coords; words are in PDF coords.
        // Convert cell bboxes to PDF coord space for matching.
        let (grid, markdown) = build_slanet_cells_table(&matching_cells, num_rows, num_cols, &table_words, sx, sy);

        if markdown.is_empty() {
            tracing::debug!(page = page_index, "SLANeXT: empty markdown output for table hint");
            continue;
        }

        // Validate: reject if too few cells have content
        let total_cells = num_rows * num_cols;
        let filled_cells = grid
            .iter()
            .flat_map(|r| r.iter())
            .filter(|c| !c.trim().is_empty())
            .count();
        if total_cells > 4 && filled_cells < total_cells / 4 {
            tracing::debug!(
                page = page_index,
                total_cells,
                filled_cells,
                "SLANeXT table rejected: too few filled cells"
            );
            continue;
        }

        // Tighten the top edge of the bbox to the first row that has genuine
        // column structure (mirrors the same logic applied to the TATR path).
        let table_width = hint.right - hint.left;
        let col_gap_for_tighten = compute_col_gap_for_word_refs(&table_words, table_width);
        let slanet_num_cols = grid.first().map_or(0, |r| r.len());
        let min_column_gaps = (slanet_num_cols / 2).max(1);
        // hint_img_top is (page_height - hint.top).max(0.0) — unpadded for SLANeXT.
        let tightened_y1 = tighten_table_bbox_top(
            &table_words,
            hint_img_top,
            hint.top,
            col_gap_for_tighten,
            min_column_gaps,
            page_height,
        );

        let bounding_box = Some(crate::types::BoundingBox {
            x0: hint.left as f64,
            y0: hint.bottom as f64,
            x1: hint.right as f64,
            y1: tightened_y1,
        });

        tables.push(Table {
            cells: grid,
            markdown,
            page_number: (page_index + 1) as u32,
            bounding_box,
        });
    }

    tables
}

/// Build markdown table from SLANeXT cells matched to a single table region.
///
/// `cells` are already filtered to those overlapping the RT-DETR table hint.
/// Cell bboxes are in full-page image pixel coords; convert to PDF coords for
/// word matching.
#[cfg(feature = "layout-detection")]
fn build_slanet_cells_table(
    cells: &[&crate::layout::models::slanet::SlanetCell],
    num_rows: usize,
    num_cols: usize,
    words: &[&crate::pdf::table_reconstruct::HocrWord],
    sx: f32,
    sy: f32,
) -> (Vec<Vec<String>>, String) {
    if cells.is_empty() || num_rows == 0 || num_cols == 0 {
        return (Vec::new(), String::new());
    }

    // Renumber rows/cols to be 0-based relative to the filtered cell set.
    let min_row = cells.iter().map(|c| c.row).min().unwrap_or(0);
    let min_col = cells.iter().map(|c| c.col).min().unwrap_or(0);

    let grid_rows = num_rows.min(cells.iter().map(|c| c.row - min_row + 1).max().unwrap_or(1));
    let grid_cols = num_cols.min(cells.iter().map(|c| c.col - min_col + 1).max().unwrap_or(1));

    let mut grid: Vec<Vec<String>> = (0..grid_rows).map(|_| vec![String::new(); grid_cols]).collect();

    // Convert cell bboxes from image pixel coords to PDF/HocrWord coords.
    // Image pixel → PDF: x_pdf = x_px / sx, y_pdf = y_px / sy
    let converted_cells: Vec<(usize, usize, f32, f32, f32, f32)> = cells
        .iter()
        .map(|cell| {
            let cell_left = cell.bbox[0] / sx;
            let cell_top = cell.bbox[1] / sy;
            let cell_right = cell.bbox[2] / sx;
            let cell_bottom = cell.bbox[3] / sy;
            (
                cell.row - min_row,
                cell.col - min_col,
                cell_left,
                cell_top,
                cell_right,
                cell_bottom,
            )
        })
        .collect();

    // Best-match word-to-cell assignment
    let mut word_assignments: Vec<(usize, usize, f32, f32)> = Vec::new();

    for (wi, &word) in words.iter().enumerate() {
        let mut best_iow: f32 = 0.0;
        let mut best_cell_idx: usize = 0;

        for (ci, &(_row, _col, cl, ct, cr, cb)) in converted_cells.iter().enumerate() {
            let iow = word_hint_iow(word, cl, ct, cr, cb);
            if iow > best_iow {
                best_iow = iow;
                best_cell_idx = ci;
            }
        }

        if best_iow >= 0.2 {
            let cx = word.left as f32 + word.width as f32 / 2.0;
            let cy = word.top as f32 + word.height as f32 / 2.0;
            word_assignments.push((wi, best_cell_idx, cx, cy));
        }
    }

    // Group words by cell and sort by reading order
    let mut cell_word_groups: Vec<Vec<(usize, f32, f32)>> = vec![Vec::new(); cells.len()];
    for &(wi, cell_idx, cx, cy) in &word_assignments {
        if cell_idx < cell_word_groups.len() {
            cell_word_groups[cell_idx].push((wi, cx, cy));
        }
    }

    let assigned_count = cell_word_groups.iter().filter(|g| !g.is_empty()).count();
    tracing::trace!(
        total_words = words.len(),
        assigned_words = word_assignments.len(),
        cells_with_words = assigned_count,
        total_cells = cells.len(),
        "SLANeXT: word-to-cell assignment complete"
    );

    for (ci, group) in cell_word_groups.iter_mut().enumerate() {
        group.sort_by(|a, b| a.2.total_cmp(&b.2).then_with(|| a.1.total_cmp(&b.1)));
        let text: String = group
            .iter()
            .map(|(wi, _, _)| words[*wi].text.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let (row, col) = (converted_cells[ci].0, converted_cells[ci].1);
        if row < grid_rows && col < grid_cols {
            grid[row][col] = text;
        }
    }

    let markdown = render_grid_as_markdown(&grid);
    (grid, markdown)
}

/// Compute the adaptive column-gap threshold for a slice of `&HocrWord` references.
///
/// Mirrors the logic in `tables::compute_adaptive_column_gap` for borrowed slices.
#[cfg(feature = "layout-detection")]
fn compute_col_gap_for_word_refs(
    words: &[&crate::pdf::table_reconstruct::HocrWord],
    table_width: f32,
) -> u32 {
    let mut gaps: Vec<u32> = Vec::new();

    if words.len() >= 4 {
        let mut heights: Vec<u32> = words.iter().map(|w| w.height).collect();
        heights.sort_unstable();
        let median_h = heights[heights.len() / 2];
        let row_tolerance = (median_h / 2).max(3);

        let mut sorted: Vec<(u32, u32, u32)> = words
            .iter()
            .map(|w| {
                let yc = w.top + w.height / 2;
                (yc, w.left, w.left + w.width)
            })
            .collect();
        sorted.sort_by_key(|&(yc, x, _)| (yc, x));

        let mut row_start = 0;
        while row_start < sorted.len() {
            let row_yc = sorted[row_start].0;
            let mut row_end = row_start + 1;
            while row_end < sorted.len()
                && sorted[row_end].0.abs_diff(row_yc) <= row_tolerance
            {
                row_end += 1;
            }
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
        let large_gaps: Vec<u32> = gaps.iter().copied().filter(|&g| g >= 40).collect();
        if !large_gaps.is_empty() {
            let median_gap = large_gaps[large_gaps.len() / 2];
            return (median_gap / 2).clamp(20, 60);
        } else {
            let median_gap = gaps[gaps.len() / 2];
            return (median_gap * 3).clamp(20, 60);
        }
    }

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

/// Tighten the table bounding-box top edge to the first row with genuine column structure.
///
/// The layout model hint bbox often extends above the actual table grid to cover
/// an adjacent header/metadata block (e.g. "Precinct RUN 12/3/2014" on election
/// pages).  Using raw `hint.top` as `bbox.y1` causes
/// `filter_segments_by_table_bboxes` to suppress those header paragraphs, making
/// them invisible in the extraction output.
///
/// Strategy: walk word rows in image-y order (ascending = top-of-page first).
/// The first row whose words span at least `min_column_gaps` gaps ≥ `col_gap` is
/// the first genuine table content row.  Setting `min_column_gaps` to
/// `(num_table_cols / 2).max(1)` lets header rows with 1–2 text blocks pass
/// through while still accepting sparse table rows.
///
/// Returns the tightened PDF y coordinate (≤ `hint_top_pdf`).
#[cfg(feature = "layout-detection")]
fn tighten_table_bbox_top(
    table_words: &[&crate::pdf::table_reconstruct::HocrWord],
    unpadded_hint_img_top: f32,
    hint_top_pdf: f32,
    col_gap: u32,
    min_column_gaps: usize,
    page_height: f32,
) -> f64 {
    /// Small upward margin (image pts) added to the first-row top so that the
    /// row's own top edge is fully inside the bbox.  Must match the constant
    /// `TABLE_BBOX_TOP_TIGHTEN_MARGIN_PTS` in `tables.rs`.
    const TABLE_BBOX_TOP_TIGHTEN_MARGIN_PTS: u32 = 4;
    const SAME_ROW_TOLERANCE_PTS: u32 = 5;

    let mut sorted: Vec<&crate::pdf::table_reconstruct::HocrWord> =
        table_words.iter().copied().collect();
    sorted.sort_by_key(|w| w.top);

    let mut first_table_row_top: Option<u32> = None;
    let mut row_start = 0_usize;
    while row_start < sorted.len() {
        let row_anchor = sorted[row_start].top;
        let row_end = sorted[row_start..]
            .iter()
            .position(|w| w.top.saturating_sub(row_anchor) > SAME_ROW_TOLERANCE_PTS)
            .map(|p| row_start + p)
            .unwrap_or(sorted.len());

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

    let img_top = first_table_row_top.unwrap_or(unpadded_hint_img_top as u32);
    let img_top_with_margin = img_top.saturating_sub(TABLE_BBOX_TOP_TIGHTEN_MARGIN_PTS);
    let pdf_top = page_height - img_top_with_margin as f32;
    // Never extend the bbox beyond the original hint top.
    (pdf_top as f64).min(hint_top_pdf as f64)
}

#[cfg(test)]
#[cfg(feature = "layout-detection")]
mod tests {
    use super::{compute_col_gap_for_word_refs, tighten_table_bbox_top};
    use crate::pdf::table_reconstruct::HocrWord;

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

    /// Verifies that a two-text-block header row (1 column gap) is skipped when
    /// `min_column_gaps = 2` (4-column table), and the first genuine table row
    /// (3 column gaps) is found instead.
    ///
    /// Models the la-precinct-bulletin-2014-p1 regression: the ballot header had
    /// two text blocks at widely-separated x positions, giving it a 181 pt gap
    /// that exceeded the col_gap threshold — making it look like a table row
    /// under the old `min_column_gaps = 1` logic.
    #[test]
    fn test_tighten_skips_two_block_header_finds_four_column_table_row() {
        let page_height = 612.0_f32;

        // Header row at image-y = 16 (two text blocks, 181 pt gap between them)
        let header_precinct = make_word("Precinct", 34, 16, 47, 10); // right=81
        let header_registrar = make_word("REGISTRAR", 262, 16, 90, 10); // left=262, gap=181

        // Table first row at image-y = 86 (4 columns → 3 gaps)
        let col1 = make_word("GOVERNOR", 33, 86, 47, 10); // right=80
        let col2 = make_word("COLUMN2", 217, 86, 70, 10); // left=217 right=287 gap=137
        let col3 = make_word("COLUMN3", 400, 86, 70, 10); // left=400 right=470 gap=113
        let col4 = make_word("COLUMN4", 580, 86, 70, 10); // left=580 gap=110

        let all_words: Vec<&HocrWord> =
            vec![&header_precinct, &header_registrar, &col1, &col2, &col3, &col4];

        // col_gap = 30 (any gap > 30 counts); min_column_gaps = 2 (4-col table)
        // hint top in PDF coords = page_height - 16 = 596.0
        let hint_img_top = (page_height - 596.0_f32).max(0.0); // = 16.0
        let result = tighten_table_bbox_top(
            &all_words,
            hint_img_top,
            596.0,
            30,
            2, // min_column_gaps for a 4-column table
            page_height,
        );

        // Expected: first table row at img-y=86, margin=4 → img_top_margin=82
        // pdf_top = 612 - 82 = 530.0; tightened_y1 = min(530.0, 596.0) = 530.0
        assert!(
            (result - 530.0).abs() < 1.0,
            "expected tightened_y1 ≈ 530.0, got {result}"
        );
    }

    /// When `min_column_gaps = 1` (2-column table), a two-block header is
    /// accepted as the first table row — tightening stops there, which is the
    /// correct behaviour for tables that look exactly like a 2-block row.
    #[test]
    fn test_tighten_two_column_table_accepts_first_gap_row() {
        let page_height = 612.0_f32;

        // "Header" row with 2 text blocks at image-y = 16
        let block_a = make_word("LEFT", 10, 16, 60, 10); // right=70
        let block_b = make_word("RIGHT", 200, 16, 60, 10); // gap=130

        let all_words: Vec<&HocrWord> = vec![&block_a, &block_b];

        let hint_img_top = (page_height - 596.0_f32).max(0.0); // = 16.0
        let result = tighten_table_bbox_top(
            &all_words,
            hint_img_top,
            596.0,
            30,
            1, // min_column_gaps = 1 → first gap row accepted
            page_height,
        );

        // First row (img-y=16) qualifies → img_top_margin = 16-4 = 12
        // pdf_top = 612-12 = 600.0; clamped to min(600.0, 596.0) = 596.0
        assert!(
            (result - 596.0).abs() < 1.0,
            "expected tightened_y1 ≈ 596.0 (no tightening past hint), got {result}"
        );
    }

    /// When no words meet the min-column-gaps threshold, the function falls back
    /// to `unpadded_hint_img_top` — bbox top stays at the original hint top.
    #[test]
    fn test_tighten_no_qualifying_row_falls_back_to_hint_top() {
        let page_height = 612.0_f32;

        // All words in a single narrow group (no column gaps)
        let w1 = make_word("word1", 10, 20, 40, 10);
        let w2 = make_word("word2", 55, 20, 40, 10); // gap = 5, < col_gap=30

        let all_words: Vec<&HocrWord> = vec![&w1, &w2];
        let hint_img_top = (page_height - 592.0_f32).max(0.0); // =20.0
        let result = tighten_table_bbox_top(&all_words, hint_img_top, 592.0, 30, 2, page_height);

        // No row qualifies → fallback: img_top=20, margin=4, img_top_margin=16
        // pdf_top = 612-16 = 596.0; clamped to min(596.0, 592.0) = 592.0
        assert!(
            (result - 592.0).abs() < 1.0,
            "expected fallback to hint_top_pdf=592.0, got {result}"
        );
    }

    #[test]
    fn test_compute_col_gap_for_word_refs_returns_sensible_gap() {
        let page_height = 800.0_f32;
        // 4 words in 2 columns on 1 row, large inter-column gap ≈ 200pt
        let w1 = make_word("A", 10, 10, 40, 10);
        let w2 = make_word("B", 60, 10, 40, 10); // small intra-col gap = 10
        let w3 = make_word("C", 300, 10, 40, 10); // inter-col gap = 200
        let w4 = make_word("D", 350, 10, 40, 10); // small intra-col gap = 10
        let _ = page_height;

        let words: Vec<&HocrWord> = vec![&w1, &w2, &w3, &w4];
        let col_gap = compute_col_gap_for_word_refs(&words, 400.0);
        // Large gaps are ≥40; here 200 > 40. median_gap=200, threshold=100 → clamped to 60.
        assert_eq!(col_gap, 60, "expected col_gap=60 (large-gap median/2 clamped), got {col_gap}");
    }
}
