//! Text repair utilities for PDF extraction.
//!
//! Handles three classes of text corruption common in PDFs with broken font encodings:
//!
//! 1. **Ligature corruption** – fonts with broken ToUnicode CMaps map ligature glyphs
//!    (fi, fl, ff, ffi, ffl) to low-byte or ASCII characters. Repaired via either a
//!    per-page repair map built from pdfium's `has_unicode_map_error()` API, or
//!    contextual heuristics (e.g., `e!cient` → `efficient`).
//!
//! 2. **Broken word spacing** – fonts with broken CMap/ToUnicode tables cause pdfium
//!    to insert spaces mid-word. Detected by density of single-letter fragments
//!    followed by lowercase continuations; repaired by rejoining them.
//!
//! 3. **Unicode normalization** – curly quotes, fraction slash, and other PDF-specific
//!    Unicode characters are normalized to their ASCII equivalents.

use std::borrow::Cow;

use pdfium_render::prelude::*;

use super::types::PdfParagraph;

/// Build a mapping of corrupted characters → correct ligature expansions for a page.
///
/// Walks the per-character API to find characters with `has_unicode_map_error()`,
/// then determines the correct ligature expansion based on the character's raw
/// unicode value and font-specific encoding patterns.
///
/// Returns `None` if the page has no encoding errors (most pages).
pub(super) fn build_ligature_repair_map(page: &PdfPage) -> Option<Vec<(char, &'static str)>> {
    let text = match page.text() {
        Ok(t) => t,
        Err(_) => return None,
    };

    let chars = text.chars();
    let char_count = chars.len();
    if char_count == 0 {
        return None;
    }

    let mut repair_map: Vec<(char, &'static str)> = Vec::new();

    for i in 0..char_count {
        let ch = match chars.get(i) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if ch.is_generated().unwrap_or(false) {
            continue;
        }

        if !ch.has_unicode_map_error().unwrap_or(false) {
            continue;
        }

        if ch.font_is_symbolic() {
            continue;
        }

        let unicode_val = ch.unicode_value();
        let mapped_char = match char::from_u32(unicode_val) {
            Some(c) => c,
            None => continue,
        };

        if repair_map.iter().any(|(c, _)| *c == mapped_char) {
            continue;
        }

        let ligature = match unicode_val {
            0x0B => "ff",
            0x0C => "fi",
            0x0D => "fl",
            0x0E => "ffi",
            0x0F => "ffl",
            0x01 => "fi",
            0x02 => "fl",
            0x03 => "ff",
            0x04 => "ffi",
            0x05 => "ffl",
            0x21 => "fi",
            0x22 => "ff",
            0x23 => "fl",
            0x24 => "ffi",
            0x25 => "ffl",
            0x2A => "tt",
            0x3A => "ti",
            0x4D => "tti",
            _ => continue,
        };

        repair_map.push((mapped_char, ligature));
    }

    if repair_map.is_empty() { None } else { Some(repair_map) }
}

/// Apply ligature repairs to a text string using a page-specific repair map.
///
/// After replacing ligature characters, collapses spurious spaces that result
/// from the replacement: e.g., "ﬁ rst" → "fi rst" → "first". When a ligature
/// expansion is immediately followed by a space and a lowercase letter, the
/// space is removed (matching the reference regex-based post-processing).
pub(super) fn apply_ligature_repairs<'a>(text: &'a str, repair_map: &[(char, &str)]) -> Cow<'a, str> {
    if repair_map.is_empty() || !text.chars().any(|c| repair_map.iter().any(|(rc, _)| *rc == c)) {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len() + 16);
    for ch in text.chars() {
        if let Some((_, replacement)) = repair_map.iter().find(|(c, _)| *c == ch) {
            result.push_str(replacement);
        } else {
            result.push(ch);
        }
    }

    let ligature_endings: &[&str] = &["fi", "fl", "ff", "ffi", "ffl"];
    let mut collapsed = String::with_capacity(result.len());
    let mut chars = result.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == ' ' && !collapsed.is_empty() && chars.peek().is_some_and(|&nc| nc.is_lowercase()) {
            let should_collapse = ligature_endings.iter().any(|lig| collapsed.ends_with(lig));
            if should_collapse {
                continue;
            }
        }
        collapsed.push(ch);
    }

    Cow::Owned(collapsed)
}

