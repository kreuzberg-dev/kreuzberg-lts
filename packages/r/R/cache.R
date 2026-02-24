#' Clear the extraction cache
#'
#' @export
clear_cache <- function() {
  invisible(.Call("wrap__clear_cache"))
}

#' Get cache statistics
#'
#' @return Named list with total_entries and total_size_bytes.
#' @export
cache_stats <- function() {
  .Call("wrap__cache_stats")
}
