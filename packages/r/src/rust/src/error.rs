//! Error mapping from kreuzberg errors to R errors

/// Convert a kreuzberg error to an extendr error string
pub fn to_r_error<E: std::fmt::Display>(err: E) -> extendr_api::Error {
    extendr_api::Error::Other(err.to_string())
}