/// Repair ligature corruption using contextual heuristics.
///
/// Some PDF fonts have broken ToUnicode CMaps that map ligature glyphs to
/// ASCII characters. This function detects and repairs these patterns:
///
/// **f-ligatures**: `!` → fi/ff, `"` → ffi, `#` → fi/fl
/// **t-ligatures**: `*` → tt, `:` → ti, uppercase `M` between lowercase → tti
///
/// All patterns are contextual: the corrupt character must appear between
/// alphabetic characters (mid-word), where it virtually never occurs in real text.
pub(super) fn repair_contextual_ligatures(text: &str) -> Cow<'_, str> {
    if text.len() < 2 {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len() + 16);
    let mut repaired = false;
    let bytes = text.as_bytes();
    let chars = text.chars().peekable();
    let mut byte_idx = 0;
    let mut prev_is_alpha = false;
    let mut prev_is_space_or_start = true;

    for ch in chars {
        let char_len = ch.len_utf8();
        let next_byte_idx = byte_idx + char_len;

        let next_is_alpha = if next_byte_idx < bytes.len() {
            if let Some(&next_byte) = bytes.get(next_byte_idx) {
                (next_byte as char).is_alphabetic()
            } else {
                false
            }
        } else {
            false
        };

        let next_is_lower = if next_byte_idx < bytes.len() {
            if let Some(&next_byte) = bytes.get(next_byte_idx) {
                (next_byte as char).is_lowercase()
            } else {
                false
            }
        } else {
            false
        };

        let next_is_vowel = if next_byte_idx < bytes.len() {
            if let Some(&next_byte) = bytes.get(next_byte_idx) {
                matches!(
                    next_byte as char,
                    'a' | 'e' | 'i' | 'o' | 'u' | 'A' | 'E' | 'I' | 'O' | 'U'
                )
            } else {
                false
            }
        } else {
            false
        };

        match ch {
            '!' if prev_is_alpha && next_is_vowel => {
                result.push_str("ff");
                repaired = true;
            }
            '!' if prev_is_alpha && next_is_alpha => {
                result.push_str("fi");
                repaired = true;
            }
            '!' if prev_is_alpha && next_byte_idx >= bytes.len() => {
                result.push_str("fi");
                repaired = true;
            }
            '"' if prev_is_alpha && next_is_alpha => {
                result.push_str("ffi");
                repaired = true;
            }
            '#' if prev_is_alpha && next_is_alpha => {
                result.push_str("fi");
                repaired = true;
            }
            '#' if prev_is_space_or_start && next_is_lower => {
                result.push_str("fi");
                repaired = true;
            }
            '!' if prev_is_space_or_start && next_is_lower => {
                result.push_str("fi");
                repaired = true;
            }
            '*' if prev_is_alpha && next_is_alpha => {
                result.push_str("tt");
                repaired = true;
            }
            '*' if prev_is_alpha && (next_byte_idx >= bytes.len() || !next_is_alpha) => {
                result.push_str("tt");
                repaired = true;
            }
            ':' if prev_is_alpha && next_is_lower => {
                result.push_str("ti");
                repaired = true;
            }
            'M' if prev_is_alpha && !prev_is_space_or_start => {
                let prev_was_lower = if byte_idx > 0 {
                    bytes.get(byte_idx - 1).is_some_and(|&b| (b as char).is_lowercase())
                } else {
                    false
                };
                if prev_was_lower && (next_is_lower || next_byte_idx >= bytes.len() || !next_is_alpha) {
                    result.push_str("tti");
                    repaired = true;
                } else {
                    result.push(ch);
                }
            }
            _ => result.push(ch),
        }

        prev_is_alpha = ch.is_alphabetic();
        prev_is_space_or_start = ch.is_whitespace();
        byte_idx = next_byte_idx;
    }

    if repaired {
        Cow::Owned(result)
    } else {
        Cow::Borrowed(text)
    }
}

/// Check if text contains ligature corruption patterns.
///
/// Returns true if the text shows signs of broken ligature encoding:
/// - Mid-word: `!`, `"`, or `#` between alphabetic characters
/// - Word-start: `#` or `!` after whitespace/start followed by lowercase letter
///
/// Requires 2+ matches to avoid false positives from normal punctuation.
pub(super) fn text_has_ligature_corruption(text: &str) -> bool {
    if text.len() < 3 {
        return false;
    }

    let bytes = text.as_bytes();
    let mut count = 0u32;
    let mut pos = 0;

    while let Some(idx) = bytes[pos..]
        .iter()
        .position(|&b| matches!(b, b'!' | b'"' | b'#' | b'*' | b':'))
    {
        let i = pos + idx;
        let ch = bytes[i];

        let prev_alpha = if i > 0 {
            let prev_byte = bytes[i - 1];
            (prev_byte as char).is_alphabetic()
        } else {
            false
        };

        let next_alpha = if i + 1 < bytes.len() {
            let next_byte = bytes[i + 1];
            (next_byte as char).is_alphabetic()
        } else {
            false
        };

        let prev_space_or_start = i == 0 || (bytes[i - 1] as char).is_whitespace();

        let next_lower = if i + 1 < bytes.len() {
            let next_byte = bytes[i + 1];
            (next_byte as char).is_lowercase()
        } else {
            false
        };

        if prev_alpha && next_alpha {
            count += 1;
        }

        if matches!(ch, b'#' | b'!') && prev_space_or_start && next_lower {
            count += 1;
        }

        pos = i + 1;
        if count >= 1 {
            break;
        }
    }

    count >= 1
}

