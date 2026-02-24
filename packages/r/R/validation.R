#' Validate an OCR backend name
#'
#' @param backend Backend name string.
#' @return Logical indicating if the backend name is valid.
#' @keywords internal
validate_ocr_backend_name <- function(backend) {
  validate_ocr_backend_name_native(backend)
}

#' Validate a language code
#'
#' @param code Language code string.
#' @return Logical indicating if the code is valid.
#' @keywords internal
validate_language_code <- function(code) {
  validate_language_code_native(code)
}

#' Validate an output format
#'
#' @param format Output format string.
#' @return Logical indicating if the format is valid.
#' @keywords internal
validate_output_format <- function(format) {
  validate_output_format_native(format)
}
