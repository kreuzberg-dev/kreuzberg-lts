//! Sparse (SPLADE) embedding inference engine.
//!
//! Runs a `BertForMaskedLM` ONNX model and turns its `[batch, seq, vocab]` MLM
//! logits into sparse vocabulary vectors via SPLADE pooling:
//! `log(1 + relu(logits))`, attention-masked, max-pooled over the sequence,
//! L2-normalized, then thresholded to keep only the non-zero terms.
//!
//! Like [`crate::embeddings::engine::EmbeddingEngine`], `embed()` takes `&self`
//! so a single engine can serve concurrent callers via `Arc` — `ort::Session::run`
//! is thread-safe despite its `&mut self` signature.

use ndarray::{ArrayView, Axis, Dim, IxDynImpl};
use ort::session::Session;
use ort::value::Value;
use tokenizers::Tokenizer;

use super::SparseEmbedding;

/// Errors raised by the sparse-embedding engine.
///
/// Rust-only: the `Ort` variant wraps `ort::Error`, which has no faithful
/// binding representation. Public callers receive `crate::XbergError` instead.
#[cfg_attr(alef, alef(skip))]
#[derive(Debug)]
pub enum SparseEmbedError {
    /// Tokenization failed with the given message.
    Tokenizer(String),
    /// ONNX Runtime returned an error during inference.
    Ort(ort::Error),
    /// The model output tensor had an unexpected shape.
    Shape(String),
    /// The model produced no output tensors.
    NoOutput,
}

impl From<ort::Error> for SparseEmbedError {
    fn from(e: ort::Error) -> Self {
        SparseEmbedError::Ort(e)
    }
}

/// SPLADE sparse-embedding model with thread-safe inference.
///
/// Rust-only: an opaque ORT-backed handle with no faithful binding
/// representation (mirrors `reranking::engine::RerankerEngine`). Bindings drive
/// inference through the module-level functions, not this type.
#[cfg_attr(alef, alef(skip))]
pub struct SparseEmbeddingEngine {
    tokenizer: Tokenizer,
    session: Session,
    need_token_type_ids: bool,
}

impl SparseEmbeddingEngine {
    /// Create a new engine from a pre-built session and tokenizer.
    pub(crate) fn new(tokenizer: Tokenizer, session: Session) -> Self {
        let need_token_type_ids = session.inputs().iter().any(|input| input.name() == "token_type_ids");
        Self {
            tokenizer,
            session,
            need_token_type_ids,
        }
    }

    /// Generate sparse embeddings for a batch of texts.
    ///
    /// Thread-safe: multiple threads may call `embed()` concurrently on the same
    /// engine instance.
    pub(crate) fn embed<S: AsRef<str>>(
        &self,
        texts: &[S],
        batch_size: usize,
    ) -> Result<Vec<SparseEmbedding>, SparseEmbedError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let batch_size = if batch_size == 0 { 16 } else { batch_size };