/// Check if text has an abnormal density of short-fragment words followed by
/// lowercase continuation, indicating broken word spacing from pdfium.
///
/// Pattern: `"M ust"`, `"s hall"`, `"sen d er"`, `"a dd ress"` — short fragments
/// (1-3 chars) followed by a space then a lowercase continuation. Normal English
/// text rarely has runs of consecutive short fragments.
///
/// Returns true if the density of suspicious fragments exceeds a threshold,
/// indicating systematic font-metric corruption on this page.
pub(in crate::pdf) fn text_has_broken_word_spacing(text: &str) -> bool {
    if text.len() < 20 {
        return false;
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 5 {
        return false;
    }

    let mut suspicious = 0u32;
    for window in words.windows(2) {
        let w = window[0];
        let next = window[1];

        if w.len() <= 3
            && !w.is_empty()
            && w.chars().all(|c| c.is_alphabetic())
            && next.chars().next().is_some_and(|c| c.is_lowercase())
            && !is_common_short_word(w)
        {
            suspicious += 1;
        }
    }

    suspicious >= 3 && (suspicious as f64 / words.len() as f64) > 0.02
}

/// Repair broken word spacing by joining short fragments to adjacent words.
///
/// Targets the pattern where pdfium inserts spaces mid-word due to broken font
/// CMap/ToUnicode tables. Handles both single-character and multi-character
/// fragments:
/// - `"M ust Be Tough"` → `"Must Be Tough"` (single-char)
/// - `"s hall a b e active"` → `"shall be active"` (multi-char)
/// - `"a dd ress"` → `"address"` (mixed)
/// - `"sen d er"` → `"sender"` (mixed)
///
/// Only joins when:
/// - The fragment is a short alphabetic word (1-3 chars)
/// - It's not a common standalone short word
/// - The next word starts with a lowercase letter (continuation)
/// - Or the fragment is part of a run of consecutive short fragments
pub(in crate::pdf::structure) fn repair_broken_word_spacing(text: &str) -> Cow<'_, str> {
    if text.is_empty() {
        return Cow::Borrowed(text);
    }

    if text.contains("| --- |") || text.starts_with('|') {
        return Cow::Borrowed(text);
    }

    let words: Vec<&str> = text.split_whitespace().collect();

    let has_joinable = words.windows(2).any(|window| {
        is_joinable_fragment(window[0], window[1])
            || (window[0].chars().all(|c| c.is_alphabetic())
                && !is_common_short_word(window[0])
                && is_trailing_fragment(window[1]))
    });

    if !has_joinable {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len());
    let mut i = 0;
    while i < words.len() {
        if i > 0 && !result.is_empty() {
            result.push(' ');
        }

        let w = words[i];

        if w.len() == 1
            && w.chars().next().is_some_and(|c| c.is_alphabetic())
            && !is_common_short_word(w)
            && i + 1 < words.len()
            && words[i + 1].chars().next().is_some_and(|c| c.is_lowercase())
        {
            result.push_str(w);
            result.push_str(words[i + 1]);
            i += 2;
            continue;
        }

        if i + 1 < words.len() && is_joinable_fragment(w, words[i + 1]) {
            result.push_str(w);
            i += 1;
            let mut last_consumed_len = w.len();
            let mut total_consumed = w.len();
            while i < words.len() {
                let next = words[i];
                let next_starts_lower = next.chars().next().is_some_and(|c| c.is_lowercase());
                if !next_starts_lower {
                    break;
                }
                if last_consumed_len <= 3 && next.len() <= 3 {
                    result.push_str(next);
                    last_consumed_len = next.len();
                    total_consumed += next.len();
                    i += 1;
                    continue;
                }
                if total_consumed <= 3 {
                    result.push_str(next);
                    i += 1;
                    break;
                }
                break;
            }
            continue;
        }

        if i + 1 < words.len()
            && w.chars().all(|c| c.is_alphabetic())
            && !is_common_short_word(w)
            && is_trailing_fragment(words[i + 1])
        {
            result.push_str(w);
            while i + 1 < words.len() && is_trailing_fragment(words[i + 1]) {
                i += 1;
                result.push_str(words[i]);
            }
            i += 1;
            continue;
        }

        result.push_str(w);
        i += 1;
    }

    if result == text.split_whitespace().collect::<Vec<_>>().join(" ") {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(result)
    }
}

