//! Document orientation detection using PP-LCNet_x1_0_doc_ori.
//!
//! Detects page-level orientation (0°, 90°, 180°, 270°) for scanned documents.
//! Used as a PaddleOCR-native alternative to Tesseract's `DetectOrientationScript`
//! when `auto_rotate` is enabled with the PaddleOCR backend.

use std::sync::Mutex;

use image::RgbImage;
use ort::session::Session;
use ort::session::builder::SessionBuilder;
use ort::value::Tensor;

use crate::Result;
use crate::error::KreuzbergError;

use super::model_manager::ModelManager;

// PP-LCNet_x1_0_doc_ori preprocessing constants.
// Input: resize short side to 256, center crop 224×224, ImageNet normalize.
const INPUT_SIZE: u32 = 224;
const RESIZE_SHORT: u32 = 256;

// ImageNet normalization: (pixel - MEAN) * NORM
const MEAN: [f32; 3] = [0.485 * 255.0, 0.456 * 255.0, 0.406 * 255.0];
const NORM: [f32; 3] = [1.0 / (0.229 * 255.0), 1.0 / (0.224 * 255.0), 1.0 / (0.225 * 255.0)];

/// Output labels: index -> degrees.
const ORIENTATION_LABELS: [u32; 4] = [0, 90, 180, 270];

/// Document orientation detection result.
#[derive(Debug, Clone, Copy)]
pub struct OrientationResult {
    /// Detected orientation in degrees (0, 90, 180, or 270).
    pub degrees: u32,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
}

/// Detects document page orientation using the PP-LCNet model.
pub struct DocOrientationDetector {
    session: Mutex<Option<Session>>,
    model_manager: ModelManager,
}

impl DocOrientationDetector {
    /// Creates a new detector. The model is loaded lazily on first use.
    pub fn new(model_manager: ModelManager) -> Self {
        Self {
            session: Mutex::new(None),
            model_manager,
        }
    }

