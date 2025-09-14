from __future__ import annotations

from typing import TYPE_CHECKING
from unittest.mock import Mock, patch

import pytest

from kreuzberg._language_detection import (
    _create_fast_langdetect_config,
    detect_languages,
)
from kreuzberg._types import LanguageDetectionConfig
from kreuzberg.exceptions import MissingDependencyError

if TYPE_CHECKING:
    from collections.abc import Generator


@pytest.fixture(autouse=True)
def clear_language_detection_cache() -> Generator[None, None, None]:
    """Clear the detect_languages LRU cache before each test."""
    detect_languages.cache_clear()
    yield
    detect_languages.cache_clear()


def test_create_fast_langdetect_config_when_library_missing() -> None:
    """Test _create_fast_langdetect_config when fast-langdetect is missing."""
    config = LanguageDetectionConfig()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", False),
        patch("kreuzberg._language_detection.FastLangDetectConfig", None),
    ):
        result = _create_fast_langdetect_config(config)

    assert result is None


def test_create_fast_langdetect_config_with_basic_config() -> None:
    """Test _create_fast_langdetect_config with basic configuration."""
    config = LanguageDetectionConfig(allow_fallback=True)

    mock_fast_config_class = Mock()
    mock_instance = Mock()
    mock_fast_config_class.return_value = mock_instance

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.FastLangDetectConfig", mock_fast_config_class),
    ):
        result = _create_fast_langdetect_config(config)

    assert result == mock_instance
    mock_fast_config_class.assert_called_once_with(allow_fallback=True)


def test_create_fast_langdetect_config_with_cache_dir() -> None:
    """Test _create_fast_langdetect_config with cache directory specified."""
    config = LanguageDetectionConfig(allow_fallback=False, cache_dir="/tmp/langdetect")

    mock_fast_config_class = Mock()
    mock_instance = Mock()
    mock_fast_config_class.return_value = mock_instance

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.FastLangDetectConfig", mock_fast_config_class),
    ):
        result = _create_fast_langdetect_config(config)

    assert result == mock_instance
    mock_fast_config_class.assert_called_once_with(allow_fallback=False, cache_dir="/tmp/langdetect")


def test_create_fast_langdetect_config_without_cache_dir() -> None:
    """Test _create_fast_langdetect_config with cache_dir=None."""
    config = LanguageDetectionConfig(allow_fallback=True, cache_dir=None)

    mock_fast_config_class = Mock()
    mock_instance = Mock()
    mock_fast_config_class.return_value = mock_instance

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.FastLangDetectConfig", mock_fast_config_class),
    ):
        result = _create_fast_langdetect_config(config)

    assert result == mock_instance
    # Should not include cache_dir in kwargs when it's None
    mock_fast_config_class.assert_called_once_with(allow_fallback=True)


def test_detect_languages_when_library_missing() -> None:
    """Test detect_languages raises MissingDependencyError when fast-langdetect is missing."""
    text = "This is some English text."

    with patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", False):
        with pytest.raises(MissingDependencyError) as exc_info:
            detect_languages(text)

        error = exc_info.value
        assert "fast-langdetect" in str(error)
        assert "language detection" in str(error)


def test_detect_languages_when_detect_function_missing() -> None:
    """Test detect_languages raises MissingDependencyError when detect function is None."""
    text = "This is some English text."

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", None),
    ):
        with pytest.raises(MissingDependencyError) as exc_info:
            detect_languages(text)

        error = exc_info.value
        assert "fast-langdetect" in str(error)
        assert "language detection" in str(error)


def test_detect_languages_when_detect_multilingual_function_missing() -> None:
    """Test detect_languages raises MissingDependencyError when detect_multilingual is None."""
    text = "This is some English text."

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", Mock()),
        patch("kreuzberg._language_detection.detect_multilingual", None),
    ):
        with pytest.raises(MissingDependencyError) as exc_info:
            detect_languages(text)

        error = exc_info.value
        assert "fast-langdetect" in str(error)
        assert "language detection" in str(error)


def test_detect_languages_single_language_success() -> None:
    """Test detect_languages with single language detection success."""
    text = "This is some English text."
    mock_detect_result = {"lang": "EN", "score": 0.99}

    mock_detect = Mock(return_value=mock_detect_result)
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text)

    assert result == ["en"]  # Should be lowercased
    mock_detect.assert_called_once_with(text, low_memory=True)
    mock_detect_multilingual.assert_not_called()


def test_detect_languages_single_language_no_lang_key() -> None:
    """Test detect_languages with single language detection but no lang key."""
    text = "This is some text."
    mock_detect_result = {"score": 0.50}  # No 'lang' key

    mock_detect = Mock(return_value=mock_detect_result)
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text)

    assert result is None
    mock_detect.assert_called_once_with(text, low_memory=True)


def test_detect_languages_single_language_empty_lang() -> None:
    """Test detect_languages with single language detection but empty lang."""
    text = "This is some text."
    mock_detect_result = {"lang": "", "score": 0.50}  # Empty lang

    mock_detect = Mock(return_value=mock_detect_result)
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text)

    assert result is None
    mock_detect.assert_called_once_with(text, low_memory=True)


def test_detect_languages_single_language_none_result() -> None:
    """Test detect_languages with single language detection returning None."""
    text = "This is some text."
    mock_detect = Mock(return_value=None)
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text)

    assert result is None
    mock_detect.assert_called_once_with(text, low_memory=True)