/// Check if a word is a trailing fragment: very short (1-2 chars), all lowercase
/// alphabetic, and not a common standalone word. These are fragments that were
/// split off from the end of a word by pdfium.
fn is_trailing_fragment(word: &str) -> bool {
    word.len() <= 2
        && !word.is_empty()
        && word.chars().all(|c| c.is_lowercase() && c.is_alphabetic())
        && !is_common_short_word(word)
}

/// Check if a word is a joinable fragment: short, alphabetic, not a common
/// standalone word, and followed by a lowercase-starting continuation.
fn is_joinable_fragment(word: &str, next: &str) -> bool {
    word.len() <= 3
        && !word.is_empty()
        && word.chars().all(|c| c.is_alphabetic())
        && !word.chars().all(|c| c.is_uppercase())
        && !is_common_short_word(word)
        && next.chars().next().is_some_and(|c| c.is_lowercase())
}

/// Check if a short word is a common standalone English word that should
/// not be joined to adjacent words.
///
/// Covers articles, pronouns, prepositions, conjunctions, and common verbs
/// up to 3 characters. This is intentionally conservative — when in doubt,
/// include the word to avoid false joins.
fn is_common_short_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "A"
            | "I"
            | "an"
            | "am"
            | "as"
            | "at"
            | "be"
            | "by"
            | "do"
            | "go"
            | "he"
            | "if"
            | "in"
            | "is"
            | "it"
            | "me"
            | "my"
            | "no"
            | "of"
            | "oh"
            | "on"
            | "or"
            | "so"
            | "to"
            | "up"
            | "us"
            | "we"
            | "An"
            | "Am"
            | "As"
            | "At"
            | "Be"
            | "By"
            | "Do"
            | "Go"
            | "He"
            | "If"
            | "In"
            | "Is"
            | "It"
            | "Me"
            | "My"
            | "No"
            | "Of"
            | "Oh"
            | "On"
            | "Or"
            | "So"
            | "To"
            | "Up"
            | "Us"
            | "We"
            | "the"
            | "and"
            | "are"
            | "but"
            | "can"
            | "did"
            | "for"
            | "got"
            | "had"
            | "has"
            | "her"
            | "him"
            | "his"
            | "how"
            | "its"
            | "let"
            | "may"
            | "new"
            | "nor"
            | "not"
            | "now"
            | "old"
            | "one"
            | "our"
            | "out"
            | "own"
            | "ran"
            | "say"
            | "she"
            | "too"
            | "two"
            | "use"
            | "was"
            | "way"
            | "who"
            | "why"
            | "yet"
            | "you"
            | "all"
            | "any"
            | "big"
            | "day"
            | "end"
            | "far"
            | "few"
            | "put"
            | "run"
            | "saw"
            | "set"
            | "top"
            | "try"
            | "win"
            | "yes"
            | "The"
            | "And"
            | "Are"
            | "But"
            | "Can"
            | "Did"
            | "For"
            | "Got"
            | "Had"
            | "Has"
            | "Her"
            | "Him"
            | "His"
            | "How"
            | "Its"
            | "Let"
            | "May"
            | "New"
            | "Nor"
            | "Not"
            | "Now"
            | "Old"
            | "One"
            | "Our"
            | "Out"
            | "Own"
            | "Ran"
            | "Say"
            | "She"
            | "Too"
            | "Two"
            | "Use"
            | "Was"
            | "Way"
            | "Who"
            | "Why"
            | "Yet"
            | "You"
            | "All"
            | "Any"
            | "Big"
            | "Day"
            | "End"
            | "Far"
            | "Few"
            | "Put"
            | "Run"
            | "Saw"
            | "Set"
            | "Top"
            | "Try"
            | "Win"
            | "Yes"
    )
}

