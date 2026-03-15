//! SLANet-plus table structure recognition model.
//!
//! Takes a cropped table image and outputs HTML structure tokens with cell
//! bounding boxes for markdown table reconstruction.
//!
//! Model: SLANet-plus (~6.9MB ONNX, TEDS 0.845)
//! Input: `[1, 3, 488, 488]` f32, BGR channel order, ImageNet normalization
//! Output 0: `loc_preds`      `[1, seq_len, 8]`  — cell bbox corners (sigmoid → [0,1])
//! Output 1: `structure_probs` `[1, seq_len, 50]` — HTML token softmax probabilities

use image::RgbImage;
use ndarray::Array4;
use ort::{inputs, session::Session, value::Tensor};

use crate::layout::error::LayoutError;
use crate::layout::session::build_session;
use crate::layout::types::BBox;

/// SLANet-plus input resolution.
const INPUT_SIZE: u32 = 488;

/// ImageNet normalization in BGR channel order (as SLANet expects).
const IMAGENET_MEAN_BGR: [f32; 3] = [0.406, 0.456, 0.485];
const IMAGENET_STD_BGR: [f32; 3] = [0.225, 0.224, 0.229];

/// Standard PaddlePaddle SLANet vocabulary (50 tokens).
///
/// Index 0 = `sos`, Index 49 = `eos`, Indices 1–48 = structure tokens.
/// Source: PaddleOCR `ppocr/utils/dict/table_structure_dict.txt` + sos/eos.
const VOCABULARY: &[&str] = &[
    // 0: start of sequence
    "sos",
    // 1–9: structural tags
    "<td></td>",
    "<td>",
    "</td>",
    "<tr>",
    "</tr>",
    "<thead>",
    "</thead>",
    "<tbody>",
    "</tbody>",
    // 10: attribute opener
    "<td",
    // 11–25: rowspan attributes
    " rowspan=\"2\"",
    " rowspan=\"3\"",
    " rowspan=\"4\"",
    " rowspan=\"5\"",
    " rowspan=\"6\"",
    " rowspan=\"7\"",
    " rowspan=\"8\"",
    " rowspan=\"9\"",
    " rowspan=\"10\"",
    " rowspan=\"11\"",
    " rowspan=\"12\"",
    " rowspan=\"13\"",
    " rowspan=\"14\"",
    " rowspan=\"15\"",
    " rowspan=\"16\"",
    // 26–49: colspan attributes + closer
    " colspan=\"2\"",
    " colspan=\"3\"",
    " colspan=\"4\"",
    " colspan=\"5\"",
    " colspan=\"6\"",
    " colspan=\"7\"",
    " colspan=\"8\"",
    " colspan=\"9\"",
    " colspan=\"10\"",
    " colspan=\"11\"",
    " colspan=\"12\"",
    " colspan=\"13\"",
    " colspan=\"14\"",
    " colspan=\"15\"",
    " colspan=\"16\"",
    " colspan=\"17\"",
    " colspan=\"18\"",
    " colspan=\"19\"",
    " colspan=\"20\"",
    " colspan=\"21\"",
    " colspan=\"22\"",
    " colspan=\"23\"",
    // 48: attribute closer
    ">",
    // 49: end of sequence
    "eos",
];

/// Tokens that indicate a cell start (bbox is extracted at these positions).
const CELL_TOKENS: &[&str] = &["<td>", "<td", "<td></td>"];

/// SLANet-plus table structure recognition result.
#[derive(Debug, Clone)]
pub struct SlaNetResult {
    /// HTML structure tokens (e.g., `["<tr>", "<td>", "</td>", "</tr>"]`).
    pub structure_tokens: Vec<String>,
    /// Cell bounding boxes in input image pixel coordinates.
    /// One bbox per cell-start token (`<td>`, `<td`, `<td></td>`).
    pub cell_bboxes: Vec<BBox>,
}

/// SLANet-plus table structure recognition model.
pub struct SlaNetModel {
    session: Session,
    input_name: String,
    vocabulary: Vec<String>,
    eos_index: usize,
}

