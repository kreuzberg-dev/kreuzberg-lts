#' @param x A named list from native tree-sitter processing.
#' @return Object with class \code{kreuzberg_code_process_result}.
as_code_process_result <- function(x) {
  if (!inherits(x, "kreuzberg_code_process_result")) {
    class(x) <- c("kreuzberg_code_process_result", "list")
  }
  x
}

#' @param x A \code{kreuzberg_code_process_result} object.
#' @param ... Additional arguments (ignored).
print.kreuzberg_code_process_result <- function(x, ...) {
  cat("<kreuzberg_code_process_result>\n")
  if (!is.null(x$language)) cat("  Language:", x$language, "\n")
  if (!is.null(x$metrics)) {
    cat("  Metrics:\n")
    cat("    Total lines:", x$metrics$total_lines %||% 0, "\n")
    cat("    Code lines:", x$metrics$code_lines %||% 0, "\n")
    cat("    Comment lines:", x$metrics$comment_lines %||% 0, "\n")
    cat("    Blank lines:", x$metrics$blank_lines %||% 0, "\n")
    cat("    Error count:", x$metrics$error_count %||% 0, "\n")
  }
  if (!is.null(x$structure)) cat("  Structure items:", length(x$structure), "\n")
  if (!is.null(x$imports)) cat("  Imports:", length(x$imports), "\n")
  if (!is.null(x$exports)) cat("  Exports:", length(x$exports), "\n")
  if (!is.null(x$comments)) cat("  Comments:", length(x$comments), "\n")
  if (!is.null(x$docstrings)) cat("  Docstrings:", length(x$docstrings), "\n")
  if (!is.null(x$symbols)) cat("  Symbols:", length(x$symbols), "\n")
  if (!is.null(x$diagnostics)) cat("  Diagnostics:", length(x$diagnostics), "\n")
  if (!is.null(x$chunks)) cat("  Chunks:", length(x$chunks), "\n")
  invisible(x)
}

#' @param object A \code{kreuzberg_code_process_result} object.
#' @param ... Additional arguments (ignored).
summary.kreuzberg_code_process_result <- function(object, ...) {
  cat("<kreuzberg_code_process_result summary>\n")
  cat("  Language:       ", object$language %||% "(unknown)", "\n")
  cat("  Total lines:    ", object$metrics$total_lines %||% 0, "\n")
  cat("  Code lines:     ", object$metrics$code_lines %||% 0, "\n")
  cat("  Comment lines:  ", object$metrics$comment_lines %||% 0, "\n")
  cat("  Structure:      ", length(object$structure %||% list()), "\n")
  cat("  Imports:        ", length(object$imports %||% list()), "\n")
  cat("  Exports:        ", length(object$exports %||% list()), "\n")
  cat("  Comments:       ", length(object$comments %||% list()), "\n")
  cat("  Docstrings:     ", length(object$docstrings %||% list()), "\n")
  cat("  Symbols:        ", length(object$symbols %||% list()), "\n")
  cat("  Diagnostics:    ", length(object$diagnostics %||% list()), "\n")
  cat("  Chunks:         ", length(object$chunks %||% list()), "\n")
  invisible(object)
}

#' @param x A \code{kreuzberg_code_process_result} object.
#' @param ... Additional arguments (ignored).
#' @return A character string representation.
format.kreuzberg_code_process_result <- function(x, ...) {
  paste0(
    "<kreuzberg_code_process_result: ",
    x$language %||% "unknown",
    ", ",
    x$metrics$total_lines %||% 0,
    " lines>"
  )
}
