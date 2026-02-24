test_that("extraction_config creates valid config", {
  config <- extraction_config()
  expect_true(is.list(config))

  config <- extraction_config(force_ocr = TRUE)
  expect_true(config$force_ocr)

  config <- extraction_config(
    ocr = ocr_config(backend = "tesseract", language = "deu"),
    chunking = chunking_config(max_characters = 500L)
  )
  expect_equal(config$ocr$backend, "tesseract")
  expect_equal(config$ocr$language, "deu")
  expect_equal(config$chunking$max_characters, 500L)
})

test_that("ocr_config creates valid config", {
  config <- ocr_config()
  expect_equal(config$backend, "tesseract")
  expect_equal(config$language, "eng")

  config <- ocr_config(backend = "paddle-ocr", language = "chi_sim", dpi = 300L)
  expect_equal(config$backend, "paddle-ocr")
  expect_equal(config$dpi, 300L)
})

test_that("chunking_config creates valid config", {
  config <- chunking_config()
  expect_equal(config$max_characters, 1000L)
  expect_equal(config$overlap, 200L)

  config <- chunking_config(max_characters = 2000L, overlap = 100L)
  expect_equal(config$max_characters, 2000L)
  expect_equal(config$overlap, 100L)
})

test_that("extraction_config serializes to JSON", {
  config <- extraction_config(force_ocr = TRUE, output_format = "markdown")
  json <- jsonlite::toJSON(config, auto_unbox = TRUE)
  parsed <- jsonlite::fromJSON(json)
  expect_true(parsed$force_ocr)
  expect_equal(parsed$output_format, "markdown")
})
