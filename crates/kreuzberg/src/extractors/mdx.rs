//! MDX extractor with JSX stripping and frontmatter support.
//!
//! MDX is a superset of Markdown that adds JSX support (imports, exports,
//! JSX components, and inline expressions). This extractor strips MDX-specific
//! syntax and then processes the remaining content as standard Markdown.
//!
//! Requires the `mdx` feature (which includes `pulldown-cmark`).

use super::frontmatter_utils::{
    cells_to_markdown, extract_frontmatter, extract_metadata_from_yaml, extract_title_from_content,
};
use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::builder::DocumentStructureBuilder;
use crate::types::document_structure::DocumentStructure;
use crate::types::{ExtractionResult, Metadata, Table};
use async_trait::async_trait;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;

/// Annotation tracking entry: (kind_tag, byte_start, optional link data).
///
/// kind_tag: 0=bold, 1=italic, 2=strikethrough, 3=code, 4=link
type AnnotationEntry = (u8, u32, Option<(String, Option<String>)>);

/// Regex matching JSX component tags (capitalized tag names).
/// Matches opening tags like `<Component prop="value">`, closing tags like `</Component>`,
/// and self-closing tags like `<Component />`.
static JSX_TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"</?[A-Z][a-zA-Z0-9_.]*(?:\s[^>]*)?>|<[A-Z][a-zA-Z0-9_.]*(?:\s[^>]*)?\s*/>").unwrap());

/// Regex matching standalone JSX expression lines like `{expression}` or `{/* comment */}`.
static JSX_EXPR_LINE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*\{.*\}\s*$").unwrap());

/// Regex matching inline JSX comments like `{/* ... */}`.
static JSX_INLINE_COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\{/\*.*?\*/\}").unwrap());

/// MDX extractor with JSX stripping and Markdown processing.
///
/// Strips MDX-specific syntax (imports, exports, JSX component tags,
/// inline expressions) and processes the remaining content as Markdown,
/// extracting metadata from YAML frontmatter and tables.
pub struct MdxExtractor;

impl MdxExtractor {
    /// Create a new MDX extractor.
    pub fn new() -> Self {
        Self
    }

    /// Strip MDX-specific syntax from content, preserving standard Markdown.
    ///
    /// Removes:
    /// - `import` statements (single and multi-line)
    /// - `export` statements (single and multi-line)
    /// - JSX component tags (capitalized: `<Component>`, `</Component>`, `<Component />`)
    /// - Standalone JSX expression lines (`{expression}`, `{/* comment */}`)
    ///
    /// Preserves:
    /// - Content inside code fences (``` blocks)
    /// - Standard HTML tags (lowercase: `<div>`, `<p>`, etc.)
    /// - Text content between JSX component tags
    #[cfg(test)]
    pub(crate) fn strip_mdx_syntax(content: &str) -> String {
        Self::strip_mdx_syntax_collecting(content, None)
    }

    /// Strip MDX syntax, optionally collecting stripped JSX blocks into `jsx_blocks`.
    fn strip_mdx_syntax_collecting(content: &str, mut jsx_blocks: Option<&mut Vec<String>>) -> String {
        let mut result = String::with_capacity(content.len());
        let mut in_code_fence = false;
        let mut skip_block_depth: i32 = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Track code fences - toggle on ``` lines
            if trimmed.starts_with("```") {
                in_code_fence = !in_code_fence;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            // Inside code fence - preserve everything
            if in_code_fence {
                result.push_str(line);
                result.push('\n');
                continue;
            }

            // Handle multi-line import/export blocks (tracking brace depth)
            if skip_block_depth > 0 {
                skip_block_depth += count_braces(trimmed);
                if skip_block_depth <= 0 {
                    skip_block_depth = 0;
                }
                continue;
            }

            // Skip import statements
            if trimmed.starts_with("import ") || trimmed == "import" {
                let depth = count_braces(trimmed);
                if depth > 0 {
                    skip_block_depth = depth;
                }
                continue;
            }

            // Skip export statements
            if trimmed.starts_with("export ") || trimmed == "export" {
                let depth = count_braces(trimmed);
                if depth > 0 {
                    skip_block_depth = depth;
                }
                continue;
            }

            // Skip standalone JSX expression lines ({...} on own line)
            if JSX_EXPR_LINE_RE.is_match(trimmed) {
                continue;
            }

            // Strip inline JSX comments like {/* ... */}
            let without_comments = JSX_INLINE_COMMENT_RE.replace_all(line, "");

            // Strip JSX component tags from the line, keeping text content
            let processed = JSX_TAG_RE.replace_all(&without_comments, "");
            let processed_trimmed = processed.trim();

            // Lines that became empty after stripping were pure JSX tag lines
            if processed_trimmed.is_empty() && !trimmed.is_empty() {
                // Collect stripped JSX tags if requested
                if let Some(ref mut blocks) = jsx_blocks {
                    let tags: Vec<String> = JSX_TAG_RE
                        .find_iter(&without_comments)
                        .map(|m| m.as_str().to_string())
                        .collect();
                    for tag in tags {
                        blocks.push(tag);
                    }
                }
                continue;
            }

            result.push_str(&processed);
            result.push('\n');
        }