impl SlaNetModel {
    /// Load a SLANet-plus ONNX model from a file.
    pub fn from_file(path: &str) -> Result<Self, LayoutError> {
        // Try standard session first; fall back to CPU-only if platform EP fails
        // (SLANet uses ops that CoreML sometimes can't handle).
        let session = match build_session(path, None) {
            Ok(s) => s,
            Err(first_err) => {
                tracing::warn!("SLANet: platform EP failed ({first_err}), retrying with CPU-only");
                match Self::build_cpu_session(path) {
                    Ok(s) => s,
                    Err(cpu_err) => {
                        tracing::warn!("SLANet: CPU-only also failed: {cpu_err}");
                        return Err(cpu_err);
                    }
                }
            }
        };
        let input_name = session.inputs()[0].name().to_string();

        // Try reading vocabulary from ONNX metadata, fall back to hardcoded.
        let vocabulary = read_vocabulary_from_metadata(&session)
            .unwrap_or_else(|| VOCABULARY.iter().map(|s| (*s).to_string()).collect());

        let eos_index = vocabulary
            .iter()
            .position(|t| t == "eos")
            .unwrap_or(vocabulary.len() - 1);

        Ok(Self {
            session,
            input_name,
            vocabulary,
            eos_index,
        })
    }

    /// Build a CPU-only ORT session (no CoreML/CUDA).
    fn build_cpu_session(path: &str) -> Result<ort::session::Session, LayoutError> {
        use ort::session::builder::GraphOptimizationLevel;
        let num_cores = num_cpus::get();
        let builder = ort::session::Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(num_cores)?
            .with_inter_threads(1)?;
        Ok(builder.commit_from_file(path)?)
    }

    /// Recognize table structure from a cropped table image.
    ///
    /// Returns HTML structure tokens and cell bounding boxes in the
    /// input image's pixel coordinate space.
    pub fn recognize(&mut self, table_img: &RgbImage) -> Result<SlaNetResult, LayoutError> {
        let orig_w = table_img.width();
        let orig_h = table_img.height();

        // Preprocess: aspect-preserving resize to 488×488, BGR, ImageNet normalize, zero-pad.
        let (input_tensor, scale, pad_x, pad_y) = preprocess_slanet(table_img);
        let tensor = Tensor::from_array(input_tensor)?;

        let outputs = self.session.run(inputs![
            self.input_name.as_str() => tensor
        ])?;

        // Parse outputs: loc_preds [1, seq_len, 8] and structure_probs [1, seq_len, vocab_size].
        let mut float_outputs: Vec<(Vec<usize>, Vec<f32>)> = Vec::new();

        for (_name, value) in outputs.iter() {
            if let Ok(view) = value.try_extract_tensor::<f32>() {
                let shape: Vec<usize> = view.0.iter().map(|&d| d as usize).collect();
                let data: Vec<f32> = view.1.to_vec();
                float_outputs.push((shape, data));
            }
        }

        if float_outputs.len() < 2 {
            return Err(LayoutError::InvalidOutput(format!(
                "SLANet expected 2 float outputs, got {}",
                float_outputs.len()
            )));
        }

        // Identify which output is loc_preds (last dim == 8) vs structure_probs (last dim == vocab_size).
        let (loc_shape, loc_data, struct_shape, struct_data) = if float_outputs[0].0.last() == Some(&8) {
            let (ls, ld) = float_outputs.remove(0);
            let (ss, sd) = float_outputs.remove(0);
            (ls, ld, ss, sd)
        } else {
            let (ss, sd) = float_outputs.remove(0);
            let (ls, ld) = float_outputs.remove(0);
            (ls, ld, ss, sd)
        };

        let seq_len = struct_shape.get(1).copied().unwrap_or(0);
        let vocab_size = struct_shape.last().copied().unwrap_or(0);

        if seq_len == 0 || vocab_size == 0 {
            return Ok(SlaNetResult {
                structure_tokens: Vec::new(),
                cell_bboxes: Vec::new(),
            });
        }

        // Decode: argmax over vocab at each position, stop at eos.
        let mut structure_tokens = Vec::new();
        let mut cell_bboxes = Vec::new();

        for pos in 0..seq_len {
            // Argmax over vocabulary dimension
            let offset = pos * vocab_size;
            let probs = &struct_data[offset..offset + vocab_size];
            let token_idx = probs
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| i)
                .unwrap_or(self.eos_index);

            // Stop at eos
            if token_idx == self.eos_index {
                break;
            }
            // Skip sos
            if token_idx == 0 {
                continue;
            }

            let token = self.vocabulary.get(token_idx).cloned().unwrap_or_default();

            // Extract cell bbox for cell-start tokens
            if CELL_TOKENS.contains(&token.as_str()) {
                let bbox_offset = pos * 8;
                if bbox_offset + 8 <= loc_data.len() {
                    let raw = &loc_data[bbox_offset..bbox_offset + 8];
                    let bbox = decode_cell_bbox(raw, scale, pad_x, pad_y, orig_w, orig_h, &loc_shape);
                    cell_bboxes.push(bbox);
                }
            }

            structure_tokens.push(token);
        }

