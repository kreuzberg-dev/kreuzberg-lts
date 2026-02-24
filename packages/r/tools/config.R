# Build configuration for kreuzberg R package
# Generates src/Makevars from src/Makevars.in

additional_libs <- ""

# Platform-specific libraries
if (.Platform$OS.type == "windows") {
  makevars_in <- "src/Makevars.win.in"
  makevars_out <- "src/Makevars.win"
} else {
  makevars_in <- "src/Makevars.in"
  makevars_out <- "src/Makevars"

  if (Sys.info()[["sysname"]] == "Linux") {
    additional_libs <- "-lpthread -ldl -lm"
  } else if (Sys.info()[["sysname"]] == "Darwin") {
    additional_libs <- "-framework Security -framework CoreFoundation"
  }
}

makevars_content <- readLines(makevars_in)
makevars_content <- gsub("@ADDITIONAL_PKG_LIBS@", additional_libs, makevars_content)
writeLines(makevars_content, makevars_out)
