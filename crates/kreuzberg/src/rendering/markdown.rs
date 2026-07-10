//! Render an `InternalDocument` to GFM-compliant Markdown via comrak.

use comrak::{Arena, Options, format_commonmark};

use crate::types::internal::InternalDocument;

use super::comrak_bridge::build_comrak_ast;

/// Render an `InternalDocument` to GFM Markdown.
pub fn render_markdown(doc: &InternalDocument) -> String {
    tracing::debug!(element_count = doc.elements.len(), "markdown rendering starting");
    let arena = Arena::new();
    let root = build_comrak_ast(doc, &arena);

    if root.first_child().is_none() {
        tracing::debug!("markdown rendering: empty AST, returning empty string");
        return String::new();
    }

    let mut options = comrak_options();
    options.render.width = 0;

    let mut output = String::new();
    format_commonmark(root, &options, &mut output).expect("comrak formatting should not fail");

    if output.contains("<!--") {
        output = output
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("<!--") || !trimmed.ends_with("-->")
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    if output.contains("&#") {
        output = output.replace("&#10;", " ").replace("&#2;", "");
    }

    if output.contains("\\_") {
        output = output.replace("\\_", "_");
    }

    if output.contains("\\[") || output.contains("\\]") || output.contains("\\(") || output.contains("\\)") {
        output = output
            .replace("\\[", "[")
            .replace("\\]", "]")
            .replace("\\(", "(")
            .replace("\\)", ")");
    }

    if output.contains("\\*") || output.contains("\\#") {
        output = output
            .lines()
            .map(|line| {
                let trimmed = line.trim_start();
                if trimmed.starts_with("\\* ") || trimmed.starts_with("\\#.") || trimmed.starts_with("\\#\\.") {
                    line.replacen("\\*", "*", 1).replacen("\\#", "#", 1)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    while output.contains("\n\n\n") {
        output = output.replace("\n\n\n", "\n\n");
    }

    output = strip_arxiv_watermark_noise(output);

    let trimmed_len = output.trim_end().len();
    if trimmed_len == 0 {
        return String::new();
    }
    output.truncate(trimmed_len);
    output.push('\n');
    tracing::debug!(output_length = output.len(), "markdown rendering complete");
    output
}

/// Strip arXiv watermark noise from rendered markdown.
///
/// LaTeX-generated PDFs often have a rotated sidebar with the arXiv identifier
/// that pdfium concatenates with body text. This strips patterns like:
/// "Title N arXiv:NNNN.NNNNNvN [cat.SC] DD Mon YYYY" from the first pages.
fn strip_arxiv_watermark_noise(mut text: String) -> String {
    let search_limit = text.floor_char_boundary(text.len().min(6000));
    let search_area = &text[..search_limit];

    let re = regex::Regex::new(
        r"(?:\s+\S+(?:\s+\S+){0,8})?\s*arXiv:\d{4}\.\d{4,5}(?:v\d+)?(?:\s*\[[\w.-]+\])?\s*(?:\d{1,2}\s+\w+\s+\d{4})?",
    )
    .expect("valid regex");

    if let Some(m) = re.find(search_area) {
        let after = &search_area[m.end()..];
        let before_char = if m.start() > 0 {
            search_area[..m.start()].chars().last()
        } else {
            None
        };

        let is_at_paragraph_boundary = before_char == Some('.') || after.starts_with('\n') || after.starts_with("\n\n");
        if is_at_paragraph_boundary {
            let start = m.start();
            let end = m.end();
            tracing::trace!(
                stripped = %&text[start..end].chars().take(80).collect::<String>(),
                "stripping arXiv watermark from markdown output"
            );
            text.replace_range(start..end, "");
        }
    }

    text
}

/// Shared comrak options with all GFM extensions enabled.
pub(crate) fn comrak_options<'a>() -> Options<'a> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.extension.math_dollars = true;
    options.extension.underline = true;
    options.extension.subscript = true;
    options.extension.superscript = true;
    options.extension.highlight = true;
    options.extension.alerts = true;
    options.render.prefer_fenced = true;
    options
}