def test_detect_languages_multilingual_success() -> None:
    """Test detect_languages with multilingual detection success."""
    text = "Hello world. Bonjour le monde."
    config = LanguageDetectionConfig(multilingual=True, top_k=3)

    mock_multilingual_results = [
        {"lang": "EN", "score": 0.8},
        {"lang": "FR", "score": 0.7},
        {"lang": "ES", "score": 0.1},
    ]

    mock_detect = Mock()
    mock_detect_multilingual = Mock(return_value=mock_multilingual_results)

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text, config)

    assert result == ["en", "fr", "es"]  # Should be lowercased
    mock_detect_multilingual.assert_called_once_with(text, low_memory=True, k=3)
    mock_detect.assert_not_called()


def test_detect_languages_multilingual_with_low_memory() -> None:
    """Test detect_languages with multilingual detection and low memory mode."""
    text = "Hello world. Bonjour le monde."
    config = LanguageDetectionConfig(multilingual=True, low_memory=True, top_k=2)

    mock_multilingual_results = [{"lang": "EN", "score": 0.8}, {"lang": "FR", "score": 0.7}]

    mock_detect = Mock()
    mock_detect_multilingual = Mock(return_value=mock_multilingual_results)

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text, config)

    assert result == ["en", "fr"]
    mock_detect_multilingual.assert_called_once_with(text, low_memory=True, k=2)


def test_detect_languages_multilingual_results_missing_lang() -> None:
    """Test detect_languages with multilingual results that have missing lang keys."""
    text = "Mixed language text."
    config = LanguageDetectionConfig(multilingual=True)

    mock_multilingual_results = [
        {"lang": "EN", "score": 0.8},
        {"score": 0.6},  # No lang key
        {"lang": "", "score": 0.4},  # Empty lang
        {"lang": "FR", "score": 0.3},
    ]

    mock_detect = Mock()
    mock_detect_multilingual = Mock(return_value=mock_multilingual_results)

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text, config)

    # Should only include results with valid lang values
    assert result == ["en", "fr"]


def test_detect_languages_with_default_config() -> None:
    """Test detect_languages with default config (config=None)."""
    text = "This is English text."
    mock_detect_result = {"lang": "EN", "score": 0.95}

    mock_detect = Mock(return_value=mock_detect_result)
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text, config=None)

    assert result == ["en"]
    # Should use default config values
    mock_detect.assert_called_once_with(text, low_memory=True)


def test_detect_languages_with_low_memory_single_language() -> None:
    """Test detect_languages with low_memory=True for single language detection."""
    text = "This is English text."
    config = LanguageDetectionConfig(low_memory=True)
    mock_detect_result = {"lang": "EN", "score": 0.95}

    mock_detect = Mock(return_value=mock_detect_result)
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text, config)

    assert result == ["en"]
    mock_detect.assert_called_once_with(text, low_memory=True)


def test_detect_languages_exception_handling() -> None:
    """Test detect_languages handles exceptions gracefully."""
    text = "This is some text."

    mock_detect = Mock(side_effect=RuntimeError("Detection failed"))
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text)

    assert result is None
    mock_detect.assert_called_once_with(text, low_memory=True)


def test_detect_languages_multilingual_exception_handling() -> None:
    """Test detect_languages handles exceptions in multilingual mode gracefully."""
    text = "Mixed language text."
    config = LanguageDetectionConfig(multilingual=True)

    mock_detect = Mock()
    mock_detect_multilingual = Mock(side_effect=ValueError("Multilingual detection failed"))

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        result = detect_languages(text, config)

    assert result is None
    mock_detect_multilingual.assert_called_once_with(text, low_memory=True, k=3)


def test_detect_languages_caching_behavior() -> None:
    """Test detect_languages uses LRU cache correctly."""
    text = "This is English text."
    mock_detect_result = {"lang": "EN", "score": 0.95}

    mock_detect = Mock(return_value=mock_detect_result)
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        # Call twice with same text and config
        config = LanguageDetectionConfig()
        result1 = detect_languages(text, config)
        result2 = detect_languages(text, config)

    assert result1 == ["en"]
    assert result2 == ["en"]
    # Should only be called once due to caching
    mock_detect.assert_called_once_with(text, low_memory=True)


def test_detect_languages_cache_different_configs() -> None:
    """Test detect_languages caches differently for different configs."""
    text = "This is English text."
    mock_detect_result = {"lang": "EN", "score": 0.95}

    mock_detect = Mock(return_value=mock_detect_result)
    mock_detect_multilingual = Mock()

    with (
        patch("kreuzberg._language_detection.HAS_FAST_LANGDETECT", True),
        patch("kreuzberg._language_detection.detect", mock_detect),
        patch("kreuzberg._language_detection.detect_multilingual", mock_detect_multilingual),
    ):
        # Call with different configs
        config1 = LanguageDetectionConfig(low_memory=True)
        config2 = LanguageDetectionConfig(low_memory=False)

        result1 = detect_languages(text, config1)
        result2 = detect_languages(text, config2)

    assert result1 == ["en"]
    assert result2 == ["en"]
    # Should be called twice due to different configs (different cache keys)
    assert mock_detect.call_count == 2
    mock_detect.assert_any_call(text, low_memory=True)
    mock_detect.assert_any_call(text, low_memory=False)