        result
    }

    /// Build a `DocumentStructure` from pulldown-cmark events after JSX stripping.
    ///
    /// JSX blocks that were stripped are recorded as raw blocks in the document structure.
    fn build_document_structure(
        events: &[Event],
        yaml: &Option<serde_yaml_ng::Value>,
        raw_jsx_blocks: &[String],
    ) -> DocumentStructure {
        let mut builder = DocumentStructureBuilder::new().source_format("mdx");

        // Emit frontmatter as a metadata block
        if let Some(serde_yaml_ng::Value::Mapping(map)) = yaml {
            let entries: Vec<(String, String)> = map
                .iter()
                .filter_map(|(k, v)| {
                    let key = k.as_str()?.to_string();
                    let val = match v {
                        serde_yaml_ng::Value::String(s) => s.clone(),
                        other => format!("{other:?}"),
                    };
                    Some((key, val))
                })
                .collect();
            if !entries.is_empty() {
                builder.push_metadata_block(entries, None);
            }
        }

        // Emit stripped JSX components as raw blocks
        for jsx in raw_jsx_blocks {
            if !jsx.trim().is_empty() {
                builder.push_raw_block("jsx", jsx, None);
            }
        }

        Self::walk_events_into_builder(events, &mut builder);
        builder.build()
    }

    /// Walk pulldown-cmark events and push nodes into the builder.
    fn walk_events_into_builder(events: &[Event], builder: &mut DocumentStructureBuilder) {
        use crate::types::builder;
        use crate::types::document_structure::TextAnnotation;

        let mut paragraph_text = String::new();
        let mut paragraph_annotations: Vec<TextAnnotation> = Vec::new();
        let mut in_paragraph = false;
        let mut heading_text = String::new();
        let mut heading_level: u8 = 0;
        let mut in_heading = false;
        let mut code_text = String::new();
        let mut code_lang: Option<String> = None;
        let mut in_code_block = false;
        let mut blockquote_depth: u32 = 0;
        let mut table_rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();
        let mut in_table_cell = false;
        let mut list_stack: Vec<(crate::types::document_structure::NodeIndex, bool)> = Vec::new();
        let mut list_item_text = String::new();
        let mut in_list_item = false;
        let mut in_image = false;
        let mut image_alt = String::new();

        // Annotation tracking: stack of (kind_tag, byte_start, optional link data).
        // kind_tag: 0=bold, 1=italic, 2=strikethrough, 3=code, 4=link
        let mut annotation_starts: Vec<AnnotationEntry> = Vec::new();

        fn text_offset(paragraph_text: &str, in_paragraph: bool) -> u32 {
            if in_paragraph { paragraph_text.len() as u32 } else { 0 }
        }

        for event in events {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    heading_text.clear();
                    heading_level = match *level {
                        pulldown_cmark::HeadingLevel::H1 => 1,
                        pulldown_cmark::HeadingLevel::H2 => 2,
                        pulldown_cmark::HeadingLevel::H3 => 3,
                        pulldown_cmark::HeadingLevel::H4 => 4,
                        pulldown_cmark::HeadingLevel::H5 => 5,
                        pulldown_cmark::HeadingLevel::H6 => 6,
                    };
                    in_heading = true;
                }
                Event::End(TagEnd::Heading(_)) => {
                    in_heading = false;
                    let trimmed = heading_text.trim();
                    if !trimmed.is_empty() {
                        builder.push_heading(heading_level, trimmed, None, None);
                    }
                    heading_text.clear();
                }
                Event::Start(Tag::Paragraph) => {
                    if !in_heading && !in_list_item {
                        paragraph_text.clear();
                        paragraph_annotations.clear();
                        in_paragraph = true;
                    }
                }
                Event::End(TagEnd::Paragraph) => {
                    if in_paragraph {
                        in_paragraph = false;
                        let trimmed = paragraph_text.trim();
                        if !trimmed.is_empty() {
                            let trim_offset = paragraph_text.len() - paragraph_text.trim_start().len();
                            let trimmed_len = trimmed.len() as u32;
                            let annotations = if trim_offset > 0 {
                                paragraph_annotations
                                    .drain(..)
                                    .map(|mut a| {
                                        a.start = a.start.saturating_sub(trim_offset as u32);
                                        a.end = a.end.saturating_sub(trim_offset as u32);
                                        a
                                    })
                                    .filter(|a| a.start < a.end && a.end <= trimmed_len)
                                    .collect()
                            } else {
                                paragraph_annotations
                                    .drain(..)
                                    .filter(|a| a.start < a.end && a.end <= trimmed_len)
                                    .collect()
                            };
                            builder.push_paragraph(trimmed, annotations, None, None);
                        }
                        paragraph_text.clear();
                        paragraph_annotations.clear();
                    }
                }
                // Inline formatting — annotation tracking
                Event::Start(Tag::Strong) => {
                    if in_paragraph {
                        annotation_starts.push((0, text_offset(&paragraph_text, in_paragraph), None));
                    }
                }
                Event::End(TagEnd::Strong) => {
                    if in_paragraph
                        && let Some((0, start, _)) = annotation_starts
                            .iter()
                            .rposition(|(k, _, _)| *k == 0)
                            .map(|i| annotation_starts.remove(i))
                    {
                        let end = text_offset(&paragraph_text, in_paragraph);
                        if start < end {
                            paragraph_annotations.push(builder::bold(start, end));
                        }
                    }
                }
                Event::Start(Tag::Emphasis) => {
                    if in_paragraph {
                        annotation_starts.push((1, text_offset(&paragraph_text, in_paragraph), None));
                    }
                }
                Event::End(TagEnd::Emphasis) => {
                    if in_paragraph
                        && let Some((1, start, _)) = annotation_starts
                            .iter()
                            .rposition(|(k, _, _)| *k == 1)
                            .map(|i| annotation_starts.remove(i))
                    {
                        let end = text_offset(&paragraph_text, in_paragraph);
                        if start < end {
                            paragraph_annotations.push(builder::italic(start, end));
                        }
                    }
                }
                Event::Start(Tag::Strikethrough) => {
                    if in_paragraph {
                        annotation_starts.push((2, text_offset(&paragraph_text, in_paragraph), None));
                    }
                }
                Event::End(TagEnd::Strikethrough) => {
                    if in_paragraph
                        && let Some((2, start, _)) = annotation_starts
                            .iter()
                            .rposition(|(k, _, _)| *k == 2)
                            .map(|i| annotation_starts.remove(i))
                    {
                        let end = text_offset(&paragraph_text, in_paragraph);
                        if start < end {
                            paragraph_annotations.push(builder::strikethrough(start, end));
                        }
                    }
                }
                Event::Start(Tag::Link { dest_url, title, .. }) => {
                    if in_paragraph {
                        let url = dest_url.to_string();
                        let title_opt = if title.is_empty() {
                            None
                        } else {
                            Some(title.to_string())
                        };
                        annotation_starts.push((4, text_offset(&paragraph_text, in_paragraph), Some((url, title_opt))));
                    }
                }
                Event::End(TagEnd::Link) => {
                    if in_paragraph
                        && let Some((4, start, Some((url, title)))) = annotation_starts
                            .iter()
                            .rposition(|(k, _, _)| *k == 4)
                            .map(|i| annotation_starts.remove(i))
                    {
                        let end = text_offset(&paragraph_text, in_paragraph);
                        if start < end {
                            paragraph_annotations.push(builder::link(start, end, &url, title.as_deref()));
                        }
                    }
                }
                Event::Start(Tag::CodeBlock(pulldown_cmark::CodeBlockKind::Fenced(lang))) => {
                    code_text.clear();
                    code_lang = if lang.is_empty() { None } else { Some(lang.to_string()) };
                    in_code_block = true;
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    code_text.clear();
                    code_lang = None;
                    in_code_block = true;
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    let trimmed = code_text.trim_end();
                    if !trimmed.is_empty() {
                        builder.push_code(trimmed, code_lang.as_deref(), None);
                    }
                    code_text.clear();
                    code_lang = None;
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    builder.push_quote(None);
                    blockquote_depth += 1;
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if blockquote_depth > 0 {
                        builder.exit_container();
                        blockquote_depth -= 1;
                    }
                }
                Event::Start(Tag::List(start)) => {
                    let ordered = start.is_some();
                    let list_idx = builder.push_list(ordered, None);
                    list_stack.push((list_idx, ordered));
                }
                Event::End(TagEnd::List(_)) => {
                    list_stack.pop();
                }
                Event::Start(Tag::Item) => {
                    list_item_text.clear();
                    in_list_item = true;
                }
                Event::End(TagEnd::Item) => {
                    in_list_item = false;
                    let trimmed = list_item_text.trim();
                    if let Some((list_idx, _)) = list_stack.last()
                        && !trimmed.is_empty()
                    {
                        builder.push_list_item(*list_idx, trimmed, None);
                    }
                    list_item_text.clear();
                }
                Event::Start(Tag::Table(_)) => {
                    table_rows.clear();
                }
                Event::End(TagEnd::Table) => {
                    if !table_rows.is_empty() {
                        builder.push_table_from_cells(&table_rows, None);
                    }
                    table_rows.clear();
                }
                Event::Start(Tag::TableHead | Tag::TableRow) => {
                    current_row.clear();
                }
                Event::End(TagEnd::TableHead | TagEnd::TableRow) => {
                    if !current_row.is_empty() {
                        table_rows.push(std::mem::take(&mut current_row));
                    }
                }
                Event::Start(Tag::TableCell) => {
                    current_cell.clear();
                    in_table_cell = true;
                }
                Event::End(TagEnd::TableCell) => {
                    in_table_cell = false;
                    current_row.push(current_cell.trim().to_string());
                    current_cell.clear();
                }
                Event::Start(Tag::Image { .. }) => {
                    in_image = true;
                    image_alt.clear();
                }
                Event::End(TagEnd::Image) => {
                    in_image = false;
                    let trimmed = image_alt.trim();
                    let desc = if trimmed.is_empty() { None } else { Some(trimmed) };
                    builder.push_image(desc, None, None, None);
                    image_alt.clear();
                }
                Event::Code(s) => {
                    if in_code_block {
                        code_text.push_str(s);
                    } else if in_heading {
                        heading_text.push_str(s);
                    } else if in_image {
                        image_alt.push_str(s);
                    } else if in_table_cell {
                        current_cell.push_str(s);
                    } else if in_list_item {
                        list_item_text.push_str(s);
                    } else if in_paragraph {
                        let start = paragraph_text.len() as u32;
                        paragraph_text.push_str(s);
                        let end = paragraph_text.len() as u32;
                        if start < end {
                            paragraph_annotations.push(builder::code(start, end));
                        }
                    }
                }
                Event::Text(s) => {
                    if in_code_block {
                        code_text.push_str(s);
                    } else if in_heading {
                        heading_text.push_str(s);
                    } else if in_image {
                        image_alt.push_str(s);
                    } else if in_table_cell {
                        current_cell.push_str(s);
                    } else if in_list_item {
                        list_item_text.push_str(s);
                    } else if in_paragraph {
                        paragraph_text.push_str(s);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if in_code_block {
                        code_text.push('\n');
                    } else if in_heading {
                        heading_text.push(' ');
                    } else if in_list_item {
                        list_item_text.push(' ');
                    } else if in_paragraph {
                        paragraph_text.push(' ');
                    }
                }
                Event::FootnoteReference(name) => {
                    builder.push_footnote(name, None);
                }
                Event::Html(s) => {
                    if in_paragraph {
                        paragraph_text.push_str(s);
                    }
                }
                Event::TaskListMarker(checked) => {
                    if in_list_item {
                        list_item_text.push_str(if *checked { "[x] " } else { "[ ] " });
                    }
                }
                _ => {}
            }
        }
    }

    /// Extract tables from markdown AST.
    fn extract_tables_from_events(events: &[Event]) -> Vec<Table> {
        let mut tables = Vec::new();
        let mut current_table: Option<(Vec<Vec<String>>, usize)> = None;
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();
        let mut in_table_cell = false;
        let mut table_index = 0;

        for event in events {
            match event {
                Event::Start(Tag::Table(_)) => {
                    current_table = Some((Vec::new(), table_index));
                }
                Event::Start(Tag::TableHead) => {}
                Event::Start(Tag::TableRow) => {
                    current_row = Vec::new();
                }
                Event::Start(Tag::TableCell) => {
                    current_cell = String::new();
                    in_table_cell = true;
                }
                Event::Text(s) if in_table_cell => {
                    current_cell.push_str(s);
                }
                Event::Code(s) if in_table_cell => {
                    current_cell.push_str(s);
                }
                Event::End(TagEnd::TableCell) if in_table_cell => {
                    current_row.push(current_cell.trim().to_string());
                    current_cell = String::new();
                    in_table_cell = false;
                }
                Event::End(TagEnd::TableHead) => {
                    if !current_row.is_empty()
                        && let Some((ref mut rows, _)) = current_table
                    {
                        rows.push(std::mem::take(&mut current_row));
                    }
                    current_row = Vec::new();
                }
                Event::End(TagEnd::TableRow) => {
                    if !current_row.is_empty()
                        && let Some((ref mut rows, _)) = current_table
                    {
                        rows.push(std::mem::take(&mut current_row));
                    }
                    current_row = Vec::new();
                }
                Event::End(TagEnd::Table) => {
                    if let Some((cells, idx)) = current_table.take()
                        && !cells.is_empty()
                    {
                        let markdown = cells_to_markdown(&cells);
                        tables.push(Table {
                            cells,
                            markdown,
                            page_number: idx + 1,
                            bounding_box: None,
                        });
                        table_index += 1;
                    }
                }
                _ => {}
            }
        }

        tables
    }
}

