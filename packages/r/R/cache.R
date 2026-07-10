#' @return Invisible NULL on success.
clear_cache <- function() {
  check_native_result(clear_cache_native())
}

#' @return A named list with:
cache_stats <- function() {
  check_native_result(cache_stats_native())
}