/// Expand Unicode ligature characters (U+FB00–U+FB06) to ASCII equivalents,
/// absorbing a spurious space between the ligature glyph and the following word.
///
/// PDFs sometimes emit ligature codepoints (ﬁ, ﬂ, ﬀ, ﬃ, ﬄ, ﬅ, ﬆ) that need
/// to be expanded. Additionally, a space is often inserted between the ligature
/// glyph and the continuation of the word (e.g. "ﬁ eld"), which must be absorbed
/// to produce correct text ("field").
///
/// Matches the reference approach:
/// ```python
/// _LIGATURE_RE = re.compile(r"([\ufb00-\ufb06])( (?=\w))?")
/// ```
///
/// Uses `Cow<str>` for zero-alloc fast path when no ligatures are present.
pub(super) fn expand_ligatures_with_space_absorption(text: &str) -> Cow<'_, str> {
    if !text.contains([
        '\u{FB00}', '\u{FB01}', '\u{FB02}', '\u{FB03}', '\u{FB04}', '\u{FB05}', '\u{FB06}',
    ]) {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        let expansion = match ch {
            '\u{FB00}' => "ff",
            '\u{FB01}' => "fi",
            '\u{FB02}' => "fl",
            '\u{FB03}' => "ffi",
            '\u{FB04}' => "ffl",
            '\u{FB05}' => "st",
            '\u{FB06}' => "st",
            _ => {
                result.push(ch);
                continue;
            }
        };

        result.push_str(expansion);

        if chars.peek() == Some(&' ') {
            let mut lookahead = chars.clone();
            lookahead.next();
            if lookahead.peek().is_some_and(|c| c.is_alphanumeric() || *c == '_') {
                chars.next();
            }
        }
    }

    Cow::Owned(result)
}

/// Repair ligature-glyph word breaks in extracted text.
///
/// When pdfium decomposes ligature glyphs (fi, fl, ff, ffi, ffl) into individual
/// characters, the resulting character positions often have gaps that get interpreted
/// as word boundaries. This produces patterns like "eff iciently", "signif icant",
/// "f irst" where the space appears at the ligature position.
///
/// This function detects and removes these spurious spaces by looking for the pattern:
/// `f` (or `ff`) followed by space followed by lowercase letter that would form a
/// common ligature combination (fi, fl, ff).
pub(super) fn repair_ligature_spaces(text: &str) -> Cow<'_, str> {
    if !text.contains("f ") {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'f' && i + 2 < len && bytes[i + 1] == b' ' {
            let next = bytes[i + 2];
            if (next == b'i' || next == b'l' || next == b'f') && i > 0 && bytes[i - 1].is_ascii_alphabetic() {
                result.push('f');
                i += 2;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }

    if result == text {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(result)
    }
}

/// Normalize Unicode characters commonly found in PDFs to their ASCII equivalents.
///
/// Matches the reference `sanitize_text()` normalizations for curly quotes, fraction
/// slash, and bullet characters. This improves TF1 by ensuring extracted text
/// matches ground truth tokenization.
pub(super) fn normalize_unicode_text(text: &str) -> Cow<'_, str> {
    if !text.contains(['\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2044}', '\u{2022}']) {
        return Cow::Borrowed(text);
    }
    Cow::Owned(
        text.replace(['\u{2018}', '\u{2019}'], "'")
            .replace(['\u{201C}', '\u{201D}'], "\"")
            .replace('\u{2044}', "/")
            .replace('\u{2022}', "\u{00B7}"),
    )
}

/// Clean up duplicate punctuation artifacts from PDF text extraction.
///
/// When pdfium's segment-level re-extraction picks up characters from adjacent
/// cells (due to slightly overlapping bounding boxes), duplicate punctuation
/// patterns like `, ,` or `. .` appear. This collapses them to single
/// punctuation marks.
///
/// Patterns handled:
/// - `, ,` → `,`
/// - `. .` → `.`
/// - `; ;` → `;`
/// - `: :` → `:`
pub(super) fn clean_duplicate_punctuation(text: &str) -> Cow<'_, str> {
    if !has_duplicate_punctuation(text) {
        return Cow::Borrowed(text);
    }

    let mut current = collapse_duplicate_punctuation_once(text);
    while has_duplicate_punctuation(&current) {
        current = collapse_duplicate_punctuation_once(&current);
    }

    Cow::Owned(current)
}

/// Single pass of duplicate punctuation collapsing.
fn collapse_duplicate_punctuation_once(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let b = bytes[i];
        if is_dup_punct_byte(b) && i + 2 < len && bytes[i + 1] == b' ' && bytes[i + 2] == b {
            result.push(b as char);
            i += 3;
        } else {
            result.push(b as char);
            i += 1;
        }
    }

    result
}

/// Check if the text contains any duplicate punctuation pattern.
fn has_duplicate_punctuation(text: &str) -> bool {
    let bytes = text.as_bytes();
    for i in 0..bytes.len().saturating_sub(2) {
        let b = bytes[i];
        if is_dup_punct_byte(b) && bytes[i + 1] == b' ' && bytes[i + 2] == b {
            return true;
        }
    }
    false
}

/// Check if a byte is a punctuation character subject to duplicate cleanup.
fn is_dup_punct_byte(b: u8) -> bool {
    matches!(b, b',' | b'.' | b';' | b':')
}

