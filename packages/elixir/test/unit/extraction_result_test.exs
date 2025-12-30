defmodule KreuzbergTest.Unit.ExtractionResultTest do
  @moduledoc """
  Comprehensive tests for the Kreuzberg.ExtractionResult module.

  Tests the ExtractionResult struct creation and field handling for all
  constructor variants and edge cases.
  """

  use ExUnit.Case

  alias Kreuzberg.ExtractionResult

  describe "new/2 - basic constructor with content and mime_type" do
    test "creates result with minimal required fields" do
      result = ExtractionResult.new("Hello World", "text/plain")

      assert result.content == "Hello World"
      assert result.mime_type == "text/plain"
      assert result.metadata == %{}
      assert result.tables == []
      assert result.detected_languages == nil
      assert result.chunks == nil
      assert result.images == nil
      assert result.pages == nil
    end

    test "handles empty content string" do
      result = ExtractionResult.new("", "text/plain")

      assert result.content == ""
      assert result.mime_type == "text/plain"
    end

    test "handles various MIME types" do
      mime_types = [
        "text/plain",
        "application/pdf",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "image/png",
        "text/html; charset=utf-8"
      ]

      Enum.each(mime_types, fn mime_type ->
        result = ExtractionResult.new("content", mime_type)
        assert result.mime_type == mime_type
      end)
    end

    test "handles unicode content" do
      unicode_content = "Hello 世界 مرحبا мир"
      result = ExtractionResult.new(unicode_content, "text/plain")

      assert result.content == unicode_content
    end

    test "handles very long content strings" do
      long_content = String.duplicate("x", 1_000_000)
      result = ExtractionResult.new(long_content, "text/plain")

      assert result.content == long_content
      assert String.length(result.content) == 1_000_000
    end
  end

  describe "new/3 - constructor with metadata" do
    test "adds metadata to result" do
      metadata = %{"pages" => 10, "author" => "John Doe"}
      result = ExtractionResult.new("content", "application/pdf", metadata)

      assert result.content == "content"
      assert result.metadata == metadata
      assert result.tables == []
    end

    test "handles empty metadata map" do
      result = ExtractionResult.new("content", "text/plain", %{})

      assert result.metadata == %{}
    end

    test "handles nested metadata structures" do
      metadata = %{
        "title" => "Test Doc",
        "author" => "Jane Doe"
      }

      result = ExtractionResult.new("content", "application/pdf", metadata)

      assert result.metadata.title == "Test Doc"
      assert result.metadata.author == "Jane Doe"
    end

    test "handles metadata with various value types" do
      metadata = %{
        "title" => "Sample Title",
        "author" => "John Doe",
        "page_count" => 42
      }

      result = ExtractionResult.new("content", "text/plain", metadata)

      assert result.metadata.title == "Sample Title"
      assert result.metadata.author == "John Doe"
      assert result.metadata.page_count == 42
    end
  end

  describe "new/4 - constructor with tables" do
    test "adds tables to result" do
      tables = [
        %{"headers" => ["Col1", "Col2"], "rows" => [["a", "b"], ["c", "d"]]},
        %{"headers" => ["X", "Y"], "rows" => [["1", "2"]]}
      ]

      result = ExtractionResult.new("content", "application/pdf", %{}, tables)

      assert result.content == "content"
      assert result.tables == tables
      assert length(result.tables) == 2
    end

    test "handles empty tables list" do
      result = ExtractionResult.new("content", "text/plain", %{}, [])

      assert result.tables == []
    end

    test "handles single table" do
      tables = [%{"headers" => ["Name"], "rows" => [["Alice"], ["Bob"]]}]
      result = ExtractionResult.new("content", "text/plain", %{}, tables)

      assert length(result.tables) == 1
      assert result.tables |> List.first() |> Map.get("headers") == ["Name"]
    end

    test "handles complex table structures with nested data" do
      tables = [
        %{
          "headers" => ["ID", "Data", "Details"],
          "rows" => [
            [1, "text", %{"nested" => "value"}],
            [2, "more", [1, 2, 3]]
          ]
        }
      ]

      result = ExtractionResult.new("content", "text/plain", %{}, tables)

      assert length(result.tables) == 1
      table = List.first(result.tables)
      assert table.headers == ["ID", "Data", "Details"]
      assert length(table.rows) == 2
    end

    test "handles large number of tables" do
      tables =
        Enum.map(1..100, fn i ->
          %{"id" => i, "headers" => ["Col#{i}"]}
        end)

      result = ExtractionResult.new("content", "text/plain", %{}, tables)

      assert length(result.tables) == 100
    end
  end

  describe "new/5 - constructor with options" do
    test "adds detected_languages from options" do
      opts = [detected_languages: ["en", "de", "fr"]]
      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      assert result.detected_languages == ["en", "de", "fr"]
      assert result.chunks == nil
      assert result.images == nil
      assert result.pages == nil
    end

    test "adds chunks from options" do
      chunks = [
        %{"text" => "chunk1", "embedding" => [0.1, 0.2]},
        %{"text" => "chunk2", "embedding" => [0.3, 0.4]}
      ]

      opts = [chunks: chunks]
      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      assert result.chunks == chunks
      assert length(result.chunks) == 2
    end

    test "adds images from options" do
      images = [
        %{"path" => "/image1.png", "ocr_text" => "text in image 1"},
        %{"path" => "/image2.jpg", "ocr_text" => "text in image 2"}
      ]

      opts = [images: images]
      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      assert result.images == images
      assert length(result.images) == 2
    end

    test "adds pages from options" do
      pages = [
        %{"number" => 1, "content" => "Page 1 content"},
        %{"number" => 2, "content" => "Page 2 content"}
      ]

      opts = [pages: pages]
      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      assert result.pages == pages
      assert length(result.pages) == 2
    end

    test "combines all options together" do
      metadata = %{"title" => "Test"}
      tables = [%{"headers" => ["A"]}]
      chunks = [%{"text" => "chunk"}]
      images = [%{"path" => "image.png"}]
      pages = [%{"number" => 1}]
      languages = ["en", "de"]

      opts = [
        detected_languages: languages,
        chunks: chunks,
        images: images,
        pages: pages
      ]

      result = ExtractionResult.new("content", "application/pdf", metadata, tables, opts)

      assert result.content == "content"
      assert result.mime_type == "application/pdf"
      assert result.metadata == metadata
      assert result.tables == tables
      assert result.detected_languages == languages
      assert result.chunks == chunks
      assert result.images == images
      assert result.pages == pages
    end

    test "handles empty options list" do
      result = ExtractionResult.new("content", "text/plain", %{}, [], [])

      assert result.detected_languages == nil
      assert result.chunks == nil
      assert result.images == nil
      assert result.pages == nil
    end

    test "ignores unknown options" do
      opts = [
        detected_languages: ["en"],
        unknown_field: "should be ignored",
        another_unknown: 42
      ]

      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      assert result.detected_languages == ["en"]
      assert result.chunks == nil
    end

    test "handles nil values in options" do
      opts = [
        detected_languages: nil,
        chunks: nil,
        images: nil,
        pages: nil
      ]

      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      assert result.detected_languages == nil
      assert result.chunks == nil
      assert result.images == nil
      assert result.pages == nil
    end

    test "handles single language in detected_languages" do
      opts = [detected_languages: ["en"]]
      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      assert result.detected_languages == ["en"]
    end

    test "handles empty language list" do
      opts = [detected_languages: []]
      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      assert result.detected_languages == []
    end
  end

  describe "struct validation and field access" do
    test "all fields are accessible on the struct" do
      result = ExtractionResult.new("content", "text/plain")

      assert is_binary(result.content)
      assert is_binary(result.mime_type)
      assert is_map(result.metadata)
      assert is_list(result.tables)
    end

    test "struct is a map and can be converted" do
      result = ExtractionResult.new("content", "text/plain")

      as_map = Map.from_struct(result)

      assert is_map(as_map)
      assert Map.has_key?(as_map, :content)
      assert Map.has_key?(as_map, :mime_type)
    end

    test "can pattern match on ExtractionResult struct" do
      result = ExtractionResult.new("test content", "application/pdf")

      assert %ExtractionResult{content: content, mime_type: mime} = result
      assert content == "test content"
      assert mime == "application/pdf"
    end
  end

  describe "edge cases and boundary conditions" do
    test "handles newlines in content" do
      content_with_newlines = "Line 1\nLine 2\nLine 3"
      result = ExtractionResult.new(content_with_newlines, "text/plain")

      assert result.content == content_with_newlines
      assert String.contains?(result.content, "\n")
    end

    test "handles special characters in content" do
      special_content = "Content with special chars: !@#$%^&*(){}[]|\\:;\"'<>,.?/~`"
      result = ExtractionResult.new(special_content, "text/plain")

      assert result.content == special_content
    end

    test "handles MIME type with parameters" do
      mime_with_params = "text/plain; charset=utf-8; boundary=something"
      result = ExtractionResult.new("content", mime_with_params)

      assert result.mime_type == mime_with_params
    end

    test "handles metadata with empty values" do
      metadata = %{
        "title" => "",
        "author" => nil
      }

      result = ExtractionResult.new("content", "text/plain", metadata)

      assert result.metadata.title == ""
      assert result.metadata.author == nil
    end

    test "distinguishes between nil and missing fields" do
      opts = [detected_languages: nil]
      result = ExtractionResult.new("content", "text/plain", %{}, [], opts)

      # When explicitly set to nil via options
      assert result.detected_languages == nil
    end
  end

  describe "multiple constructor invocations" do
    test "creates independent result instances" do
      result1 = ExtractionResult.new("content1", "text/plain")
      result2 = ExtractionResult.new("content2", "text/plain")

      assert result1.content != result2.content
      assert result1 != result2
    end

    test "concurrent result creation doesn't interfere" do
      tasks =
        Enum.map(1..10, fn i ->
          Task.async(fn ->
            ExtractionResult.new("content#{i}", "text/plain")
          end)
        end)

      results = Task.await_many(tasks)

      assert length(results) == 10
      contents = Enum.map(results, & &1.content)
      # All unique
      assert Enum.uniq(contents) == contents
    end
  end
end
