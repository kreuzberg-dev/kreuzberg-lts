from __future__ import annotations

from unittest.mock import Mock, patch

import pytest

from kreuzberg._entity_extraction import (
    _load_spacy_model,
    _select_spacy_model,
    extract_entities,
    extract_keywords,
)
from kreuzberg._types import SpacyEntityExtractionConfig
from kreuzberg.exceptions import MissingDependencyError


def test_extract_entities_with_custom_patterns_only() -> None:
    text = "Contact john@example.com or call 555-1234 for more info."

    custom_patterns = frozenset(
        [
            ("EMAIL", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"),
            ("PHONE", r"\b\d{3}-\d{4}\b"),
        ]
    )

    with patch("kreuzberg._entity_extraction._select_spacy_model", return_value=None):
        entities = extract_entities(text, custom_patterns=custom_patterns)

    assert len(entities) == 2

    email_entity = next((e for e in entities if e.type == "EMAIL"), None)
    assert email_entity is not None
    assert email_entity.text == "john@example.com"
    assert email_entity.start == 8
    assert email_entity.end == 24

    phone_entity = next((e for e in entities if e.type == "PHONE"), None)
    assert phone_entity is not None
    assert phone_entity.text == "555-1234"
    assert phone_entity.start == 33
    assert phone_entity.end == 41


def test_extract_entities_no_spacy_model() -> None:
    text = "John Smith works at Google."

    with (
        patch("kreuzberg._entity_extraction._select_spacy_model", return_value=None),
    ):
        entities = extract_entities(text)

    assert entities == []


def test_extract_entities_spacy_load_fails() -> None:
    text = "John Smith works at Google."

    with (
        patch("kreuzberg._entity_extraction._select_spacy_model", return_value="en_core_web_sm"),
        patch("kreuzberg._entity_extraction._load_spacy_model", return_value=None),
    ):
        entities = extract_entities(text)

    assert entities == []


def test_extract_entities_with_spacy_success() -> None:
    text = "John Smith works at Google in New York."

    mock_nlp = Mock()
    mock_doc = Mock()

    mock_person = Mock()
    mock_person.label_ = "PERSON"
    mock_person.text = "John Smith"
    mock_person.start_char = 0
    mock_person.end_char = 10

    mock_org = Mock()
    mock_org.label_ = "ORG"
    mock_org.text = "Google"
    mock_org.start_char = 20
    mock_org.end_char = 26

    mock_location = Mock()
    mock_location.label_ = "LOCATION"
    mock_location.text = "New York"
    mock_location.start_char = 30
    mock_location.end_char = 38

    mock_doc.ents = [mock_person, mock_org, mock_location]
    mock_nlp.return_value = mock_doc

    with (
        patch("kreuzberg._entity_extraction._select_spacy_model", return_value="en_core_web_sm"),
        patch("kreuzberg._entity_extraction._load_spacy_model", return_value=mock_nlp),
    ):
        entities = extract_entities(text)

    assert len(entities) == 2

    person_entity = entities[0]
    assert person_entity.type == "PERSON"
    assert person_entity.text == "John Smith"

    location_entity = entities[1]
    assert location_entity.type == "LOCATION"
    assert location_entity.text == "New York"


def test_extract_entities_text_truncation() -> None:
    long_text = "A" * 2000000
    config = SpacyEntityExtractionConfig(max_doc_length=1000000)

    mock_nlp = Mock()
    mock_doc = Mock()
    mock_doc.ents = []
    mock_nlp.return_value = mock_doc

    with (
        patch("kreuzberg._entity_extraction._select_spacy_model", return_value="en_core_web_sm"),
        patch("kreuzberg._entity_extraction._load_spacy_model", return_value=mock_nlp),
    ):
        extract_entities(long_text, spacy_config=config)

    called_text = mock_nlp.call_args[0][0]
    assert len(called_text) == 1000000


def test_extract_entities_mixed_patterns_and_spacy() -> None:
    text = "Contact john@example.com. John Smith works at Google."

    custom_patterns = frozenset(
        [
            ("EMAIL", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"),
        ]
    )

    mock_nlp = Mock()
    mock_doc = Mock()

    mock_person = Mock()
    mock_person.label_ = "PERSON"
    mock_person.text = "John Smith"
    mock_person.start_char = 26
    mock_person.end_char = 36

    mock_doc.ents = [mock_person]
    mock_nlp.return_value = mock_doc

    with (
        patch("kreuzberg._entity_extraction._select_spacy_model", return_value="en_core_web_sm"),
        patch("kreuzberg._entity_extraction._load_spacy_model", return_value=mock_nlp),
    ):
        entities = extract_entities(text, custom_patterns=custom_patterns)

    assert len(entities) == 2

    email_entity = next((e for e in entities if e.type == "EMAIL"), None)
    assert email_entity is not None
    assert email_entity.text == "john@example.com"

    person_entity = next((e for e in entities if e.type == "PERSON"), None)
    assert person_entity is not None
    assert person_entity.text == "John Smith"


def test_extract_entities_missing_spacy() -> None:
    text = "Test text"

    with patch.dict("sys.modules", {"spacy": None}):
        with pytest.raises(MissingDependencyError, match="spacy"):
            extract_entities(text)


def test_load_spacy_model_success() -> None:
    config = SpacyEntityExtractionConfig(max_doc_length=500000)

    mock_spacy = Mock()
    mock_nlp = Mock()
    mock_spacy.load.return_value = mock_nlp

    with patch.dict("sys.modules", {"spacy": mock_spacy}):
        _load_spacy_model.cache_clear()
        result = _load_spacy_model("en_core_web_sm", config)

    assert result == mock_nlp
    assert mock_nlp.max_length == 500000
    mock_spacy.load.assert_called_once_with("en_core_web_sm")


def test_load_spacy_model_with_cache_dir() -> None:
    import os

    config = SpacyEntityExtractionConfig(max_doc_length=500000, model_cache_dir="/custom/cache")

    mock_spacy = Mock()
    mock_nlp = Mock()
    mock_spacy.load.return_value = mock_nlp

    original_env = os.environ.copy()

    with (
        patch.dict("sys.modules", {"spacy": mock_spacy}),
        patch.dict(os.environ, {}, clear=True),
    ):
        _load_spacy_model.cache_clear()
        result = _load_spacy_model("en_core_web_sm", config)

        assert os.environ.get("SPACY_DATA") == "/custom/cache"

    assert result == mock_nlp

    os.environ.clear()
    os.environ.update(original_env)


def test_load_spacy_model_failure() -> None:
    config = SpacyEntityExtractionConfig()

    mock_spacy = Mock()
    mock_spacy.load.side_effect = OSError("Model not found")

    with patch.dict("sys.modules", {"spacy": mock_spacy}):
        _load_spacy_model.cache_clear()
        result = _load_spacy_model("en_core_web_sm", config)

    assert result is None


def test_load_spacy_model_import_error() -> None:
    config = SpacyEntityExtractionConfig()

    mock_spacy = Mock()
    mock_spacy.load.side_effect = ImportError("Module not found")

    with patch.dict("sys.modules", {"spacy": mock_spacy}):
        _load_spacy_model.cache_clear()
        result = _load_spacy_model("en_core_web_sm", config)

    assert result is None


def test_select_spacy_model_no_languages() -> None:
    config = SpacyEntityExtractionConfig()

    result = _select_spacy_model(None, config)

    assert result == "en_core_web_sm"


def test_select_spacy_model_with_languages() -> None:
    config = SpacyEntityExtractionConfig()

    result = _select_spacy_model(["de"], config)
    assert result == "de_core_news_sm"

    result = _select_spacy_model(["ko", "fr", "en"], config)
    assert result == "ko_core_news_sm"

    result = _select_spacy_model(["xyz"], config)
    assert result == "xx_ent_wiki_sm"


def test_extract_keywords_success() -> None:
    text = "Python is a programming language. Python is widely used for data science."

    mock_keybert_class = Mock()
    mock_model = Mock()
    mock_model.extract_keywords.return_value = [
        ("python", 0.8),
        ("programming", 0.6),
        ("data science", 0.5),
    ]
    mock_keybert_class.return_value = mock_model

    mock_keybert_module = Mock()
    mock_keybert_module.KeyBERT = mock_keybert_class

    with patch.dict("sys.modules", {"keybert": mock_keybert_module}):
        keywords = extract_keywords(text, keyword_count=3)

    assert len(keywords) == 3
    assert keywords[0] == ("python", 0.8)
    assert keywords[1] == ("programming", 0.6)
    assert keywords[2] == ("data science", 0.5)

    mock_model.extract_keywords.assert_called_once_with(text, top_n=3)


def test_extract_keywords_runtime_error() -> None:
    text = "Test text"

    mock_keybert_class = Mock()
    mock_model = Mock()
    mock_model.extract_keywords.side_effect = RuntimeError("Model error")
    mock_keybert_class.return_value = mock_model

    mock_keybert_module = Mock()
    mock_keybert_module.KeyBERT = mock_keybert_class

    with patch.dict("sys.modules", {"keybert": mock_keybert_module}):
        keywords = extract_keywords(text)

    assert keywords == []


def test_extract_keywords_os_error() -> None:
    text = "Test text"

    mock_keybert_class = Mock()
    mock_model = Mock()
    mock_model.extract_keywords.side_effect = OSError("File not found")
    mock_keybert_class.return_value = mock_model

    mock_keybert_module = Mock()
    mock_keybert_module.KeyBERT = mock_keybert_class

    with patch.dict("sys.modules", {"keybert": mock_keybert_module}):
        keywords = extract_keywords(text)

    assert keywords == []


def test_extract_keywords_value_error() -> None:
    text = "Test text"

    mock_keybert_class = Mock()
    mock_model = Mock()
    mock_model.extract_keywords.side_effect = ValueError("Invalid value")
    mock_keybert_class.return_value = mock_model

    mock_keybert_module = Mock()
    mock_keybert_module.KeyBERT = mock_keybert_class

    with patch.dict("sys.modules", {"keybert": mock_keybert_module}):
        keywords = extract_keywords(text)

    assert keywords == []


def test_extract_keywords_missing_keybert() -> None:
    text = "Test text"

    with patch.dict("sys.modules", {"keybert": None}):
        with pytest.raises(MissingDependencyError, match="keybert"):
            extract_keywords(text)
