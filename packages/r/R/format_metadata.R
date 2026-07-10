#'   \item{copyright}{Character or NULL. Copyright statement.}
#'   \item{license}{Character or NULL. License information.}
NULL

#' @param metadata A metadata list from an extraction result.
#' @param format_type Character string of the expected format type
#' @return Logical indicating if the metadata matches the given format type.
is_format_type <- function(metadata, format_type) {
  stopifnot(is.list(metadata), is.character(format_type), length(format_type) == 1L)
  identical(metadata[["format_type"]], format_type)
}

#' @param metadata A metadata list from an extraction result.
#' @return Named list of format-specific fields, or NULL if no format_type is present.
format_metadata_fields <- function(metadata) {
  if (is.null(metadata) || is.null(metadata[["format_type"]])) {
    return(NULL)
  }

  common_keys <- c(
    "title", "subject", "authors", "keywords", "language",
    "created_at", "modified_at", "created_by", "modified_by",
    "pages", "image_preprocessing", "json_schema", "error",
    "category", "tags", "document_version", "abstract_text",
    "output_format", "extraction_duration_ms"
  )

  format_keys <- setdiff(names(metadata), common_keys)
  metadata[format_keys]
}