    /// Detect document page orientation.
    ///
    /// Returns the detected orientation (0°, 90°, 180°, 270°) and confidence.
    pub fn detect(&self, image: &RgbImage) -> Result<OrientationResult> {
        let session_guard = self.get_or_init_session()?;
        let mut session = session_guard.lock().map_err(|e| KreuzbergError::Plugin {
            message: format!("Failed to acquire doc_ori session lock: {e}"),
            plugin_name: "paddle-ocr".to_string(),
        })?;

        let session = session.as_mut().ok_or_else(|| KreuzbergError::Plugin {
            message: "Doc orientation session not initialized".to_string(),
            plugin_name: "paddle-ocr".to_string(),
        })?;

        // Preprocess: resize short side to 256, center crop 224×224
        let preprocessed = preprocess(image);

        // Build input tensor: [1, 3, 224, 224]
        let input_tensor = normalize(&preprocessed);
        let tensor = Tensor::from_array(input_tensor).map_err(|e| KreuzbergError::Ocr {
            message: format!("Failed to create doc_ori input tensor: {e}"),
            source: None,
        })?;

        // Run inference (input name defaults to "x" for PP-LCNet models)
        let outputs = session
            .run(ort::inputs!["x" => tensor])
            .map_err(|e| KreuzbergError::Ocr {
                message: format!("Doc orientation inference failed: {e}"),
                source: None,
            })?;

        // Parse output: find argmax
        let (_, output_value) = outputs.iter().next().ok_or_else(|| KreuzbergError::Ocr {
            message: "No output from doc orientation model".to_string(),
            source: None,
        })?;

        let scores: Vec<f32> = output_value
            .try_extract_tensor::<f32>()
            .map_err(|e| KreuzbergError::Ocr {
                message: format!("Failed to extract doc_ori output: {e}"),
                source: None,
            })?
            .1
            .to_vec();

        // Softmax + argmax
        let max_score = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_scores: Vec<f32> = scores.iter().map(|&s| (s - max_score).exp()).collect();
        let sum_exp: f32 = exp_scores.iter().sum();
        let probabilities: Vec<f32> = exp_scores.iter().map(|&e| e / sum_exp).collect();

        let (best_idx, &best_prob) = probabilities
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0, &0.0));

        let degrees = ORIENTATION_LABELS.get(best_idx).copied().unwrap_or(0);

        Ok(OrientationResult {
            degrees,
            confidence: best_prob,
        })
    }

    /// Get or initialize the ONNX session.
    fn get_or_init_session(&self) -> Result<&Mutex<Option<Session>>> {
        {
            let guard = self.session.lock().map_err(|e| KreuzbergError::Plugin {
                message: format!("Failed to acquire doc_ori lock: {e}"),
                plugin_name: "paddle-ocr".to_string(),
            })?;
            if guard.is_some() {
                drop(guard);
                return Ok(&self.session);
            }
        }

        // Download model if needed
        let model_dir = self.model_manager.ensure_doc_ori_model()?;
        let model_path = model_dir.join("model.onnx");

        crate::ort_discovery::ensure_ort_available();

        let num_threads = num_cpus::get().min(4);
        let session = SessionBuilder::new()
            .map_err(|e| KreuzbergError::Ocr {
                message: format!("Failed to create doc_ori session builder: {e}"),
                source: None,
            })?
            .with_intra_threads(num_threads)
            .map_err(|e| KreuzbergError::Ocr {
                message: format!("Failed to set doc_ori thread count: {e}"),
                source: None,
            })?
            .commit_from_file(&model_path)
            .map_err(|e| KreuzbergError::Ocr {
                message: format!("Failed to load doc_ori model: {e}"),
                source: None,
            })?;

        let mut guard = self.session.lock().map_err(|e| KreuzbergError::Plugin {
            message: format!("Failed to acquire doc_ori lock: {e}"),
            plugin_name: "paddle-ocr".to_string(),
        })?;

        if guard.is_none() {
            *guard = Some(session);
            tracing::info!("Doc orientation model loaded");
        }

        drop(guard);
        Ok(&self.session)
    }
}

/// Resize short side to 256, then center crop to 224×224.
fn preprocess(image: &RgbImage) -> RgbImage {
    let (w, h) = (image.width(), image.height());

    // Resize: scale so short side = RESIZE_SHORT
    let (new_w, new_h) = if w < h {
        let scale = RESIZE_SHORT as f32 / w as f32;
        (RESIZE_SHORT, (h as f32 * scale).round() as u32)
    } else {
        let scale = RESIZE_SHORT as f32 / h as f32;
        ((w as f32 * scale).round() as u32, RESIZE_SHORT)
    };

    let resized = image::imageops::resize(image, new_w, new_h, image::imageops::FilterType::Triangle);

    // Center crop to INPUT_SIZE × INPUT_SIZE
    let x_offset = (new_w.saturating_sub(INPUT_SIZE)) / 2;
    let y_offset = (new_h.saturating_sub(INPUT_SIZE)) / 2;
    let crop_w = INPUT_SIZE.min(new_w);
    let crop_h = INPUT_SIZE.min(new_h);

    image::imageops::crop_imm(&resized, x_offset, y_offset, crop_w, crop_h).to_image()
}

/// Normalize image to [1, 3, H, W] tensor with ImageNet mean/std.
fn normalize(image: &RgbImage) -> ndarray::Array4<f32> {
    let (w, h) = (image.width() as usize, image.height() as usize);
    let mut tensor = ndarray::Array4::<f32>::zeros((1, 3, h, w));

    for y in 0..h {
        for x in 0..w {
            let pixel = image.get_pixel(x as u32, y as u32);
            for ch in 0..3 {
                let value = pixel[ch] as f32;
                tensor[[0, ch, y, x]] = (value - MEAN[ch]) * NORM[ch];
            }
        }
    }

    tensor
}
