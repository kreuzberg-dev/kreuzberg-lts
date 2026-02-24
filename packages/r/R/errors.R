#' Kreuzberg error condition
#'
#' @param message Error message.
#' @param class Additional error class.
#' @param call The call that triggered the error.
#' @return A condition object.
#' @keywords internal
kreuzberg_error <- function(message, class = NULL, call = NULL) {
  structure(
    class = c(class, "kreuzberg_error", "error", "condition"),
    list(message = message, call = call)
  )
}