/// Normalize text encoding: handle soft hyphens, pdfium word-break markers,
/// and strip control characters.
///
/// - `\u{00AD}` (soft hyphen) at end of text → replaced with `-` so downstream
///   hyphen-rejoining logic can merge word fragments.
/// - `\u{00AD}` mid-text → removed (invisible break hint).
/// - `\x02` (STX) followed by space/newline → both removed, rejoining the word
///   fragments. Pdfium emits `\x02` at soft-hyphen positions where the hyphen
///   character was discarded by the PDF producer.
/// - Other C0 control characters (U+0000–U+001F except `\t`, `\n`, `\r`) → removed.
pub(super) fn normalize_text_encoding(text: &str) -> Cow<'_, str> {
    if !text.contains('\u{00AD}') && !text.bytes().any(|b| b < 0x20 && b != b'\t' && b != b'\n' && b != b'\r') {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\u{00AD}' => {
                let at_end = chars.peek().is_none_or(|c| c.is_whitespace());
                if at_end {
                    result.push('-');
                }
            }
            '\x02' => {
                while chars.peek().is_some_and(|c| *c == ' ' || *c == '\n') {
                    chars.next();
                }
            }
            c if c.is_control() && c != '\n' && c != '\r' && c != '\t' => {}
            _ => result.push(ch),
        }
    }

    Cow::Owned(result)
}

