test_that("config builders accept extra arguments", {
  config <- extraction_config(custom_field = "value")
  expect_equal(config$custom_field, "value")

  config <- ocr_config(custom_option = TRUE)
  expect_true(config$custom_option)

  config <- chunking_config(strategy = "semantic")
  expect_equal(config$strategy, "semantic")
})

test_that("config defaults are correct", {
  ocr <- ocr_config()
  expect_equal(ocr$backend, "tesseract")
  expect_equal(ocr$language, "eng")

  chunking <- chunking_config()
  expect_equal(chunking$max_characters, 1000L)
  expect_equal(chunking$overlap, 200L)
})