        let mut all = Vec::with_capacity(texts.len());
        for batch in texts.chunks(batch_size) {
            all.extend(self.embed_batch(batch)?);
        }
        Ok(all)
    }

    fn embed_batch<S: AsRef<str>>(&self, batch: &[S]) -> Result<Vec<SparseEmbedding>, SparseEmbedError> {
        let inputs: Vec<&str> = batch.iter().map(|t| t.as_ref()).collect();
        let encodings = self
            .tokenizer
            .encode_batch(inputs, true)
            .map_err(|e| SparseEmbedError::Tokenizer(e.to_string()))?;

        let encoding_length = encodings
            .first()
            .ok_or_else(|| SparseEmbedError::Tokenizer("Empty encodings".to_string()))?
            .len();
        let batch_size = batch.len();
        let max_size = encoding_length * batch_size;

        let mut ids_array = Vec::with_capacity(max_size);
        let mut mask_array = Vec::with_capacity(max_size);
        let mut type_ids_array = Vec::with_capacity(max_size);
        for encoding in &encodings {
            ids_array.extend(encoding.get_ids().iter().map(|&x| x as i64));
            mask_array.extend(encoding.get_attention_mask().iter().map(|&x| x as i64));
            type_ids_array.extend(encoding.get_type_ids().iter().map(|&x| x as i64));
        }

        let ids_tensor = ndarray::Array::from_shape_vec((batch_size, encoding_length), ids_array)
            .map_err(|e| SparseEmbedError::Shape(e.to_string()))?;
        let type_ids_tensor = ndarray::Array::from_shape_vec((batch_size, encoding_length), type_ids_array)
            .map_err(|e| SparseEmbedError::Shape(e.to_string()))?;
        let mask_nd = ndarray::Array::from_shape_vec((batch_size, encoding_length), mask_array)
            .map_err(|e| SparseEmbedError::Shape(e.to_string()))?;

        let mut session_inputs = ort::inputs![
            "input_ids" => Value::from_array(ids_tensor)?,
            "attention_mask" => Value::from_array(mask_nd.clone())?,
        ];
        if self.need_token_type_ids {
            session_inputs.push(("token_type_ids".into(), Value::from_array(type_ids_tensor)?.into()));
        }

        // SAFETY: ort::Session::run() takes &mut self but delegates to run_inner(&self)
        // with zero actual mutation. The ONNX Runtime C API (OrtApi::Run) is documented
        // as thread-safe for concurrent Run() calls on the same session.
        #[allow(unsafe_code)]
        let outputs = unsafe {
            let session_ptr = &self.session as *const Session as *mut Session;
            (*session_ptr).run(session_inputs)
        }
        .map_err(SparseEmbedError::Ort)?;

        // MLM logits: [batch, seq, vocab].
        let (_, output_value) = outputs.iter().next().ok_or(SparseEmbedError::NoOutput)?;
        let logits: ArrayView<f32, Dim<IxDynImpl>> = output_value.try_extract_array().map_err(SparseEmbedError::Ort)?;
        let logits = logits
            .into_dimensionality::<ndarray::Ix3>()
            .map_err(|e| SparseEmbedError::Shape(format!("expected [batch, seq, vocab] logits: {e}")))?;

        splade_pool(&logits, &mask_nd)
    }
}

