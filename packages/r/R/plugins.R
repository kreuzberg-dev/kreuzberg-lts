#' @export
register_ocr_backend <- function(name, callback) {
  register_ocr_backend_native(name, callback)
}

#' @export
unregister_ocr_backend <- function(name) {
  unregister_ocr_backend_native(name)
}

#' @export
list_ocr_backends <- function() {
  list_ocr_backends_native()
}

#' @export
clear_ocr_backends <- function() {
  clear_ocr_backends_native()
}

#' @export
register_post_processor <- function(name, callback) {
  register_post_processor_native(name, callback)
}

#' @export
unregister_post_processor <- function(name) {
  unregister_post_processor_native(name)
}

#' @export
list_post_processors <- function() {
  list_post_processors_native()
}

#' @export
clear_post_processors <- function() {
  clear_post_processors_native()
}

#' @export
register_validator <- function(name, callback) {
  register_validator_native(name, callback)
}

#' @export
unregister_validator <- function(name) {
  unregister_validator_native(name)
}

#' @export
list_validators <- function() {
  list_validators_native()
}

#' @export
clear_validators <- function() {
  clear_validators_native()
}

#' @export
list_document_extractors <- function() {
  list_document_extractors_native()
}

#' @export
unregister_document_extractor <- function(name) {
  unregister_document_extractor_native(name)
}

#' @export
clear_document_extractors <- function() {
  clear_document_extractors_native()
}