        Ok(SlaNetResult {
            structure_tokens,
            cell_bboxes,
        })
    }
}

/// Preprocess an image for SLANet-plus inference.
///
/// Pipeline: aspect-preserving resize → zero-pad to 488×488 → BGR → ImageNet normalize → NCHW.
///
/// Returns `(tensor, scale, pad_x, pad_y)` for coordinate denormalization.
fn preprocess_slanet(img: &RgbImage) -> (Array4<f32>, f32, u32, u32) {
    let (orig_w, orig_h) = (img.width() as f32, img.height() as f32);
    let scale = (INPUT_SIZE as f32 / orig_w).min(INPUT_SIZE as f32 / orig_h);
    let new_w = (orig_w * scale).round() as u32;
    let new_h = (orig_h * scale).round() as u32;

    let resized = image::imageops::resize(img, new_w, new_h, image::imageops::FilterType::CatmullRom);

    // Top-left placement (no centering — SLANet uses top-left padding)
    let pad_x = 0u32;
    let pad_y = 0u32;

    let ts = INPUT_SIZE as usize;
    let hw = ts * ts;

    let inv_std_b = 1.0 / IMAGENET_STD_BGR[0];
    let inv_std_g = 1.0 / IMAGENET_STD_BGR[1];
    let inv_std_r = 1.0 / IMAGENET_STD_BGR[2];

    // Zero-pad: normalized zero → (0.0 - mean) / std
    let pad_b = (0.0 - IMAGENET_MEAN_BGR[0]) * inv_std_b;
    let pad_g = (0.0 - IMAGENET_MEAN_BGR[1]) * inv_std_g;
    let pad_r = (0.0 - IMAGENET_MEAN_BGR[2]) * inv_std_r;

    let mut data = vec![0.0f32; 3 * hw];
    // Fill with padding values (channel order: BGR)
    for i in 0..hw {
        data[i] = pad_b;
        data[hw + i] = pad_g;
        data[2 * hw + i] = pad_r;
    }

    // Copy resized pixels (RGB input → BGR output)
    let rw = new_w as usize;
    let rh = new_h as usize;
    let pixels = resized.as_raw();

    for y in 0..rh {
        for x in 0..rw {
            let src_idx = (y * rw + x) * 3;
            let dst_idx = y * ts + x;
            let r = pixels[src_idx] as f32 * (1.0 / 255.0);
            let g = pixels[src_idx + 1] as f32 * (1.0 / 255.0);
            let b = pixels[src_idx + 2] as f32 * (1.0 / 255.0);
            // BGR channel order
            data[dst_idx] = (b - IMAGENET_MEAN_BGR[0]) * inv_std_b;
            data[hw + dst_idx] = (g - IMAGENET_MEAN_BGR[1]) * inv_std_g;
            data[2 * hw + dst_idx] = (r - IMAGENET_MEAN_BGR[2]) * inv_std_r;
        }
    }

    let tensor = Array4::from_shape_vec((1, 3, ts, ts), data).expect("shape mismatch in preprocess_slanet");

    (tensor, scale, pad_x, pad_y)
}

/// Decode a raw cell bounding box from SLANet output to input image pixel coordinates.
///
/// Raw values are sigmoid-normalized [0,1] relative to the 488×488 input.
/// We denormalize to the original image coordinate space.
fn decode_cell_bbox(
    raw: &[f32],
    scale: f32,
    pad_x: u32,
    pad_y: u32,
    orig_w: u32,
    orig_h: u32,
    loc_shape: &[usize],
) -> BBox {
    // Apply sigmoid to get [0,1] normalized coordinates
    let sigmoid = |x: f32| 1.0 / (1.0 + (-x).exp());

    // Raw format: [x1,y1, x2,y2, x3,y3, x4,y4] — 4 corners
    let coords: Vec<f32> = raw.iter().map(|&v| sigmoid(v)).collect();

    // The bbox coordinates may be relative to the model input size
    // or already in [0,1] normalized space. Check shape to determine.
    let _loc_last = loc_shape.last().copied().unwrap_or(8);

    // Scale from [0,1] → 488×488 pixel space, then un-pad and un-scale to original image
    let inv_scale = 1.0 / scale;
    let pad_x_f = pad_x as f32;
    let pad_y_f = pad_y as f32;
    let ow = orig_w as f32;
    let oh = orig_h as f32;

    // Extract all x and y coordinates, denormalize
    let mut xs = Vec::with_capacity(4);
    let mut ys = Vec::with_capacity(4);
    for i in 0..4 {
        let x_norm = coords[i * 2] * INPUT_SIZE as f32;
        let y_norm = coords[i * 2 + 1] * INPUT_SIZE as f32;
        let x = ((x_norm - pad_x_f) * inv_scale).clamp(0.0, ow);
        let y = ((y_norm - pad_y_f) * inv_scale).clamp(0.0, oh);
        xs.push(x);
        ys.push(y);
    }

    // Convert 4-corner polygon to axis-aligned bounding box
    let x1 = xs.iter().copied().fold(f32::INFINITY, f32::min);
    let y1 = ys.iter().copied().fold(f32::INFINITY, f32::min);
    let x2 = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let y2 = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    BBox::new(x1, y1, x2, y2)
}

