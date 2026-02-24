#' Detect MIME type from raw bytes
#'
#' @param data Raw vector of bytes.
#' @return Character string with detected MIME type.
#' @export
detect_mime_type <- function(data) {
  detect_mime_type_native(data)
}

#' Detect MIME type from file path
#'
#' @param path Path to the file.
#' @return Character string with detected MIME type.
#' @export
detect_mime_type_from_path <- function(path) {
  detect_mime_type_from_path_native(path)
}

#' Get file extensions for a MIME type
#'
#' @param mime_type MIME type string.
#' @return Character vector of file extensions.
#' @export
get_extensions_for_mime <- function(mime_type) {
  get_extensions_for_mime_native(mime_type)
}

#' Validate a MIME type string
#'
#' @param mime_type MIME type string to validate.
#' @return Logical indicating if the MIME type is valid.
#' @export
validate_mime_type <- function(mime_type) {
  validate_mime_type_native(mime_type)
}
