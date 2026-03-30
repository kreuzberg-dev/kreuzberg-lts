//! Native Rust LaTeX text extractor.
//!
//! This extractor provides comprehensive LaTeX document parsing and text extraction.
//!
//! Features:
//! - Metadata extraction: title, author, date from \title{}, \author{}, \date{}
//! - Section hierarchy: \section{}, \subsection{}, \subsubsection{}, etc.
//! - Inline formatting: \emph{}, \textbf{}, \textit{}, \texttt{}, \underline{}
//! - Lists: itemize, enumerate, description environments
//! - Tables: tabular environment parsing
//! - Math: inline ($...$) and display (\[...\]) math preservation
//! - Unicode support
//!
//! Requires the `office` feature.

mod commands;
mod environments;
mod metadata;
mod parser;
mod utilities;

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::builder::DocumentStructureBuilder;
use crate::types::document_structure::{AnnotationKind, DocumentStructure, TextAnnotation};
use crate::types::{ExtractionResult, Metadata, Table};
use async_trait::async_trait;

use std::sync::LazyLock;

use parser::LatexParser;
use utilities::{collect_environment, extract_env_name, extract_heading_title};

/// Heading command → level map for documents that contain `\chapter` commands.
/// Chapter occupies level 1; section is demoted to level 2, etc.
static HEADING_LEVELS_WITH_CHAPTERS: LazyLock<ahash::AHashMap<&'static str, u8>> = LazyLock::new(|| {
    let mut m = ahash::AHashMap::with_capacity(10);
    m.insert("chapter", 1);
    m.insert("chapter*", 1);
    m.insert("section", 2);
    m.insert("section*", 2);
    m.insert("subsection", 3);
    m.insert("subsection*", 3);
    m.insert("subsubsection", 4);
    m.insert("subsubsection*", 4);
    m.insert("paragraph", 5);
    m.insert("paragraph*", 5);
    m
});

/// Heading command → level map for documents without `\chapter` commands.
/// Section starts at level 1.
static HEADING_LEVELS_NO_CHAPTERS: LazyLock<ahash::AHashMap<&'static str, u8>> = LazyLock::new(|| {
    let mut m = ahash::AHashMap::with_capacity(8);
    m.insert("section", 1);
    m.insert("section*", 1);
    m.insert("subsection", 2);
    m.insert("subsection*", 2);
    m.insert("subsubsection", 3);
    m.insert("subsubsection*", 3);
    m.insert("paragraph", 4);
    m.insert("paragraph*", 4);
    m
});

/// LaTeX document extractor
pub struct LatexExtractor;

impl LatexExtractor {
    /// Create a new LaTeX extractor.
    pub fn new() -> Self {
        Self
    }

    /// Parse LaTeX content and extract text.
    fn extract_from_latex(content: &str) -> (String, Metadata, Vec<Table>) {
        let mut parser = LatexParser::new(content);
        parser.parse()
    }

    /// Strip inline LaTeX formatting commands from text, returning the plain text
    /// and a list of `TextAnnotation`s referencing byte offsets in the output.
    ///
    /// Handles: `\textbf`, `\emph`, `\textit`, `\underline`, `\texttt`, `\href`.
    fn strip_inline_commands(input: &str) -> (String, Vec<TextAnnotation>) {
        let mut output = String::with_capacity(input.len());
        let mut annotations = Vec::new();
        let bytes = input.as_bytes();
        let len = bytes.len();
        let mut pos = 0;

        while pos < len {
            if bytes[pos] == b'\\' {
                // Try to match known inline commands
                if let Some((kind, content, new_pos)) = Self::try_parse_inline_command(&input[pos..]) {
                    let start = output.len() as u32;
                    // Recursively strip inner commands
                    let (inner_text, inner_anns) = Self::strip_inline_commands(&content);
                    output.push_str(&inner_text);
                    let end = output.len() as u32;
                    // Adjust inner annotations to absolute offsets
                    for mut ann in inner_anns {
                        ann.start += start;
                        ann.end += start;
                        annotations.push(ann);
                    }
                    if start < end {
                        annotations.push(TextAnnotation { start, end, kind });
                    }
                    pos += new_pos;
                    continue;
                }
                // Not a recognized command, copy the backslash
                output.push('\\');
                pos += 1;
            } else {
                output.push(input[pos..].chars().next().unwrap());
                pos += input[pos..].chars().next().unwrap().len_utf8();
            }
        }

        (output, annotations)
    }

