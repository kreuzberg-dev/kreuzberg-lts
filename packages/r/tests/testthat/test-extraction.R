test_that("extract_file_sync extracts text from a file", {
  # Create a temporary text file
  tmp <- tempfile(fileext = ".txt")
  writeLines("Hello, kreuzberg!", tmp)
  on.exit(unlink(tmp))

  result <- extract_file_sync(tmp)
  expect_s3_class(result, "kreuzberg_result")
  expect_true(nchar(result$content) > 0)
  expect_true(grepl("Hello", result$content, fixed = TRUE))
})

test_that("extract_file extracts text from a file", {
  tmp <- tempfile(fileext = ".txt")
  writeLines("Async extraction test", tmp)
  on.exit(unlink(tmp))

  result <- extract_file(tmp)
  expect_s3_class(result, "kreuzberg_result")
  expect_true(grepl("Async", result$content, fixed = TRUE))
})

test_that("extract_bytes_sync extracts from raw bytes", {
  bytes <- charToRaw("Test content in bytes")
  result <- extract_bytes_sync(bytes, "text/plain")
  expect_s3_class(result, "kreuzberg_result")
  expect_true(nchar(result$content) > 0)
})

test_that("extract_bytes extracts from raw bytes", {
  bytes <- charToRaw("Async bytes test")
  result <- extract_bytes(bytes, "text/plain")
  expect_s3_class(result, "kreuzberg_result")
  expect_true(nchar(result$content) > 0)
})

test_that("extract_file_sync with config works", {
  tmp <- tempfile(fileext = ".txt")
  writeLines("Config test", tmp)
  on.exit(unlink(tmp))

  config <- extraction_config(output_format = "text")
  result <- extract_file_sync(tmp, config = config)
  expect_s3_class(result, "kreuzberg_result")
})

test_that("kreuzberg_result print method works", {
  tmp <- tempfile(fileext = ".txt")
  writeLines("Print test content", tmp)
  on.exit(unlink(tmp))

  result <- extract_file_sync(tmp)
  expect_output(print(result), "kreuzberg_result")
})
