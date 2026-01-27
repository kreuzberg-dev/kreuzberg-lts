# frozen_string_literal: true

# Auto-generated tests for contract fixtures.

# rubocop:disable RSpec/DescribeClass, RSpec/ExampleLength, Metrics/BlockLength
require_relative 'spec_helper'

RSpec.describe 'contract fixtures' do
  it 'api_batch_bytes_async' do
    E2ERuby.run_fixture_with_method(
      'api_batch_bytes_async',
      'pdfs/fake_memo.pdf',
      nil,
      :batch_async,
      :bytes,
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_content_contains_any(result, ['May 5, 2023', 'Mallori'])
    end
  end

  it 'api_batch_bytes_sync' do
    E2ERuby.run_fixture_with_method(
      'api_batch_bytes_sync',
      'pdfs/fake_memo.pdf',
      nil,
      :batch_sync,
      :bytes,
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_content_contains_any(result, ['May 5, 2023', 'Mallori'])
    end
  end

  it 'api_batch_file_async' do
    E2ERuby.run_fixture_with_method(
      'api_batch_file_async',
      'pdfs/fake_memo.pdf',
      nil,
      :batch_async,
      :file,
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_content_contains_any(result, ['May 5, 2023', 'Mallori'])
    end
  end

  it 'api_batch_file_sync' do
    E2ERuby.run_fixture_with_method(
      'api_batch_file_sync',
      'pdfs/fake_memo.pdf',
      nil,
      :batch_sync,
      :file,
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_content_contains_any(result, ['May 5, 2023', 'Mallori'])
    end
  end

  it 'api_extract_bytes_async' do
    E2ERuby.run_fixture_with_method(
      'api_extract_bytes_async',
      'pdfs/fake_memo.pdf',
      nil,
      :async,
      :bytes,
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_content_contains_any(result, ['May 5, 2023', 'Mallori'])
    end
  end

  it 'api_extract_bytes_sync' do
    E2ERuby.run_fixture_with_method(
      'api_extract_bytes_sync',
      'pdfs/fake_memo.pdf',
      nil,
      :sync,
      :bytes,
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_content_contains_any(result, ['May 5, 2023', 'Mallori'])
    end
  end

  it 'api_extract_file_async' do
    E2ERuby.run_fixture_with_method(
      'api_extract_file_async',
      'pdfs/fake_memo.pdf',
      nil,
      :async,
      :file,
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_content_contains_any(result, ['May 5, 2023', 'Mallori'])
    end
  end

  it 'api_extract_file_sync' do
    E2ERuby.run_fixture(
      'api_extract_file_sync',
      'pdfs/fake_memo.pdf',
      nil,
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_content_contains_any(result, ['May 5, 2023', 'Mallori'])
    end
  end

  it 'config_chunking' do
    E2ERuby.run_fixture(
      'config_chunking',
      'pdfs/fake_memo.pdf',
      { chunking: { max_chars: 500, overlap: 50 } },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_chunks(result, min_count: 1, each_has_content: true)
    end
  end

  it 'config_force_ocr' do
    E2ERuby.run_fixture(
      'config_force_ocr',
      'pdfs/fake_memo.pdf',
      { force_ocr: true },
      requirements: %w[tesseract],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 5)
    end
  end

  it 'config_images' do
    E2ERuby.run_fixture(
      'config_images',
      'pdfs/embedded_images_tables.pdf',
      { images: { extract: true, format: 'png' } },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_images(result, min_count: 1)
    end
  end

  it 'config_language_detection' do
    E2ERuby.run_fixture(
      'config_language_detection',
      'pdfs/fake_memo.pdf',
      { language_detection: { enabled: true } },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
      E2ERuby::Assertions.assert_detected_languages(result, %w[eng], 0.5)
    end
  end

  it 'config_pages' do
    E2ERuby.run_fixture(
      'config_pages',
      'pdfs/multi_page.pdf',
      { pages: { end: 3, start: 1 } },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
    end
  end

  it 'config_use_cache_false' do
    E2ERuby.run_fixture(
      'config_use_cache_false',
      'pdfs/fake_memo.pdf',
      { use_cache: false },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
    end
  end

  it 'output_format_djot' do
    E2ERuby.run_fixture(
      'output_format_djot',
      'pdfs/fake_memo.pdf',
      { output_format: 'djot' },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
    end
  end

  it 'output_format_html' do
    E2ERuby.run_fixture(
      'output_format_html',
      'pdfs/fake_memo.pdf',
      { output_format: 'html' },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
    end
  end

  it 'output_format_markdown' do
    E2ERuby.run_fixture(
      'output_format_markdown',
      'pdfs/fake_memo.pdf',
      { output_format: 'markdown' },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
    end
  end

  it 'output_format_plain' do
    E2ERuby.run_fixture(
      'output_format_plain',
      'pdfs/fake_memo.pdf',
      { output_format: 'plain' },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
    end
  end

  it 'result_format_element_based' do
    E2ERuby.run_fixture(
      'result_format_element_based',
      'pdfs/fake_memo.pdf',
      { result_format: 'element_based' },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_elements(result, min_count: 1)
    end
  end

  it 'result_format_unified' do
    E2ERuby.run_fixture(
      'result_format_unified',
      'pdfs/fake_memo.pdf',
      { result_format: 'unified' },
      requirements: [],
      notes: nil,
      skip_if_missing: true
    ) do |result|
      E2ERuby::Assertions.assert_expected_mime(
        result,
        ['application/pdf']
      )
      E2ERuby::Assertions.assert_min_content_length(result, 10)
    end
  end
end
# rubocop:enable RSpec/DescribeClass, RSpec/ExampleLength, Metrics/BlockLength