    /// Try to parse an inline formatting command at the start of `text`.
    ///
    /// Returns `Some((kind, braced_content, bytes_consumed))` on success.
    fn try_parse_inline_command(text: &str) -> Option<(AnnotationKind, String, usize)> {
        // Map command names to annotation kinds
        let commands: &[(&str, AnnotationKind)] = &[
            ("\\textbf{", AnnotationKind::Bold),
            ("\\emph{", AnnotationKind::Italic),
            ("\\textit{", AnnotationKind::Italic),
            ("\\underline{", AnnotationKind::Underline),
            ("\\texttt{", AnnotationKind::Code),
        ];

        for (prefix, kind) in commands {
            if let Some(after) = text.strip_prefix(prefix)
                && let Some((content, consumed)) = Self::read_braced_content(after)
            {
                return Some((kind.clone(), content, prefix.len() + consumed));
            }
        }

        // Handle \href{url}{text}
        if let Some(after_href) = text.strip_prefix("\\href{")
            && let Some((url, url_consumed)) = Self::read_braced_content(after_href)
        {
            let after_url = &after_href[url_consumed..];
            if let Some(after_brace) = after_url.strip_prefix('{')
                && let Some((link_text, text_consumed)) = Self::read_braced_content(after_brace)
            {
                let total = "\\href{".len() + url_consumed + 1 + text_consumed;
                return Some((AnnotationKind::Link { url, title: None }, link_text, total));
            }
        }

        None
    }

