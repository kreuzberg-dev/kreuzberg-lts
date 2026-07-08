//! Pure scoring primitives for sparse and late-interaction retrieval.
//!
//! Kept free of any backend dependency so both the in-memory and SQLite
//! backends (and any future backend) share one implementation.

use crate::types::{MultiVector, SparseVector};

/// Late-interaction MaxSim: for each query token row, take the maximum
/// dot-product over all document token rows, then sum over query rows.
///
/// Returns `0.0` if either multi-vector is empty (no tokens) or their `dim`s
/// mismatch — this is a safe neutral score rather than a panic, since a
/// dimension mismatch between an embedder's query and document encodings is a
/// caller bug, not something the scoring primitive should ever fabricate a
/// comparison against.
pub(crate) fn max_sim(query: &MultiVector, doc: &MultiVector) -> f32 {
    if query.dim != doc.dim || query.num_tokens == 0 || doc.num_tokens == 0 {
        return 0.0;
    }
    query
        .rows()
        .map(|q_row| {
            doc.rows()
                .map(|d_row| dot(q_row, d_row))
                .fold(f32::NEG_INFINITY, f32::max)
        })
        .filter(|s| s.is_finite())
        .sum()
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Sparse dot product over two sorted, ascending-index sparse vectors, merged
/// in a single linear pass (both inputs are assumed sorted per
/// [`SparseVector`]'s contract).
pub(crate) fn sparse_dot(a: &SparseVector, b: &SparseVector) -> f32 {
    // Defence in depth: the store validates `is_well_formed` before scoring, but
    // bound the cursors by the shorter of each vector's parallel arrays so a
    // malformed `SparseVector` (values shorter than indices) can never index
    // out of bounds and panic the retrieve call.
    let a_len = a.indices.len().min(a.values.len());
    let b_len = b.indices.len().min(b.values.len());
    let mut i = 0usize;
    let mut j = 0usize;
    let mut sum = 0.0f32;
    while i < a_len && j < b_len {
        match a.indices[i].cmp(&b.indices[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                sum += a.values[i] * b.values[j];
                i += 1;
                j += 1;
            }
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sv(indices: &[u32], values: &[f32]) -> SparseVector {
        SparseVector {
            indices: indices.to_vec(),
            values: values.to_vec(),
        }
    }

    fn mv(num_tokens: u32, dim: u32, data: &[f32]) -> MultiVector {
        MultiVector {
            num_tokens,
            dim,
            data: data.to_vec(),
        }
    }

    #[test]
    fn sparse_dot_computes_overlap_only() {
        // a: {1: 2.0, 3: 1.0, 5: 4.0}, b: {2: 1.0, 3: 3.0, 5: 2.0}
        // overlap: index 3 -> 1.0*3.0=3.0, index 5 -> 4.0*2.0=8.0 => 11.0
        let a = sv(&[1, 3, 5], &[2.0, 1.0, 4.0]);
        let b = sv(&[2, 3, 5], &[1.0, 3.0, 2.0]);
        assert_eq!(sparse_dot(&a, &b), 11.0);
    }

    #[test]
    fn sparse_dot_no_overlap_is_zero() {
        let a = sv(&[1, 2], &[1.0, 1.0]);
        let b = sv(&[3, 4], &[1.0, 1.0]);
        assert_eq!(sparse_dot(&a, &b), 0.0);
    }

    #[test]
    fn sparse_dot_empty_is_zero() {
        assert_eq!(sparse_dot(&SparseVector::default(), &SparseVector::default()), 0.0);
    }

    #[test]
    fn multi_vector_rows_splits_row_major_data() {
        // 2 tokens x dim 3
        let v = mv(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let rows: Vec<&[f32]> = v.rows().collect();
        assert_eq!(rows, vec![&[1.0, 2.0, 3.0][..], &[4.0, 5.0, 6.0][..]]);
    }

    #[test]
    fn multi_vector_rows_dim_zero_is_empty() {
        let v = mv(0, 0, &[]);
        assert_eq!(v.rows().count(), 0);
        // Guard against a hypothetical malformed record where dim=0 but data
        // is non-empty: chunks_exact(0) would panic, so rows() must special-case it.
        let malformed = mv(0, 0, &[1.0, 2.0, 3.0]);
        assert_eq!(malformed.rows().count(), 0);
    }

    #[test]
    fn max_sim_hand_computed() {
        // Query: 2 tokens, dim 2: [[1,0], [0,1]]
        // Doc:   2 tokens, dim 2: [[1,0], [1,1]]
        // Row0 [1,0] vs doc rows: dot([1,0],[1,0])=1, dot([1,0],[1,1])=1 -> max=1
        // Row1 [0,1] vs doc rows: dot([0,1],[1,0])=0, dot([0,1],[1,1])=1 -> max=1
        // Sum = 2.0
        let q = mv(2, 2, &[1.0, 0.0, 0.0, 1.0]);
        let d = mv(2, 2, &[1.0, 0.0, 1.0, 1.0]);
        assert_eq!(max_sim(&q, &d), 2.0);
    }

    #[test]
    fn max_sim_dim_mismatch_is_zero() {
        let q = mv(1, 2, &[1.0, 0.0]);
        let d = mv(1, 3, &[1.0, 0.0, 0.0]);
        assert_eq!(max_sim(&q, &d), 0.0);
    }

    #[test]
    fn max_sim_empty_is_zero() {
        let q = MultiVector::default();
        let d = mv(1, 2, &[1.0, 0.0]);
        assert_eq!(max_sim(&q, &d), 0.0);
        assert_eq!(max_sim(&d, &MultiVector::default()), 0.0);
    }
}
