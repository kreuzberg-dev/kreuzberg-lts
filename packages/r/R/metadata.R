#' @param data Raw vector of bytes.
#' @return Character string with detected MIME type.
detect_mime_type <- function(data) {
  stopifnot(is.raw(data))
  check_native_result(detect_mime_type_native(data))
}

#' @param path Character string path to the file.
#' @return Character string with detected MIME type.
detect_mime_type_from_path <- function(path) {
  stopifnot(is.character(path), length(path) == 1L)
  check_native_result(detect_mime_type_from_path_native(path))
}

#' @param mime_type Character string MIME type (e.g., "application/pdf").
#' @return Character vector of file extensions (e.g., c("pdf")).
get_extensions_for_mime <- function(mime_type) {
  stopifnot(is.character(mime_type), length(mime_type) == 1L)
  check_native_result(get_extensions_for_mime_native(mime_type))
}

#' @param mime_type Character string MIME type to validate.
#' @return Logical indicating if the MIME type is valid.
validate_mime_type <- function(mime_type) {
  stopifnot(is.character(mime_type), length(mime_type) == 1L)
  check_native_result(validate_mime_type_native(mime_type))
}