    /// Read braced content starting after an opening `{` has already been consumed
    /// by the prefix match. The input starts at the first character inside the braces.
    ///
    /// Returns `(content, bytes_consumed_including_closing_brace)`.
    fn read_braced_content(input: &str) -> Option<(String, usize)> {
        let mut depth: u32 = 1;
        let mut content = String::new();
        let mut pos = 0;
        let bytes = input.as_bytes();

        while pos < bytes.len() {
            let ch = input[pos..].chars().next()?;
            let ch_len = ch.len_utf8();
            match ch {
                '{' => {
                    depth += 1;
                    content.push(ch);
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some((content, pos + ch_len));
                    }
                    content.push(ch);
                }
                _ => content.push(ch),
            }
            pos += ch_len;
        }
        None
    }

    /// Extract the path from `\includegraphics[opts]{path}` or `\includegraphics{path}`.
    fn extract_includegraphics_path(line: &str) -> Option<String> {
        let prefix = "\\includegraphics";
        let start = line.find(prefix)?;
        let after = &line[start + prefix.len()..];
        // Skip optional [...]
        let rest = if after.starts_with('[') {
            let bracket_end = after.find(']')?;
            &after[bracket_end + 1..]
        } else {
            after
        };
        if !rest.starts_with('{') {
            return None;
        }
        let inner = &rest[1..];
        let end = inner.find('}')?;
        let path = inner[..end].trim();
        if path.is_empty() { None } else { Some(path.to_string()) }
    }

    /// Extract the text from `\caption{text}`.
    fn extract_caption(content: &str) -> Option<String> {
        let prefix = "\\caption{";
        let start = content.find(prefix)?;
        let after = &content[start + prefix.len()..];
        Self::read_braced_content(after).map(|(text, _)| text)
    }

    /// Build a `DocumentStructure` from LaTeX source via a lightweight second pass.
    fn build_document_structure(source: &str) -> DocumentStructure {
        let mut builder = DocumentStructureBuilder::new().source_format("latex");
        let lines: Vec<&str> = source.lines().collect();
        let mut in_document = false;
        let is_plain_tex = source.contains("\\bye") && !source.contains("\\begin{document}");
        if is_plain_tex {
            in_document = true;
        }
        // When the document contains \chapter, use absolute LaTeX hierarchy
        // (chapter=1, section=2, ...). Otherwise, section starts at level 1.
        // Use a static AHashMap for O(1) lookup instead of a linear scan.
        let has_chapters = source.contains("\\chapter{") || source.contains("\\chapter*{");
        let heading_map = if has_chapters {
            &*HEADING_LEVELS_WITH_CHAPTERS
        } else {
            &*HEADING_LEVELS_NO_CHAPTERS
        };

        // Extract metadata from preamble (\title, \author, \date)
        let mut metadata_entries: Vec<(String, String)> = Vec::new();
        for &cmd in &["title", "author", "date"] {
            if let Some(value) = utilities::extract_braced(source, cmd)
                && !value.is_empty()
            {
                metadata_entries.push((cmd.to_string(), value));
            }
        }
        if !metadata_entries.is_empty() {
            builder.push_metadata_block(metadata_entries, None);
        }

        let mut i = 0;

        while i < lines.len() {
            let trimmed = lines[i].trim();

            if is_plain_tex && trimmed.contains("\\bye") {
                break;
            }

            if !is_plain_tex && trimmed.contains("\\begin{document}") {
                in_document = true;
                i += 1;
                continue;
            }

            if !is_plain_tex && trimmed.contains("\\end{document}") {
                break;
            }

            if !in_document {
                i += 1;
                continue;
            }

            // Handle environments
            if (trimmed.contains("\\begin{") || trimmed.contains("\\begin {"))
                && let Some(env_name) = extract_env_name(trimmed)
            {
                match env_name.as_str() {
                    "itemize" | "enumerate" | "description" => {
                        let ordered = env_name == "enumerate";
                        let (env_content, new_i) = collect_environment(&lines, i, &env_name);
                        let list_idx = builder.push_list(ordered, None);
                        Self::build_list_items(&mut builder, &env_content, list_idx);
                        i = new_i;
                        continue;
                    }
                    "tabular" => {
                        let (env_content, new_i) = collect_environment(&lines, i, "tabular");
                        let cells = Self::parse_tabular_cells(&env_content);
                        if !cells.is_empty() {
                            builder.push_table_from_cells(&cells, None);
                        }
                        i = new_i;
                        continue;
                    }
                    "table" => {
                        let (env_content, new_i) = collect_environment(&lines, i, "table");
                        // Extract caption from the table environment
                        let caption = Self::extract_caption(&env_content);
                        // Extract tabular inside table environment
                        let end_tag = "\\end{tabular}";
                        if env_content.contains("\\begin{tabular}")
                            && let Some(start) = env_content.find("\\begin{tabular}")
                            && let Some(end) = env_content.find(end_tag)
                        {
                            let tabular_content = &env_content[start..end + end_tag.len()];
                            let inner_lines: Vec<&str> = tabular_content.lines().collect();
                            let (inner_content, _) = collect_environment(&inner_lines, 0, "tabular");
                            let cells = Self::parse_tabular_cells(&inner_content);
                            if !cells.is_empty() {
                                let idx = builder.push_table_from_cells(&cells, None);
                                if let Some(cap) = caption {
                                    let mut attrs = ahash::AHashMap::new();
                                    attrs.insert("caption".to_string(), cap);
                                    builder.set_attributes(idx, attrs);
                                }
                            }
                        }
                        i = new_i;
                        continue;
                    }
                    "figure" => {
                        let (env_content, new_i) = collect_environment(&lines, i, "figure");
                        let caption = Self::extract_caption(&env_content);
                        // Extract \includegraphics from figure content
                        if let Some(path) = Self::extract_includegraphics_path(&env_content) {
                            let idx = builder.push_image(Some(&path), None, None, None);
                            if let Some(cap) = caption {
                                let mut attrs = ahash::AHashMap::new();
                                attrs.insert("caption".to_string(), cap);
                                builder.set_attributes(idx, attrs);
                            }
                        }
                        i = new_i;
                        continue;
                    }
                    "equation" | "equation*" | "align" | "align*" | "gather" | "gather*" | "multline" | "multline*"
                    | "eqnarray" | "eqnarray*" | "math" | "displaymath" | "flalign" | "flalign*" | "cases" => {
                        let (env_content, new_i) = collect_environment(&lines, i, &env_name);
                        let formula_text = format!("\\begin{{{}}}\n{}\\end{{{}}}", env_name, env_content, env_name);
                        builder.push_formula(&formula_text, None);
                        i = new_i;
                        continue;
                    }
                    "lstlisting" | "verbatim" | "minted" => {
                        let (env_content, new_i) = collect_environment(&lines, i, &env_name);
                        // Try to extract language from lstlisting options
                        let language = if env_name == "lstlisting" || env_name == "minted" {
                            Self::extract_code_language(trimmed)
                        } else {
                            None
                        };
                        builder.push_code(env_content.trim(), language, None);
                        i = new_i;
                        continue;
                    }
                    _ => {
                        // Skip other environments
                        let (_, new_i) = collect_environment(&lines, i, &env_name);
                        i = new_i;
                        continue;
                    }
                }
            }

            // Handle heading commands — O(1) map lookup instead of a linear scan.
            // Strip the leading backslash from `trimmed` (e.g. "\section{Intro}" → "section{Intro}"),
            // then extract the bare command name up to the first `{` or `[` to look it up.
            let mut handled = false;
            if let Some(after_backslash) = trimmed.strip_prefix('\\') {
                // Find where the command name ends (first `{`, `[`, or whitespace)
                let cmd_end = after_backslash
                    .find(|c: char| c == '{' || c == '[' || c.is_whitespace())
                    .unwrap_or(after_backslash.len());
                let cmd_name = &after_backslash[..cmd_end];
                if let Some(&level) = heading_map.get(cmd_name) {
                    let rest = &after_backslash[cmd_end..];
                    let rest = rest.trim_start();
                    if rest.starts_with('{') || rest.starts_with('[') {
                        if let Some(title) = extract_heading_title(trimmed, cmd_name) {
                            builder.push_heading(level, &title, None, None);
                        }
                        handled = true;
                    }
                }
            }

            if !handled && !trimmed.is_empty() && !trimmed.starts_with('%') {
                // Handle standalone \includegraphics outside figure environments
                if trimmed.contains("\\includegraphics")
                    && let Some(path) = Self::extract_includegraphics_path(trimmed)
                {
                    builder.push_image(Some(&path), None, None, None);
                    i += 1;
                    continue;
                }

                // Handle display math \[...\]
                if trimmed.starts_with("\\[") {
                    let mut math_content = trimmed.to_string();
                    if !trimmed.contains("\\]") {
                        i += 1;
                        while i < lines.len() {
                            math_content.push('\n');
                            math_content.push_str(lines[i]);
                            if lines[i].trim().contains("\\]") {
                                break;
                            }
                            i += 1;
                        }
                    }
                    let formula = math_content.trim_start_matches("\\[").trim_end_matches("\\]").trim();
                    if !formula.is_empty() {
                        builder.push_formula(formula, None);
                    }
                } else if trimmed.contains('$') && !trimmed.starts_with('\\') {
                    // Lines containing inline math - treat as paragraph with annotations
                    let (text, annotations) = Self::strip_inline_commands(trimmed);
                    builder.push_paragraph(&text, annotations, None, None);
                } else if !trimmed.starts_with('\\')
                    || trimmed.starts_with("\\textbf")
                    || trimmed.starts_with("\\emph")
                    || trimmed.starts_with("\\textit")
                    || trimmed.starts_with("\\underline")
                    || trimmed.starts_with("\\texttt")
                    || trimmed.starts_with("\\href")
                {
                    // Extract footnotes before processing inline formatting.
                    // Each \footnote{text} is emitted as a separate footnote node,
                    // and the command is removed from the paragraph text.
                    let mut line_text = trimmed.to_string();
                    while let Some(fn_start) = line_text.find("\\footnote{") {
                        let after = &line_text[fn_start + "\\footnote{".len()..];
                        if let Some((fn_text, consumed)) = Self::read_braced_content(after) {
                            let fn_stripped = utilities::clean_text(&fn_text);
                            if !fn_stripped.is_empty() {
                                builder.push_footnote(&fn_stripped, None);
                            }
                            let end = fn_start + "\\footnote{".len() + consumed;
                            line_text = format!("{}{}", &line_text[..fn_start], &line_text[end..]);
                        } else {
                            break;
                        }
                    }

                    let line_text = line_text.trim();
                    if !line_text.is_empty() {
                        let (text, annotations) = Self::strip_inline_commands(line_text);
                        builder.push_paragraph(&text, annotations, None, None);
                    }
                }
            }

            i += 1;
        }

        builder.build()
    }

    /// Parse tabular cells from environment content.
    fn parse_tabular_cells(content: &str) -> Vec<Vec<String>> {
        let mut rows = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\\hline")
                || trimmed.is_empty()
                || trimmed.contains("\\begin{tabular}")
                || trimmed.contains("\\end{tabular}")
            {
                continue;
            }
            let row_str = trimmed.replace("\\\\", "");
            let cells: Vec<String> = row_str
                .split('&')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
        rows
    }

    /// Build list items from a list environment's content.
    fn build_list_items(
        builder: &mut DocumentStructureBuilder,
        content: &str,
        list_idx: crate::types::document_structure::NodeIndex,
    ) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\\item") {
                let after = trimmed.strip_prefix("\\item").unwrap_or("").trim();
                // Handle \item[label] for description lists
                let text = if after.starts_with('[') {
                    if let Some(bracket_end) = after.find(']') {
                        let label = &after[1..bracket_end];
                        let rest = after[bracket_end + 1..].trim();
                        format!("{}: {}", label, rest)
                    } else {
                        after.to_string()
                    }
                } else {
                    after.to_string()
                };
                if !text.is_empty() {
                    builder.push_list_item(list_idx, &text, None);
                }
            }
        }
    }

    /// Extract language from code environment options.
    fn extract_code_language(begin_line: &str) -> Option<&str> {
        // \begin{lstlisting}[language=Python] or \begin{minted}{python}
        if let Some(lang_pos) = begin_line.find("language=") {
            let after = &begin_line[lang_pos + 9..];
            let end = after.find([',', ']', '}']).unwrap_or(after.len());
            let lang = after[..end].trim();
            if !lang.is_empty() {
                return Some(lang);
            }
        }
        // \begin{minted}{python}
        if begin_line.contains("minted")
            && let Some(brace_start) = begin_line.rfind('{')
        {
            let after = &begin_line[brace_start + 1..];
            if let Some(brace_end) = after.find('}') {
                let lang = after[..brace_end].trim();
                if !lang.is_empty() && lang != "minted" {
                    return Some(lang);
                }
            }
        }
        None
    }
}

