#' Extract content from a file (synchronous)
#'
#' @param path Path to the file.
#' @param mime_type Optional MIME type override.
#' @param config Optional extraction configuration from \code{extraction_config()}.
#' @return A \code{kreuzberg_result} object.
#' @export
extract_file_sync <- function(path, mime_type = NULL, config = NULL) {
  config_json <- if (!is.null(config)) jsonlite::toJSON(config, auto_unbox = TRUE) else NULL
  result <- extract_file_sync_native(path, mime_type, config_json)
  as_kreuzberg_result(result)
}

#' Extract content from a file (async, blocks in R)
#'
#' @param path Path to the file.
#' @param mime_type Optional MIME type override.
#' @param config Optional extraction configuration from \code{extraction_config()}.
#' @return A \code{kreuzberg_result} object.
#' @export
extract_file <- function(path, mime_type = NULL, config = NULL) {
  config_json <- if (!is.null(config)) jsonlite::toJSON(config, auto_unbox = TRUE) else NULL
  result <- extract_file_native(path, mime_type, config_json)
  as_kreuzberg_result(result)
}

#' Extract content from raw bytes (synchronous)
#'
#' @param data Raw vector of bytes.
#' @param mime_type MIME type of the data.
#' @param config Optional extraction configuration from \code{extraction_config()}.
#' @return A \code{kreuzberg_result} object.
#' @export
extract_bytes_sync <- function(data, mime_type, config = NULL) {
  config_json <- if (!is.null(config)) jsonlite::toJSON(config, auto_unbox = TRUE) else NULL
  result <- extract_bytes_sync_native(data, mime_type, config_json)
  as_kreuzberg_result(result)
}

#' Extract content from raw bytes (async, blocks in R)
#'
#' @param data Raw vector of bytes.
#' @param mime_type MIME type of the data.
#' @param config Optional extraction configuration from \code{extraction_config()}.
#' @return A \code{kreuzberg_result} object.
#' @export
extract_bytes <- function(data, mime_type, config = NULL) {
  config_json <- if (!is.null(config)) jsonlite::toJSON(config, auto_unbox = TRUE) else NULL
  result <- extract_bytes_native(data, mime_type, config_json)
  as_kreuzberg_result(result)
}