/// Try to read the vocabulary from ONNX model metadata.
///
/// PaddlePaddle models store the character dictionary in a custom metadata key
/// named `"character"` as a JSON-encoded array or newline-separated string.
fn read_vocabulary_from_metadata(session: &Session) -> Option<Vec<String>> {
    let metadata = session.metadata().ok()?;
    let raw = metadata.custom("character")?;

    // Try JSON array first: ["sos", "<td></td>", ...]
    if raw.starts_with('[')
        && let Ok(vocab) = serde_json::from_str::<Vec<String>>(&raw)
        && vocab.len() >= 3
    {
        return Some(vocab);
    }

    // Fall back to newline-separated
    let tokens: Vec<String> = raw.lines().map(str::to_string).collect();
    if tokens.len() >= 3 { Some(tokens) } else { None }
}

/// Parse the SLANet HTML structure tokens into a grid of rows and cells.
///
/// Returns a list of rows, where each row contains cells with their
/// colspan/rowspan attributes and the bbox index (if available).
pub fn parse_table_structure(result: &SlaNetResult) -> Vec<Vec<TableCell>> {
    let mut rows: Vec<Vec<TableCell>> = Vec::new();
    let mut current_row: Vec<TableCell> = Vec::new();
    let mut in_row = false;
    let mut cell_bbox_idx = 0usize;
    let mut current_cell: Option<TableCellBuilder> = None;

    for token in &result.structure_tokens {
        match token.as_str() {
            "<tr>" => {
                in_row = true;
                current_row = Vec::new();
            }
            "</tr>" => {
                if let Some(builder) = current_cell.take() {
                    current_row.push(builder.build());
                }
                if in_row {
                    rows.push(std::mem::take(&mut current_row));
                }
                in_row = false;
            }
            "<td></td>" => {
                // Empty cell with bbox
                if let Some(builder) = current_cell.take() {
                    current_row.push(builder.build());
                }
                let bbox = result.cell_bboxes.get(cell_bbox_idx).cloned();
                cell_bbox_idx += 1;
                current_row.push(TableCell {
                    colspan: 1,
                    rowspan: 1,
                    bbox,
                });
            }
            "<td>" => {
                // Simple cell start with bbox
                if let Some(builder) = current_cell.take() {
                    current_row.push(builder.build());
                }
                let bbox = result.cell_bboxes.get(cell_bbox_idx).cloned();
                cell_bbox_idx += 1;
                current_cell = Some(TableCellBuilder {
                    colspan: 1,
                    rowspan: 1,
                    bbox,
                });
            }
            "<td" => {
                // Cell with attributes — bbox extracted, attributes follow
                if let Some(builder) = current_cell.take() {
                    current_row.push(builder.build());
                }
                let bbox = result.cell_bboxes.get(cell_bbox_idx).cloned();
                cell_bbox_idx += 1;
                current_cell = Some(TableCellBuilder {
                    colspan: 1,
                    rowspan: 1,
                    bbox,
                });
            }
            ">" => {
                // Attribute closer — cell definition complete, content follows
            }
            "</td>" => {
                if let Some(builder) = current_cell.take() {
                    current_row.push(builder.build());
                }
            }
            other => {
                // Attribute tokens: rowspan="N" or colspan="N"
                if let Some(ref mut builder) = current_cell {
                    if let Some(n) = parse_span_attribute(other, "rowspan") {
                        builder.rowspan = n;
                    } else if let Some(n) = parse_span_attribute(other, "colspan") {
                        builder.colspan = n;
                    }
                }
            }
        }
    }

    // Flush any remaining cell/row
    if let Some(builder) = current_cell.take() {
        current_row.push(builder.build());
    }
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    rows
}