impl Default for LatexExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for LatexExtractor {
    fn name(&self) -> &str {
        "latex-extractor"
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    fn description(&self) -> &str {
        "Native Rust LaTeX document extractor with metadata and table support"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for LatexExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let latex_str = String::from_utf8_lossy(content).to_string();
        let (text, metadata, tables) = Self::extract_from_latex(&latex_str);
        let document = if config.include_document_structure {
            Some(Self::build_document_structure(&latex_str))
        } else {
            None
        };

        Ok(ExtractionResult {
            content: text,
            mime_type: mime_type.to_string().into(),
            metadata,
            tables,
            detected_languages: None,
            chunks: None,
            images: None,
            djot_content: None,
            pages: None,
            elements: None,
            ocr_elements: None,
            document,
            #[cfg(any(feature = "keywords-yake", feature = "keywords-rake"))]
            extracted_keywords: None,
            quality_score: None,
            processing_warnings: Vec::new(),
            annotations: None,
            children: None,
        })
    }

    fn supported_mime_types(&self) -> &[&str] {
        &["application/x-latex", "text/x-tex"]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NodeContent;

    #[test]
    fn test_basic_title_extraction() {
        let latex = r#"\title{Hello World}"#;
        let (_, metadata, _) = LatexExtractor::extract_from_latex(latex);
        assert_eq!(
            metadata.additional.get("title").and_then(|v| v.as_str()),
            Some("Hello World")
        );
    }

    #[test]
    fn test_author_extraction() {
        let latex = r#"\author{John Doe}"#;
        let (_, metadata, _) = LatexExtractor::extract_from_latex(latex);
        assert!(metadata.additional.contains_key("author"));
    }

    #[test]
    fn test_section_extraction() {
        let latex = r#"\begin{document}\section{Introduction}\end{document}"#;
        let (content, _, _) = LatexExtractor::extract_from_latex(latex);
        assert!(content.contains("Introduction"));
    }

    #[test]
    fn test_strip_inline_bold() {
        let (text, anns) = LatexExtractor::strip_inline_commands("hello \\textbf{world} end");
        assert_eq!(text, "hello world end");
        assert_eq!(anns.len(), 1);
        assert!(matches!(anns[0].kind, AnnotationKind::Bold));
        assert_eq!(&text[anns[0].start as usize..anns[0].end as usize], "world");
    }

    #[test]
    fn test_strip_inline_italic_variants() {
        let (text, anns) = LatexExtractor::strip_inline_commands("\\emph{a} and \\textit{b}");
        assert_eq!(text, "a and b");
        assert_eq!(anns.len(), 2);
        assert!(anns.iter().all(|a| matches!(a.kind, AnnotationKind::Italic)));
    }

    #[test]
    fn test_strip_inline_underline_code() {
        let (text, anns) = LatexExtractor::strip_inline_commands("\\underline{u} \\texttt{c}");
        assert_eq!(text, "u c");
        assert!(anns.iter().any(|a| matches!(a.kind, AnnotationKind::Underline)));
        assert!(anns.iter().any(|a| matches!(a.kind, AnnotationKind::Code)));
    }

    #[test]
    fn test_strip_inline_nested() {
        let (text, anns) = LatexExtractor::strip_inline_commands("\\textbf{\\emph{nested}}");
        assert_eq!(text, "nested");
        assert_eq!(anns.len(), 2);
        // Both annotations should cover the same range
        assert!(anns.iter().any(|a| matches!(a.kind, AnnotationKind::Bold)));
        assert!(anns.iter().any(|a| matches!(a.kind, AnnotationKind::Italic)));
    }

    #[test]
    fn test_strip_inline_href() {
        let (text, anns) = LatexExtractor::strip_inline_commands("see \\href{https://example.com}{link text} here");
        assert_eq!(text, "see link text here");
        assert_eq!(anns.len(), 1);
        match &anns[0].kind {
            AnnotationKind::Link { url, .. } => assert_eq!(url, "https://example.com"),
            _ => panic!("expected Link annotation"),
        }
        assert_eq!(&text[anns[0].start as usize..anns[0].end as usize], "link text");
    }

    #[test]
    fn test_strip_no_commands() {
        let (text, anns) = LatexExtractor::strip_inline_commands("plain text only");
        assert_eq!(text, "plain text only");
        assert!(anns.is_empty());
    }

    #[test]
    fn test_extract_includegraphics_path() {
        assert_eq!(
            LatexExtractor::extract_includegraphics_path("\\includegraphics[width=5cm]{img/photo.png}"),
            Some("img/photo.png".to_string())
        );
        assert_eq!(
            LatexExtractor::extract_includegraphics_path("\\includegraphics{simple.jpg}"),
            Some("simple.jpg".to_string())
        );
        assert_eq!(LatexExtractor::extract_includegraphics_path("no graphics here"), None);
    }

    #[test]
    fn test_extract_caption() {
        assert_eq!(
            LatexExtractor::extract_caption("\\caption{A nice figure}"),
            Some("A nice figure".to_string())
        );
        assert_eq!(LatexExtractor::extract_caption("no caption"), None);
    }

    #[test]
    fn test_read_braced_content_nested() {
        let (content, consumed) = LatexExtractor::read_braced_content("outer {inner} end}rest").unwrap();
        assert_eq!(content, "outer {inner} end");
        assert_eq!(&"outer {inner} end}rest"[consumed..], "rest");
    }

    #[test]
    fn test_build_document_structure_with_metadata() {
        let latex = r"\title{Test}
\author{Author}
\date{2024}
\begin{document}
Hello.
\end{document}";
        let doc = LatexExtractor::build_document_structure(latex);
        assert!(doc.validate().is_ok());
        let meta = doc.nodes.iter().find(|n| {
            matches!(&n.content, NodeContent::MetadataBlock { entries } if entries.iter().any(|(k, _)| k == "title"))
        });
        assert!(meta.is_some(), "should have metadata block");
    }

    #[test]
    fn test_build_document_structure_with_footnote() {
        let latex = r"\begin{document}
Text with\footnote{A note} more.
\end{document}";
        let doc = LatexExtractor::build_document_structure(latex);
        assert!(doc.validate().is_ok());
        let has_footnote = doc
            .nodes
            .iter()
            .any(|n| matches!(&n.content, NodeContent::Footnote { text } if text.contains("A note")));
        assert!(has_footnote);
    }

    #[test]
    fn test_build_document_structure_with_figure() {
        let latex = r"\begin{document}
\begin{figure}
\includegraphics{img.png}
\caption{My caption}
\end{figure}
\end{document}";
        let doc = LatexExtractor::build_document_structure(latex);
        assert!(doc.validate().is_ok());
        let img = doc
            .nodes
            .iter()
            .find(|n| matches!(&n.content, NodeContent::Image { .. }));
        assert!(img.is_some(), "should have image node");
        let img = img.unwrap();
        match &img.content {
            NodeContent::Image { description, .. } => {
                assert_eq!(description.as_deref(), Some("img.png"));
            }
            _ => unreachable!(),
        }
        assert_eq!(
            img.attributes
                .as_ref()
                .and_then(|a| a.get("caption"))
                .map(|s| s.as_str()),
            Some("My caption")
        );
    }

    #[test]
    fn test_build_document_structure_inline_annotations() {
        let latex = r"\begin{document}
This is \textbf{bold} and \emph{italic}.
\end{document}";
        let doc = LatexExtractor::build_document_structure(latex);
        assert!(doc.validate().is_ok());
        let para = doc
            .nodes
            .iter()
            .find(|n| matches!(&n.content, NodeContent::Paragraph { text } if text.contains("bold")))
            .expect("should have paragraph");
        assert!(!para.annotations.is_empty(), "should have annotations");
        assert!(para.annotations.iter().any(|a| matches!(a.kind, AnnotationKind::Bold)));
        assert!(
            para.annotations
                .iter()
                .any(|a| matches!(a.kind, AnnotationKind::Italic))
        );
    }
}
