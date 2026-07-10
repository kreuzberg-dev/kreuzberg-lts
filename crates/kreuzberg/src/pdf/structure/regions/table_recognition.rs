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
    page_image: &image::DynamicImage,
    hints: &[LayoutHint],
    words: &[crate::pdf::table_reconstruct::HocrWord],
    page_result: &crate::pdf::layout_runner::PageLayoutResult,
    page_height: f32,
    page_index: usize,
    tatr_model: &mut crate::layout::models::tatr::TatrModel,
) -> Vec<Table> {
    let rgb_image = page_image.to_rgb8();
    let img_w = rgb_image.width();
    let img_h = rgb_image.height();

    let sx = img_w as f32 / page_result.page_width_pts;
    let sy = img_h as f32 / page_result.page_height_pts;

    let table_hints: Vec<&LayoutHint> = hints
        .iter()
        .filter(|h| {
            if h.class != LayoutHintClass::Table || h.confidence < 0.5 {
                return false;
            }
            true
        })
        .collect();

    let mut tables = Vec::new();

    for hint in &table_hints {
        let px_left = (hint.left * sx).round().max(0.0) as u32;
        let px_top = ((page_height - hint.top) * sy).round().max(0.0) as u32;
        let px_right = (hint.right * sx).round().min(img_w as f32) as u32;
        let px_bottom = ((page_height - hint.bottom) * sy).round().min(img_h as f32) as u32;

        let crop_w = px_right.saturating_sub(px_left);
        let crop_h = px_bottom.saturating_sub(px_top);

        if crop_w < 10 || crop_h < 10 {
            continue;
        }

        if (crop_w as u64) * (crop_h as u64) > 4_000_000 {
            tracing::debug!(
                page = page_index,
                crop_w,
                crop_h,
                "Skipping TATR for oversized table crop"
            );
            continue;
        }

        let cropped = image::imageops::crop_imm(&rgb_image, px_left, px_top, crop_w, crop_h).to_image();

        let tatr_result = match tatr_model.recognize(&cropped) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("TATR inference failed for table on page {}: {e}", page_index);
                continue;
            }
        };

        if tatr_result.rows.is_empty() || tatr_result.columns.is_empty() {
            tracing::debug!(
                page = page_index,
                rows = tatr_result.rows.len(),
                columns = tatr_result.columns.len(),
                "TATR: no rows or columns detected"
            );
            continue;
        }

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

        let bounding_box = Some(crate::types::BoundingBox {
            x0: hint.left as f64,
            y0: hint.bottom as f64,
            x1: hint.right as f64,
            y1: hint.top as f64,
        });

        tables.push(Table {
            cells: grid,
            markdown,
            page_number: page_index + 1,
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

    let mut grid: Vec<Vec<String>> = Vec::with_capacity(num_rows);
    for row_cells in &cell_words {
        let mut grid_row = vec![String::new(); num_cols];
        for (ci, cell_word_indices) in row_cells.iter().enumerate() {
            if cell_word_indices.is_empty() {
                continue;
            }
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

/// Detect and fix vertically-oriented table header text.
///
/// PDFs with rotated column headers (common in wide tables) produce garbled
/// text when pdfium extracts characters individually: "y t i r o h t u A o N"
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
            let cell = fix_vertical_header_text(raw_cell);
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
    page_image: &image::DynamicImage,
    hints: &[LayoutHint],
    words: &[crate::pdf::table_reconstruct::HocrWord],
    page_result: &crate::pdf::layout_runner::PageLayoutResult,
    page_height: f32,
    page_index: usize,
    slanet_model: &mut crate::layout::models::slanet::SlanetModel,
    classifier: Option<(
        &mut crate::layout::models::table_classifier::TableClassifier,
        &mut crate::layout::models::slanet::SlanetModel,
    )>,
) -> Vec<Table> {
    let rgb_image = page_image.to_rgb8();
    let img_w = rgb_image.width();
    let img_h = rgb_image.height();

    let sx = img_w as f32 / page_result.page_width_pts;
    let sy = img_h as f32 / page_result.page_height_pts;

    let table_hints: Vec<&LayoutHint> = hints
        .iter()
        .filter(|h| h.class == LayoutHintClass::Table && h.confidence >= 0.5)
        .collect();

    if table_hints.is_empty() {
        return Vec::new();
    }

    let active_model: &mut crate::layout::models::slanet::SlanetModel = if let Some((cls, alt_model)) = classifier {
        let first_hint = table_hints[0];
        let px_left = (first_hint.left * sx).round().max(0.0) as u32;
        let px_top = ((page_height - first_hint.top) * sy).round().max(0.0) as u32;
        let px_right = (first_hint.right * sx).round().min(img_w as f32) as u32;
        let px_bottom = ((page_height - first_hint.bottom) * sy).round().min(img_h as f32) as u32;
        let crop_w = px_right.saturating_sub(px_left).max(10);
        let crop_h = px_bottom.saturating_sub(px_top).max(10);
        let crop = image::imageops::crop_imm(&rgb_image, px_left, px_top, crop_w, crop_h).to_image();

        match cls.classify(&crop) {
            Ok(crate::layout::models::table_classifier::TableType::Wireless) => {
                tracing::debug!(
                    page = page_index,
                    "TableClassifier: page classified as wireless, using wireless SLANeXT"
                );
                alt_model
            }
            Ok(crate::layout::models::table_classifier::TableType::Wired) => {
                tracing::debug!(
                    page = page_index,
                    "TableClassifier: page classified as wired, using wired SLANeXT"
                );
                slanet_model
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

    let slanet_result = match active_model.recognize(&rgb_image) {
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

    let mut tables = Vec::new();

    for hint in &table_hints {
        let hint_img_left = hint.left * sx;
        let hint_img_top = (page_height - hint.top) * sy;
        let hint_img_right = hint.right * sx;
        let hint_img_bottom = (page_height - hint.bottom) * sy;

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

        let (grid, markdown) = build_slanet_cells_table(&matching_cells, num_rows, num_cols, &table_words, sx, sy);

        if markdown.is_empty() {
            tracing::debug!(page = page_index, "SLANeXT: empty markdown output for table hint");
            continue;
        }

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

        let bounding_box = Some(crate::types::BoundingBox {
            x0: hint.left as f64,
            y0: hint.bottom as f64,
            x1: hint.right as f64,
            y1: hint.top as f64,
        });

        tables.push(Table {
            cells: grid,
            markdown,
            page_number: page_index + 1,
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

    let min_row = cells.iter().map(|c| c.row).min().unwrap_or(0);
    let min_col = cells.iter().map(|c| c.col).min().unwrap_or(0);

    let grid_rows = num_rows.min(cells.iter().map(|c| c.row - min_row + 1).max().unwrap_or(1));
    let grid_cols = num_cols.min(cells.iter().map(|c| c.col - min_col + 1).max().unwrap_or(1));

    let mut grid: Vec<Vec<String>> = (0..grid_rows).map(|_| vec![String::new(); grid_cols]).collect();

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