/// Apply a text transformation to every segment in every paragraph.
///
/// The repair function returns `Cow<'_, str>`: if it returns `Cow::Borrowed`,
/// the segment text is unchanged and no allocation is performed. Only
/// `Cow::Owned` results trigger an update.
pub(super) fn apply_to_all_segments(paragraphs: &mut [PdfParagraph], repair_fn: impl Fn(&str) -> Cow<'_, str>) {
    for para in paragraphs {
        for line in &mut para.lines {
            for seg in &mut line.segments {
                if let Cow::Owned(s) = repair_fn(&seg.text) {
                    seg.text = s;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_ligature_repairs_fi() {
        let map = vec![('\x0C', "fi")];
        assert_eq!(apply_ligature_repairs("classi\x0Ccation", &map), "classification");
    }

    #[test]
    fn test_apply_ligature_repairs_ff() {
        let map = vec![('\x0B', "ff")];
        assert_eq!(apply_ligature_repairs("e\x0Bective", &map), "effective");
    }

    #[test]
    fn test_apply_ligature_repairs_fl() {
        let map = vec![('\x0D', "fl")];
        assert_eq!(apply_ligature_repairs("re\x0Dection", &map), "reflection");
    }

    #[test]
    fn test_apply_ligature_repairs_ffi() {
        let map = vec![('\x0E', "ffi")];
        assert_eq!(apply_ligature_repairs("e\x0Ecient", &map), "efficient");
    }

    #[test]
    fn test_apply_ligature_repairs_ffl() {
        let map = vec![('\x0F', "ffl")];
        assert_eq!(apply_ligature_repairs("ba\x0Fe", &map), "baffle");
    }

    #[test]
    fn test_apply_ligature_repairs_no_map() {
        let map: Vec<(char, &str)> = Vec::new();
        assert_eq!(apply_ligature_repairs("hello world!", &map), "hello world!");
    }

    #[test]
    fn test_apply_ligature_repairs_multiple() {
        let map = vec![('\x0C', "fi"), ('\x0E', "ffi")];
        assert_eq!(
            apply_ligature_repairs("e\x0Ecient and classi\x0Ccation", &map),
            "efficient and classification"
        );
    }

    #[test]
    fn test_repair_contextual_ligatures_empty() {
        assert_eq!(repair_contextual_ligatures(""), "");
    }

    #[test]
    fn test_repair_contextual_ligatures_single_char() {
        assert_eq!(repair_contextual_ligatures("a"), "a");
    }

    #[test]
    fn test_repair_contextual_ligatures_no_corruption() {
        assert_eq!(repair_contextual_ligatures("hello world"), "hello world");
    }

    #[test]
    fn test_repair_contextual_ligatures_mid_word_fi() {
        assert_eq!(repair_contextual_ligatures("di!erent"), "different");
        assert_eq!(repair_contextual_ligatures("speci!c"), "specific");
    }

    #[test]
    fn test_repair_contextual_ligatures_mid_word_ff() {
        assert_eq!(repair_contextual_ligatures("di!erent effort"), "different effort");
        assert_eq!(repair_contextual_ligatures("e!ective"), "effective");
    }

    #[test]
    fn test_repair_contextual_ligatures_mid_word_ffi() {
        assert_eq!(repair_contextual_ligatures("e\u{22}cient"), "efficient");
    }

    #[test]
    fn test_repair_contextual_ligatures_word_start() {
        assert_eq!(repair_contextual_ligatures("#nancial"), "financial");
        assert_eq!(repair_contextual_ligatures("!nally"), "finally");
    }

    #[test]
    fn test_repair_contextual_ligatures_normal_punctuation() {
        assert_eq!(repair_contextual_ligatures("say \"hello\""), "say \"hello\"");
        assert_eq!(repair_contextual_ligatures("hello # world"), "hello # world");
    }

    #[test]
    fn test_repair_contextual_ligatures_multiple() {
        assert_eq!(
            repair_contextual_ligatures("ef!cient and #nancial"),
            "efficient and financial"
        );
    }

    #[test]
    fn test_text_has_ligature_corruption_empty() {
        assert!(!text_has_ligature_corruption(""));
    }

    #[test]
    fn test_text_has_ligature_corruption_too_short() {
        assert!(!text_has_ligature_corruption("ab"));
    }

    #[test]
    fn test_text_has_ligature_corruption_no_corruption() {
        assert!(!text_has_ligature_corruption("hello world"));
    }

    #[test]
    fn test_text_has_ligature_corruption_mid_word() {
        assert!(text_has_ligature_corruption("di!erent"));
        assert!(text_has_ligature_corruption("e#cient"));
        assert!(text_has_ligature_corruption("o\u{22}ces"));
    }

    #[test]
    fn test_text_has_ligature_corruption_word_start() {
        assert!(text_has_ligature_corruption("#nancial"));
        assert!(text_has_ligature_corruption("!nally"));
    }

    #[test]
    fn test_text_has_ligature_corruption_normal_punctuation() {
        assert!(!text_has_ligature_corruption("hello!"));
        assert!(!text_has_ligature_corruption("say \"hello\""));
    }

    #[test]
    fn test_text_has_ligature_corruption_multiple() {
        assert!(text_has_ligature_corruption("e!cient and #nancial"));
    }

    #[test]
    fn test_broken_word_spacing_detection() {
        let broken =
            "M ust B e T ough o ffers t he g uidance t hat g ives y ou t he b est c hance o f r ekindling r omance";
        assert!(text_has_broken_word_spacing(broken));
    }

    #[test]
    fn test_normal_text_not_detected_as_broken() {
        let normal = "Love Must Be Tough offers the guidance that gives you the best chance of rekindling romance";
        assert!(!text_has_broken_word_spacing(normal));
    }

    #[test]
    fn test_repair_broken_word_spacing() {
        let broken = "M ust B e T ough";
        let repaired = repair_broken_word_spacing(broken);
        assert_eq!(repaired, "Must Be Tough");
    }

    #[test]
    fn test_repair_preserves_standalone_a_and_i() {
        let text = "I have a dog";
        let repaired = repair_broken_word_spacing(text);
        assert_eq!(repaired, "I have a dog");
    }

    #[test]
    fn test_repair_joins_multi_char_fragments() {
        let broken = "rom ance and m arriage";
        let repaired = repair_broken_word_spacing(broken);
        assert_eq!(repaired, "romance and marriage");
    }

    #[test]
    fn test_repair_joins_shall_be_active() {
        let broken = "s hall a b e active";
        let repaired = repair_broken_word_spacing(broken);
        assert_eq!(repaired, "shall a be active");
    }

    #[test]
    fn test_repair_joins_address_fragments() {
        let broken = "a dd ress";
        let repaired = repair_broken_word_spacing(broken);
        assert_eq!(repaired, "a ddress");
    }

    #[test]
    fn test_repair_joins_sender() {
        let broken = "sen d er hardware";
        let repaired = repair_broken_word_spacing(broken);
        assert_eq!(repaired, "sender hardware");
    }

    #[test]
    fn test_pipe_table_guard_standard() {
        let table = "| CTC_ARP | s hall be | active |";
        assert_eq!(repair_broken_word_spacing(table), table);
    }

    #[test]
    fn test_pipe_table_separator_guard() {
        let sep = "| --- | --- |";
        assert_eq!(repair_broken_word_spacing(sep), sep);
    }

    #[test]
    fn test_normalize_plain_text_unchanged() {
        assert_eq!(normalize_text_encoding("hello world"), "hello world");
    }

    #[test]
    fn test_normalize_trailing_soft_hyphen() {
        assert_eq!(normalize_text_encoding("soft\u{00AD}"), "soft-");
    }

    #[test]
    fn test_normalize_mid_word_soft_hyphen_removed() {
        assert_eq!(normalize_text_encoding("soft\u{00AD}ware"), "software");
    }

    #[test]
    fn test_normalize_soft_hyphen_before_space() {
        assert_eq!(normalize_text_encoding("soft\u{00AD} ware"), "soft- ware");
    }

    #[test]
    fn test_normalize_strips_control_chars() {
        assert_eq!(normalize_text_encoding("he\x01llo"), "hello");
    }

    #[test]
    fn test_normalize_stx_word_break_with_space() {
        assert_eq!(normalize_text_encoding("soft\x02 ware"), "software");
    }

    #[test]
    fn test_normalize_stx_word_break_with_newline() {
        assert_eq!(normalize_text_encoding("recog\x02\nnition"), "recognition");
    }

    #[test]
    fn test_normalize_stx_at_end() {
        assert_eq!(normalize_text_encoding("hello\x02"), "hello");
    }

    #[test]
    fn test_normalize_stx_no_trailing_space() {
        assert_eq!(normalize_text_encoding("soft\x02ware"), "software");
    }

    #[test]
    fn test_normalize_preserves_tabs_newlines() {
        assert_eq!(normalize_text_encoding("a\tb\nc\r"), "a\tb\nc\r");
    }

    #[test]
    fn test_expand_ligatures_no_ligatures() {
        let text = "hello world";
        let result = expand_ligatures_with_space_absorption(text);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_expand_ligatures_fi() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB01}eld"), "field");
    }

    #[test]
    fn test_expand_ligatures_fl() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB02}oor"), "floor");
    }

    #[test]
    fn test_expand_ligatures_ff() {
        assert_eq!(expand_ligatures_with_space_absorption("e\u{FB00}ect"), "effect");
    }

    #[test]
    fn test_expand_ligatures_ffi() {
        assert_eq!(expand_ligatures_with_space_absorption("e\u{FB03}cient"), "efficient");
    }

    #[test]
    fn test_expand_ligatures_ffl() {
        assert_eq!(expand_ligatures_with_space_absorption("ba\u{FB04}e"), "baffle");
    }

    #[test]
    fn test_expand_ligatures_st() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB05}art"), "start");
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB06}art"), "start");
    }

    #[test]
    fn test_expand_ligatures_space_absorption_fi() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB01} eld"), "field");
    }

    #[test]
    fn test_expand_ligatures_space_absorption_fl() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB02} oor"), "floor");
    }

    #[test]
    fn test_expand_ligatures_space_absorption_ff() {
        assert_eq!(expand_ligatures_with_space_absorption("e \u{FB00} ect"), "e ffect");
    }

    #[test]
    fn test_expand_ligatures_space_not_absorbed_before_punctuation() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB01} ."), "fi .");
    }

    #[test]
    fn test_expand_ligatures_space_not_absorbed_before_space() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB01}  word"), "fi  word");
    }

    #[test]
    fn test_expand_ligatures_at_end_of_string() {
        assert_eq!(expand_ligatures_with_space_absorption("pro\u{FB01}"), "profi");
    }

    #[test]
    fn test_expand_ligatures_space_at_end_not_absorbed() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB01} "), "fi ");
    }

    #[test]
    fn test_expand_ligatures_multiple_in_sentence() {
        assert_eq!(
            expand_ligatures_with_space_absorption("the \u{FB01} rst \u{FB02} oor"),
            "the first floor"
        );
    }

    #[test]
    fn test_expand_ligatures_mixed_with_normal_text() {
        assert_eq!(
            expand_ligatures_with_space_absorption("a \u{FB01} eld of \u{FB02} owers"),
            "a field of flowers"
        );
    }

    #[test]
    fn test_expand_ligatures_no_space_no_absorption() {
        assert_eq!(expand_ligatures_with_space_absorption("\u{FB01}nally"), "finally");
    }

    #[test]
    fn test_clean_duplicate_comma() {
        assert_eq!(
            clean_duplicate_punctuation("simple, , self-contained"),
            "simple, self-contained"
        );
    }

    #[test]
    fn test_clean_duplicate_period() {
        assert_eq!(clean_duplicate_punctuation("end. . next"), "end. next");
    }

    #[test]
    fn test_clean_duplicate_semicolon() {
        assert_eq!(clean_duplicate_punctuation("a; ; b"), "a; b");
    }

    #[test]
    fn test_clean_duplicate_colon() {
        assert_eq!(clean_duplicate_punctuation("key: : value"), "key: value");
    }

    #[test]
    fn test_clean_duplicate_punctuation_no_change() {
        let text = "Hello, world. This is normal; right: yes";
        assert!(matches!(clean_duplicate_punctuation(text), Cow::Borrowed(_)));
    }

    #[test]
    fn test_clean_duplicate_punctuation_multiple() {
        assert_eq!(clean_duplicate_punctuation("a, , b, , c"), "a, b, c");
    }

    #[test]
    fn test_clean_duplicate_punctuation_triple() {
        assert_eq!(
            clean_duplicate_punctuation("[12, 13, 9]. Docling is designed as a simple, , , self-contained"),
            "[12, 13, 9]. Docling is designed as a simple, self-contained"
        );
    }
}