/// SPLADE pooling: `log(1 + relu(x))` → attention-masked → max-pool over the
/// sequence → L2-normalize → keep strictly-positive terms as a sparse vector.
///
/// `logits` is `[batch, seq, vocab]`; `mask` is `[batch, seq]` (1 = real token,
/// 0 = padding). Returns one [`SparseEmbedding`] per batch row, with ascending
/// `indices`.
///
/// # Errors
///
/// [`SparseEmbedError::Shape`] if the attention mask cannot broadcast against the
/// logits — pooling over unmasked padding would silently corrupt the vector, so
/// this fails loudly rather than degrading.
fn splade_pool(
    logits: &ndarray::ArrayView3<f32>,
    mask: &ndarray::Array2<i64>,
) -> Result<Vec<SparseEmbedding>, SparseEmbedError> {
    // log(1 + relu(x)) — always >= 0, and 0 exactly where x <= 0.
    let relu_log = logits.mapv(|x| (1.0_f32 + x.max(0.0)).ln());

    // Zero out padded positions by broadcasting the mask over the vocab axis.
    let mask_f = mask.mapv(|x| x as f32);
    let mask_axis = mask_f.insert_axis(Axis(2));
    let mask_b = mask_axis.broadcast(relu_log.dim()).ok_or_else(|| {
        SparseEmbedError::Shape(format!(
            "attention mask {:?} cannot broadcast to logits {:?}",
            mask.dim(),
            relu_log.dim()
        ))
    })?;
    let weighted = &relu_log * &mask_b;

    // Max-pool over the sequence axis -> [batch, vocab].
    let scores = weighted.fold_axis(Axis(1), f32::NEG_INFINITY, |&r, &v| r.max(v));

    Ok(scores
        .outer_iter()
        .map(|row| {
            let norm = row.iter().map(|&v| v * v).sum::<f32>().sqrt();
            let mut indices = Vec::new();
            let mut values = Vec::new();
            if norm > 0.0 {
                for (i, &v) in row.iter().enumerate() {
                    if v > 0.0 {
                        indices.push(i as u32);
                        values.push(v / norm);
                    }
                }
            }
            SparseEmbedding { indices, values }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array2, Array3};

    #[test]
    fn splade_pool_produces_normalized_sparse_vector() {
        // batch=1, seq=2, vocab=3. Only two vocab terms ever activate.
        // token 0 logits: [ 2.0, -1.0, 0.5 ]
        // token 1 logits: [ 0.0,  3.0, -2.0 ]
        let logits = Array3::from_shape_vec((1, 2, 3), vec![2.0, -1.0, 0.5, 0.0, 3.0, -2.0]).unwrap();
        let mask = Array2::from_shape_vec((1, 2), vec![1_i64, 1]).unwrap();
        let out = splade_pool(&logits.view(), &mask).unwrap();
        assert_eq!(out.len(), 1);
        let se = &out[0];

        // Expected pre-norm scores per vocab term (max over seq of log(1+relu)):
        // term0: max(ln(3), ln(1)) = ln(3)
        // term1: max(ln(1)=0, ln(4)) = ln(4)
        // term2: max(ln(1.5), ln(1)=0) = ln(1.5)
        let s0 = (3.0_f32).ln();
        let s1 = (4.0_f32).ln();
        let s2 = (1.5_f32).ln();
        let norm = (s0 * s0 + s1 * s1 + s2 * s2).sqrt();

        // All three terms are strictly positive -> all present, ascending indices.
        assert_eq!(se.indices, vec![0, 1, 2]);
        assert_eq!(se.values.len(), 3);
        assert!((se.values[0] - s0 / norm).abs() < 1e-5);
        assert!((se.values[1] - s1 / norm).abs() < 1e-5);
        assert!((se.values[2] - s2 / norm).abs() < 1e-5);

        // L2 norm of the emitted values is 1.
        let out_norm = se.values.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!((out_norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn splade_pool_drops_nonpositive_terms_and_masks_padding() {
        // vocab term 1 only activates on a padded token -> must be dropped.
        // batch=1, seq=2, vocab=2.
        // token 0 (real): [ 1.0, -5.0 ]
        // token 1 (pad):  [ -5.0, 4.0 ]
        let logits = Array3::from_shape_vec((1, 2, 2), vec![1.0, -5.0, -5.0, 4.0]).unwrap();
        let mask = Array2::from_shape_vec((1, 2), vec![1_i64, 0]).unwrap(); // second token padded
        let out = splade_pool(&logits.view(), &mask).unwrap();
        let se = &out[0];
        // Only term 0 survives (term 1's activation was on the masked token).
        assert_eq!(se.indices, vec![0]);
        assert_eq!(se.values.len(), 1);
        assert!((se.values[0] - 1.0).abs() < 1e-5); // single term -> normalized to 1.0
    }

    #[test]
    fn splade_pool_all_zero_row_yields_empty_sparse_vector() {
        // All logits <= 0 -> relu_log all zero -> empty sparse vector, no NaN.
        let logits = Array3::from_shape_vec((1, 2, 2), vec![-1.0, -2.0, -3.0, -4.0]).unwrap();
        let mask = Array2::from_shape_vec((1, 2), vec![1_i64, 1]).unwrap();
        let out = splade_pool(&logits.view(), &mask).unwrap();
        assert!(out[0].indices.is_empty());
        assert!(out[0].values.is_empty());
    }

    #[test]
    fn splade_pool_errors_on_mask_shape_mismatch() {
        // mask seq (3) does not match logits seq (2) -> broadcast fails -> error,
        // not a silent pool over unmasked padding.
        let logits = Array3::from_shape_vec((1, 2, 2), vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        let mask = Array2::from_shape_vec((1, 3), vec![1_i64, 1, 1]).unwrap();
        let err = splade_pool(&logits.view(), &mask).expect_err("mask/logits mismatch must error");
        assert!(matches!(err, SparseEmbedError::Shape(_)));
    }
}
