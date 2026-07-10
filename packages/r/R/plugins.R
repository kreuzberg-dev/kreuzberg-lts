# --- OCR Backend Plugins ---

#' @param name Character string naming the backend.
#' @param callback Callback object for OCR processing.
#' @return Invisible NULL on success.
register_ocr_backend <- function(name, callback) {
  stopifnot(is.character(name), length(name) == 1L)
  check_native_result(register_ocr_backend_native(name, callback))
}

#' @param name Character string naming the backend to remove.
#' @return Invisible NULL on success.
unregister_ocr_backend <- function(name) {
  stopifnot(is.character(name), length(name) == 1L)
  check_native_result(unregister_ocr_backend_native(name))
}

#' @return Character vector of registered backend names.
list_ocr_backends <- function() {
  check_native_result(list_ocr_backends_native())
}

#' @return Invisible NULL on success.
clear_ocr_backends <- function() {
  check_native_result(clear_ocr_backends_native())
}


#' @param name Character string naming the post-processor.
#' @param callback Callback object for post-processing.
#' @return Invisible NULL on success.
register_post_processor <- function(name, callback) {
  stopifnot(is.character(name), length(name) == 1L)
  check_native_result(register_post_processor_native(name, callback))
}

#' @param name Character string naming the post-processor to remove.
#' @return Invisible NULL on success.
unregister_post_processor <- function(name) {
  stopifnot(is.character(name), length(name) == 1L)
  check_native_result(unregister_post_processor_native(name))
}

#' @return Character vector of registered post-processor names.
list_post_processors <- function() {
  check_native_result(list_post_processors_native())
}

#' @return Invisible NULL on success.
clear_post_processors <- function() {
  check_native_result(clear_post_processors_native())
}


#' @param name Character string naming the validator.
#' @param callback Callback object for validation.
#' @return Invisible NULL on success.
register_validator <- function(name, callback) {
  stopifnot(is.character(name), length(name) == 1L)
  check_native_result(register_validator_native(name, callback))
}

#' @param name Character string naming the validator to remove.
#' @return Invisible NULL on success.
unregister_validator <- function(name) {
  stopifnot(is.character(name), length(name) == 1L)
  check_native_result(unregister_validator_native(name))
}

#' @return Character vector of registered validator names.
list_validators <- function() {
  check_native_result(list_validators_native())
}

#' @return Invisible NULL on success.
clear_validators <- function() {
  check_native_result(clear_validators_native())
}


#' @return Character vector of registered extractor names.
list_document_extractors <- function() {
  check_native_result(list_document_extractors_native())
}

#' @param name Character string naming the extractor to remove.
#' @return Invisible NULL on success.
unregister_document_extractor <- function(name) {
  stopifnot(is.character(name), length(name) == 1L)
  check_native_result(unregister_document_extractor_native(name))
}

#' @return Invisible NULL on success.
clear_document_extractors <- function() {
  check_native_result(clear_document_extractors_native())
}