/// A cell in the recognized table structure.
#[derive(Debug, Clone)]
pub struct TableCell {
    pub colspan: u32,
    pub rowspan: u32,
    /// Bounding box in the original table image's pixel coordinates.
    pub bbox: Option<BBox>,
}

struct TableCellBuilder {
    colspan: u32,
    rowspan: u32,
    bbox: Option<BBox>,
}

impl TableCellBuilder {
    fn build(self) -> TableCell {
        TableCell {
            colspan: self.colspan,
            rowspan: self.rowspan,
            bbox: self.bbox,
        }
    }
}

/// Parse ` rowspan="N"` or ` colspan="N"` attribute tokens.
fn parse_span_attribute(token: &str, attr: &str) -> Option<u32> {
    let prefix = format!(" {attr}=\"");
    let stripped = token.strip_prefix(&prefix)?;
    let value = stripped.strip_suffix('"')?;
    value.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_span_attribute() {
        assert_eq!(parse_span_attribute(" rowspan=\"2\"", "rowspan"), Some(2));
        assert_eq!(parse_span_attribute(" colspan=\"5\"", "colspan"), Some(5));
        assert_eq!(parse_span_attribute(" rowspan=\"16\"", "rowspan"), Some(16));
        assert_eq!(parse_span_attribute("<td>", "rowspan"), None);
    }

    #[test]
    fn test_parse_simple_table() {
        let result = SlaNetResult {
            structure_tokens: vec![
                "<thead>".into(),
                "<tr>".into(),
                "<td>".into(),
                "</td>".into(),
                "<td>".into(),
                "</td>".into(),
                "</tr>".into(),
                "</thead>".into(),
                "<tbody>".into(),
                "<tr>".into(),
                "<td>".into(),
                "</td>".into(),
                "<td>".into(),
                "</td>".into(),
                "</tr>".into(),
                "</tbody>".into(),
            ],
            cell_bboxes: vec![
                BBox::new(0.0, 0.0, 50.0, 20.0),
                BBox::new(50.0, 0.0, 100.0, 20.0),
                BBox::new(0.0, 20.0, 50.0, 40.0),
                BBox::new(50.0, 20.0, 100.0, 40.0),
            ],
        };

        let rows = parse_table_structure(&result);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 2);
        assert_eq!(rows[1].len(), 2);
        assert!(rows[0][0].bbox.is_some());
    }

    #[test]
    fn test_parse_table_with_colspan() {
        let result = SlaNetResult {
            structure_tokens: vec![
                "<tr>".into(),
                "<td".into(),
                " colspan=\"2\"".into(),
                ">".into(),
                "</td>".into(),
                "</tr>".into(),
                "<tr>".into(),
                "<td>".into(),
                "</td>".into(),
                "<td>".into(),
                "</td>".into(),
                "</tr>".into(),
            ],
            cell_bboxes: vec![
                BBox::new(0.0, 0.0, 100.0, 20.0),
                BBox::new(0.0, 20.0, 50.0, 40.0),
                BBox::new(50.0, 20.0, 100.0, 40.0),
            ],
        };

        let rows = parse_table_structure(&result);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 1);
        assert_eq!(rows[0][0].colspan, 2);
        assert_eq!(rows[1].len(), 2);
    }

    #[test]
    fn test_parse_empty_cells() {
        let result = SlaNetResult {
            structure_tokens: vec!["<tr>".into(), "<td></td>".into(), "<td></td>".into(), "</tr>".into()],
            cell_bboxes: vec![BBox::new(0.0, 0.0, 50.0, 20.0), BBox::new(50.0, 0.0, 100.0, 20.0)],
        };

        let rows = parse_table_structure(&result);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 2);
    }

    #[test]
    fn test_vocabulary_size() {
        assert_eq!(VOCABULARY.len(), 50);
        assert_eq!(VOCABULARY[0], "sos");
        assert_eq!(VOCABULARY[49], "eos");
    }

    #[test]
    fn test_decode_cell_bbox_identity() {
        // Test with scale=1.0, no padding — coords should map directly
        let raw = [0.0f32; 8]; // All zeros → sigmoid(0) = 0.5
        let bbox = decode_cell_bbox(&raw, 1.0, 0, 0, 488, 488, &[1, 10, 8]);
        // sigmoid(0) = 0.5, so x = 0.5 * 488 = 244
        assert!((bbox.x1 - 244.0).abs() < 0.1);
        assert!((bbox.y1 - 244.0).abs() < 0.1);
    }
}