/// Count net brace depth change in a line (opening `{` minus closing `}`).
fn count_braces(line: &str) -> i32 {
    let mut depth: i32 = 0;
    for ch in line.chars() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {}
        }
    }
    depth
}

impl Default for MdxExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for MdxExtractor {
    fn name(&self) -> &str {
        "mdx-extractor"
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
        "Extracts content from MDX files by stripping JSX syntax and processing as Markdown"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for MdxExtractor {
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let text = String::from_utf8_lossy(content).into_owned();

        // Extract frontmatter first (before stripping MDX syntax)
        let (yaml, remaining_content) = extract_frontmatter(&text);

        let mut metadata = if let Some(ref yaml_value) = yaml {
            extract_metadata_from_yaml(yaml_value)
        } else {
            Metadata::default()
        };

        // Strip MDX-specific syntax from the remaining content,
        // collecting JSX blocks when document structure is requested.
        let mut jsx_blocks_buf = if config.include_document_structure {
            Some(Vec::new())
        } else {
            None
        };
        let clean_markdown = Self::strip_mdx_syntax_collecting(&remaining_content, jsx_blocks_buf.as_mut());

        if metadata.title.is_none()
            && !metadata.additional.contains_key("title")
            && let Some(title) = extract_title_from_content(&clean_markdown)
        {
            metadata.title = Some(title.clone());
            // DEPRECATED: kept for backward compatibility; will be removed in next major version.
            metadata.additional.insert(Cow::Borrowed("title"), title.into());
        }

        let mut options = Options::ENABLE_TABLES;
        if config.include_document_structure {
            options |= Options::ENABLE_STRIKETHROUGH | Options::ENABLE_FOOTNOTES;
        }
        let parser = Parser::new_ext(&clean_markdown, options);
        let events: Vec<Event> = parser.collect();

        let mut extracted_images = Vec::new();
        let extracted_text = super::markdown_utils::extract_text_from_events(&events, &mut extracted_images);

        let tables = Self::extract_tables_from_events(&events);

        let document = if config.include_document_structure {
            let raw_jsx = jsx_blocks_buf.unwrap_or_default();
            Some(Self::build_document_structure(&events, &yaml, &raw_jsx))
        } else {
            None
        };

        let images = if !extracted_images.is_empty() {
            #[cfg(all(feature = "ocr", feature = "tokio-runtime"))]
            {
                let processed = crate::extraction::image_ocr::process_images_with_ocr(extracted_images, config).await?;
                Some(processed)
            }
            #[cfg(not(all(feature = "ocr", feature = "tokio-runtime")))]
            {
                Some(extracted_images)
            }
        } else {
            None
        };

        Ok(ExtractionResult {
            content: extracted_text,
            mime_type: mime_type.to_string().into(),
            metadata,
            tables,
            detected_languages: None,
            chunks: None,
            images,
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
        &["text/mdx", "text/x-mdx"]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── strip_mdx_syntax tests ──────────────────────────────────────────

    #[test]
    fn test_strip_import_statements() {
        let input = "import { Chart } from './Chart'\nimport Alert from './Alert'\n\n# Hello\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("import"));
        assert!(result.contains("# Hello"));
    }

    #[test]
    fn test_strip_multiline_import() {
        let input = "import {\n  Chart,\n  Table,\n} from './components'\n\n# Hello\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("import"));
        assert!(!result.contains("Chart"));
        assert!(result.contains("# Hello"));
    }

    #[test]
    fn test_strip_export_statements() {
        let input = "export const meta = { title: 'Hello' }\n\n# Hello\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("export"));
        assert!(result.contains("# Hello"));
    }

    #[test]
    fn test_strip_multiline_export() {
        let input = "export const meta = {\n  title: 'Hello',\n  date: '2024-01-01',\n}\n\n# Hello\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("export"));
        assert!(!result.contains("title"));
        assert!(result.contains("# Hello"));
    }

    #[test]
    fn test_strip_export_default() {
        let input = "export default function Layout({ children }) { return children }\n\n# Hello\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("export"));
        assert!(result.contains("# Hello"));
    }

    #[test]
    fn test_strip_jsx_component_tags() {
        let input = "# Hello\n\n<Alert type=\"warning\">\nBe careful!\n</Alert>\n\nMore text.\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("<Alert"));
        assert!(!result.contains("</Alert>"));
        assert!(result.contains("Be careful!"));
        assert!(result.contains("More text."));
    }

    #[test]
    fn test_strip_self_closing_jsx() {
        let input = "# Hello\n\n<Chart data={data} />\n\nSome text.\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("<Chart"));
        assert!(result.contains("Some text."));
    }

    #[test]
    fn test_strip_jsx_expression_lines() {
        let input = "# Hello\n\n{/* This is a comment */}\n\n{someExpression}\n\nText.\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("comment"));
        assert!(!result.contains("someExpression"));
        assert!(result.contains("Text."));
    }

    #[test]
    fn test_preserve_code_fences() {
        let input =
            "# Hello\n\n```jsx\nimport React from 'react'\nconst x = <Component />\nexport default App\n```\n\nText.\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(result.contains("import React from 'react'"));
        assert!(result.contains("<Component />"));
        assert!(result.contains("export default App"));
        assert!(result.contains("Text."));
    }

    #[test]
    fn test_preserve_standard_html_tags() {
        let input = "# Hello\n\n<div>Some content</div>\n\n<p>Paragraph</p>\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(result.contains("<div>"));
        assert!(result.contains("</div>"));
        assert!(result.contains("<p>"));
    }

    #[test]
    fn test_preserve_markdown_content() {
        let input = "# Title\n\nThis is **bold** and *italic* text.\n\n- Item 1\n- Item 2\n\n> Blockquote\n";
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(result.contains("# Title"));
        assert!(result.contains("**bold**"));
        assert!(result.contains("*italic*"));
        assert!(result.contains("- Item 1"));
        assert!(result.contains("> Blockquote"));
    }

    #[test]
    fn test_strip_complex_mdx() {
        let input = r#"import { Chart } from './Chart'
import Alert from './Alert'

export const meta = {
  title: 'My Post',
  date: '2024-01-01',
}

# My Post

This is a paragraph with **bold** text.

<Alert type="warning">
  Be careful with this!
</Alert>

<Chart data={data} />

{/* A comment */}

Some more text.

```javascript
const x = <div>Not JSX</div>
```

Final paragraph.
"#;
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("import"));
        assert!(!result.contains("export"));
        assert!(!result.contains("<Alert"));
        assert!(!result.contains("<Chart"));
        assert!(!result.contains("comment"));
        assert!(result.contains("# My Post"));
        assert!(result.contains("**bold**"));
        assert!(result.contains("Be careful with this!"));
        assert!(result.contains("Some more text."));
        assert!(result.contains("const x = <div>Not JSX</div>"));
        assert!(result.contains("Final paragraph."));
    }

    #[test]
    fn test_empty_content() {
        let result = MdxExtractor::strip_mdx_syntax("");
        assert!(result.is_empty());
    }

    // ── Full extraction tests ───────────────────────────────────────────

    #[test]
    fn test_plugin_interface() {
        let extractor = MdxExtractor::new();
        assert_eq!(extractor.name(), "mdx-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(extractor.priority(), 50);
        assert_eq!(extractor.supported_mime_types(), &["text/mdx", "text/x-mdx"]);
    }

    #[tokio::test]
    async fn test_extract_mdx_basic() {
        let content = b"import Chart from './Chart'\n\n# Hello World\n\nThis is content.\n";
        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(content, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should extract MDX content");

        assert_eq!(result.mime_type, "text/mdx");
        assert!(result.content.contains("Hello World"));
        assert!(result.content.contains("This is content"));
        assert!(!result.content.contains("import"));
    }

    #[tokio::test]
    async fn test_extract_mdx_with_frontmatter() {
        let content = b"---\ntitle: My MDX Post\nauthor: Test Author\ndate: 2024-01-15\n---\n\nimport Alert from './Alert'\n\n# Content\n\nBody text.\n";
        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(content, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should extract MDX with frontmatter");

        assert_eq!(
            result.metadata.additional.get("title").and_then(|v| v.as_str()),
            Some("My MDX Post")
        );
        assert_eq!(
            result.metadata.additional.get("author").and_then(|v| v.as_str()),
            Some("Test Author")
        );
        assert!(result.content.contains("Body text"));
        assert!(!result.content.contains("import"));
    }

    #[tokio::test]
    async fn test_extract_mdx_with_jsx_components() {
        let content = b"# Title\n\n<Alert type=\"warning\">\nImportant message!\n</Alert>\n\nRegular text.\n";
        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(content, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should extract MDX with JSX components");

        assert!(result.content.contains("Important message"));
        assert!(result.content.contains("Regular text"));
        assert!(!result.content.contains("<Alert"));
    }

    #[tokio::test]
    async fn test_extract_mdx_with_tables() {
        let content = b"# Tables\n\n| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |\n";
        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(content, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should extract MDX with tables");

        assert!(!result.tables.is_empty());
        let table = &result.tables[0];
        assert_eq!(table.cells[0].len(), 2);
    }

    #[tokio::test]
    async fn test_extract_mdx_title_from_heading() {
        let content = b"# My Document Title\n\nContent here.\n";
        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(content, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should extract title from heading");

        assert_eq!(result.metadata.title, Some("My Document Title".to_string()));
    }

    // ── count_braces tests ──────────────────────────────────────────────

    #[test]
    fn test_count_braces_balanced() {
        assert_eq!(count_braces("{ a: 1 }"), 0);
    }

    #[test]
    fn test_count_braces_opening() {
        assert_eq!(count_braces("const x = {"), 1);
    }

    #[test]
    fn test_count_braces_closing() {
        assert_eq!(count_braces("}"), -1);
    }

    #[test]
    fn test_count_braces_nested() {
        assert_eq!(count_braces("{ a: { b: 1 }"), 1);
    }

    #[test]
    fn test_count_braces_none() {
        assert_eq!(count_braces("no braces here"), 0);
    }

    // ── Real-world MDX file integration tests ────────────────────────────

    /// Helper: load a test document from the test_documents directory.
    fn load_test_doc(relative_path: &str) -> Vec<u8> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("test_documents")
            .join(relative_path);
        std::fs::read(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
    }

    #[tokio::test]
    async fn test_extract_real_world_getting_started() {
        let content = load_test_doc("markdown/mdx_getting_started.mdx");
        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(&content, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should extract getting-started.mdx");

        // Should extract the main heading
        assert!(result.content.contains("Getting started"), "Missing main heading");

        // Should contain real prose content
        assert!(
            result.content.contains("how to integrate MDX into your project"),
            "Missing introductory text"
        );

        // Sections should be present
        assert!(
            result.content.contains("Prerequisites"),
            "Missing Prerequisites section"
        );
        assert!(result.content.contains("Quick start"), "Missing Quick start section");
        assert!(result.content.contains("Bundler"), "Missing Bundler section");
        assert!(result.content.contains("Security"), "Missing Security section");
        assert!(result.content.contains("Integrations"), "Missing Integrations section");

        // Framework names should appear in prose
        assert!(result.content.contains("React"), "Missing React mention");
        assert!(result.content.contains("webpack"), "Missing webpack mention");
        assert!(result.content.contains("esbuild"), "Missing esbuild mention");

        // Import/export statements should be stripped
        assert!(
            !result.content.contains("import {Note}"),
            "import statement not stripped"
        );
        assert!(
            !result.content.contains("export const info"),
            "export const info not stripped"
        );
        assert!(
            !result.content.contains("export const navSortSelf"),
            "export const navSortSelf not stripped"
        );

        // JSX component tags should be stripped
        assert!(!result.content.contains("<Note type="), "<Note> JSX tags not stripped");

        // JSX comments should be stripped
        assert!(!result.content.contains("{/* more */}"), "JSX comment not stripped");

        // Code blocks should be preserved (content inside fences)
        assert!(
            result.content.contains("npm install @types/mdx"),
            "Code block content should be preserved"
        );

        // Substantial content length
        assert!(
            result.content.len() > 2000,
            "Extracted content too short: {} chars",
            result.content.len()
        );
    }

    #[tokio::test]
    async fn test_extract_real_world_using_mdx() {
        let content = load_test_doc("markdown/mdx_using_mdx.mdx");
        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(&content, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should extract using-mdx.mdx");

        // Main heading
        assert!(result.content.contains("Using MDX"), "Missing main heading");

        // Key sections
        assert!(
            result.content.contains("How MDX works"),
            "Missing 'How MDX works' section"
        );
        assert!(result.content.contains("MDX content"), "Missing 'MDX content' section");
        assert!(result.content.contains("Props"), "Missing Props section");
        assert!(result.content.contains("Components"), "Missing Components section");
        assert!(result.content.contains("Layout"), "Missing Layout section");
        assert!(result.content.contains("MDX provider"), "Missing MDX provider section");

        // Import/export stripped
        assert!(!result.content.contains("import {Note}"), "import not stripped");
        assert!(!result.content.contains("export const info"), "export not stripped");

        // JSX component tags stripped
        assert!(!result.content.contains("<Note type="), "<Note> tags not stripped");

        // Substantial content
        assert!(
            result.content.len() > 2000,
            "Extracted content too short: {} chars",
            result.content.len()
        );
    }

    #[tokio::test]
    async fn test_extract_real_world_troubleshooting() {
        let content = load_test_doc("markdown/mdx_troubleshooting.mdx");
        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(&content, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should extract troubleshooting-mdx.mdx");

        // Main heading
        assert!(result.content.contains("Troubleshooting MDX"), "Missing main heading");

        // Key error sections
        assert!(
            result.content.contains("Problems integrating MDX"),
            "Missing integrating section"
        );
        assert!(result.content.contains("ESM"), "Missing ESM section");
        assert!(result.content.contains("Problems using MDX"), "Missing using section");
        assert!(
            result.content.contains("Problems writing MDX"),
            "Missing writing section"
        );

        // Import/export stripped
        assert!(!result.content.contains("import {Note}"), "import not stripped");
        assert!(!result.content.contains("export const info"), "export not stripped");

        // JSX lint disable comment should be stripped
        assert!(!result.content.contains("{/* lint disable"), "JSX comment not stripped");

        // <Note> component tags stripped
        assert!(!result.content.contains("<Note type="), "<Note> tags not stripped");

        // Content inside Note components should be preserved
        assert!(
            result.content.contains("Had trouble with something"),
            "Content inside <Note> should be preserved"
        );

        // Substantial content
        assert!(
            result.content.len() > 2000,
            "Extracted content too short: {} chars",
            result.content.len()
        );
    }

    #[tokio::test]
    async fn test_strip_mdx_real_world_multiline_exports() {
        // Test the specific pattern from getting-started.mdx with nested Date objects
        let input = r#"import {Note} from '../_component/note.jsx'

export const info = {
  author: [
    {github: 'wooorm', name: 'Titus Wormer'}
  ],
  modified: new Date('2025-01-27'),
  published: new Date('2021-10-05')
}
export const navSortSelf = 2

# Getting started

Content here.
"#;
        let result = MdxExtractor::strip_mdx_syntax(input);
        assert!(!result.contains("import"), "import not stripped");
        assert!(!result.contains("export"), "export not stripped");
        assert!(!result.contains("wooorm"), "Nested export content not stripped");
        assert!(!result.contains("navSortSelf"), "Single-line export not stripped");
        assert!(result.contains("# Getting started"), "Heading should be preserved");
        assert!(result.contains("Content here"), "Content should be preserved");
    }

    #[tokio::test]
    async fn test_trimmed_paragraph_with_emoji_mdx() {
        let mdx = b"  **bold** \xf0\x9f\x8e\x89 text  ";

        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(mdx, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should handle emoji in trimmed MDX paragraph");

        assert!(result.content.contains("bold"), "Bold text preserved");
        assert!(result.content.contains("\u{1F389}"), "Emoji preserved after trim");
    }

    #[tokio::test]
    async fn test_cjk_paragraph_with_formatting_mdx() {
        let mdx = "# CJK\n\nこれは**太字**テスト".as_bytes();

        let extractor = MdxExtractor::new();
        let result = extractor
            .extract_bytes(mdx, "text/mdx", &ExtractionConfig::default())
            .await
            .expect("Should handle CJK with bold formatting");

        assert!(result.content.contains("太字"), "Bold CJK content present");
        assert!(result.content.contains("これは"), "Leading CJK preserved");
    }
}
