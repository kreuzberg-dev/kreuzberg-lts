#' @param x A named list from native extraction.
#' @return Object with class \code{kreuzberg_result}.
as_kreuzberg_result <- function(x) {
  if (!inherits(x, "kreuzberg_result")) {
    class(x) <- c("kreuzberg_result", "list")
  }
  x
}

#' @param x A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
print.kreuzberg_result <- function(x, ...) {
  cat("<kreuzberg_result>\n")
  if (!is.null(x$mime_type)) cat("  MIME type:", x$mime_type, "\n")
  if (!is.null(x$content)) {
    content_len <- nchar(x$content)
    cat("  Content length:", content_len, "chars\n")
    if (content_len > 0) {
      preview <- substr(x$content, 1, min(200, content_len))
      if (content_len > 200) preview <- paste0(preview, "...")
      cat("  Preview:", preview, "\n")
    }
  }
  if (!is.null(x$tables)) cat("  Tables:", length(x$tables), "\n")
  if (!is.null(x$chunks)) cat("  Chunks:", length(x$chunks), "\n")
  if (!is.null(x$images)) cat("  Images:", length(x$images), "\n")
  if (!is.null(x$pages)) cat("  Pages:", length(x$pages), "\n")
  if (!is.null(x$elements)) cat("  Elements:", length(x$elements), "\n")
  if (!is.null(x$quality_score)) cat("  Quality score:", x$quality_score, "\n")
  if (!is.null(x$detected_language)) cat("  Language:", x$detected_language, "\n")
  invisible(x)
}

#' @param object A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
summary.kreuzberg_result <- function(object, ...) {
  cat("<kreuzberg_result summary>\n")
  cat("  MIME type:       ", object$mime_type %||% "(unknown)", "\n")
  cat("  Content length:  ", nchar(object$content %||% ""), " chars\n")
  cat("  Pages:           ", length(object$pages %||% list()), "\n")
  cat("  Tables:          ", length(object$tables %||% list()), "\n")
  cat("  Chunks:          ", length(object$chunks %||% list()), "\n")
  cat("  Images:          ", length(object$images %||% list()), "\n")
  cat("  Elements:        ", length(object$elements %||% list()), "\n")
  cat("  Keywords:        ", length(object$keywords %||% list()), "\n")
  if (!is.null(object$quality_score)) {
    cat("  Quality score:   ", object$quality_score, "\n")
  }
  if (!is.null(object$detected_language)) {
    cat("  Language:        ", object$detected_language, "\n")
  }
  invisible(object)
}

#' @param x A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
#' @return A character string representation.
format.kreuzberg_result <- function(x, ...) {
  paste0(
    "<kreuzberg_result: ",
    x$mime_type %||% "unknown",
    ", ",
    nchar(x$content %||% ""),
    " chars>"
  )
}


#' @param x A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
#' @return Character string of extracted content.
content <- function(x, ...) UseMethod("content")

content.kreuzberg_result <- function(x, ...) x$content

#' @param x A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
#' @return Character string of the MIME type.
mime_type <- function(x, ...) UseMethod("mime_type")

mime_type.kreuzberg_result <- function(x, ...) x$mime_type

#' @param x A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
#' @return Integer page count.
page_count <- function(x, ...) UseMethod("page_count")

page_count.kreuzberg_result <- function(x, ...) length(x$pages %||% list())

#' @param x A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
#' @return Integer chunk count.
chunk_count <- function(x, ...) UseMethod("chunk_count")

chunk_count.kreuzberg_result <- function(x, ...) length(x$chunks %||% list())

#' @param x A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
#' @return Character string of the detected language, or NULL.
detected_language <- function(x, ...) UseMethod("detected_language")

detected_language.kreuzberg_result <- function(x, ...) x$detected_language

#' @param x A \code{kreuzberg_result} object.
#' @param name Name of the metadata field.
#' @param ... Additional arguments (ignored).
#' @return The metadata value, or NULL if not present.
metadata_field <- function(x, name, ...) UseMethod("metadata_field")

metadata_field.kreuzberg_result <- function(x, name, ...) {
  if (is.null(x$metadata)) {
    return(NULL)
  }
  x$metadata[[name]]
}
