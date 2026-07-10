#' @param backend Character string naming the OCR backend (e.g., "tesseract", "paddle-ocr").
#' @return Logical indicating if the backend name is valid.
validate_ocr_backend_name <- function(backend) {
  stopifnot(is.character(backend), length(backend) == 1L)
  check_native_result(validate_ocr_backend_name_native(backend))
}

#' @param code Character string language code (e.g., "eng", "deu", "en").
#' @return Logical indicating if the code is valid.
validate_language_code <- function(code) {
  stopifnot(is.character(code), length(code) == 1L)
  check_native_result(validate_language_code_native(code))
}

#' @param format Character string output format (e.g., "text", "markdown", "html", "json").
#' @return Logical indicating if the format is valid.
validate_output_format <- function(format) {
  stopifnot(is.character(format), length(format) == 1L)
  check_native_result(validate_output_format_native(format))
}
