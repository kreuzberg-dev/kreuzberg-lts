```elixir title="Elixir"
alias Kreuzberg.ExtractionConfig

config = %ExtractionConfig{
  ocr: %{"enabled" => true, "backend" => "paddle-ocr", "language" => "en"}
  # Add "model_tier" => "server" for max accuracy
}

{:ok, result} = Kreuzberg.extract_file("scanned_document.pdf", nil, config)

IO.puts("OCR Extracted content:")
IO.puts(result.content)
```
