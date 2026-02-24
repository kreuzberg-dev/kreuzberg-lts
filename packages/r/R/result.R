#' Convert a list to a kreuzberg_result S3 object
#'
#' @param x A named list from native extraction.
#' @return Object with class \code{kreuzberg_result}.
#' @keywords internal
as_kreuzberg_result <- function(x) {
  if (!inherits(x, "kreuzberg_result")) {
    class(x) <- c("kreuzberg_result", "list")
  }
  x
}

#' Print method for kreuzberg_result
#'
#' @param x A \code{kreuzberg_result} object.
#' @param ... Additional arguments (ignored).
#' @export
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
  invisible(x)
}
