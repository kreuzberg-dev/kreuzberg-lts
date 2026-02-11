//! Inline DOCX XML parser.
//!
//! Vendored and adapted from [docx-lite](https://github.com/v-lawyer/docx-lite) v0.2.0
//! (MIT OR Apache-2.0, V-Lawyer Team). See ATTRIBUTIONS.md for details.
//!
//! Changes from upstream:
//! - `Paragraph::to_text()` joins runs with `" "` instead of `""` (fixes #359)
//! - Adapted to use kreuzberg's existing `quick-xml` and `zip` versions
//! - Removed file-path based APIs (we only need bytes/reader)
//! - Added markdown rendering and formatting support (fixes #376)

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek};

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

// --- Types ---

/// Tracks document element ordering (paragraphs and tables interleaved).
#[derive(Debug, Clone)]
pub enum DocumentElement {
    Paragraph(usize), // index into Document::paragraphs
    Table(usize),     // index into Document::tables
}

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub paragraphs: Vec<Paragraph>,
    pub tables: Vec<Table>,
    pub lists: Vec<ListItem>,
    pub headers: Vec<HeaderFooter>,
    pub footers: Vec<HeaderFooter>,
    pub footnotes: Vec<Note>,
    pub endnotes: Vec<Note>,
    pub numbering_defs: HashMap<(i64, i64), ListType>,
    /// Document elements in their original order.
    pub elements: Vec<DocumentElement>,
}

#[derive(Debug, Clone, Default)]
pub struct Paragraph {
    pub runs: Vec<Run>,
    pub style: Option<String>,
    pub numbering_id: Option<i64>,
    pub numbering_level: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub hyperlink_url: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Table {
    pub rows: Vec<TableRow>,
}

#[derive(Debug, Clone, Default)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, Default)]
pub struct TableCell {
    pub paragraphs: Vec<Paragraph>,
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub level: u32,
    pub list_type: ListType,
    pub number: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListType {
    Bullet,
    Numbered,
}

#[derive(Debug, Clone, Default)]
pub struct HeaderFooter {
    pub paragraphs: Vec<Paragraph>,
    pub tables: Vec<Table>,
    pub header_type: HeaderFooterType,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum HeaderFooterType {
    #[default]
    Default,
    First,
    Even,
    Odd,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub id: String,
    pub note_type: NoteType,
    pub paragraphs: Vec<Paragraph>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NoteType {
    Footnote,
    Endnote,
}

// --- Helper functions ---

/// Check if a formatting element is enabled (not explicitly set to false/0/none).
fn is_format_enabled(e: &BytesStart) -> bool {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"w:val"
            && let Ok(val) = std::str::from_utf8(&attr.value)
        {
            return !matches!(val, "false" | "0" | "none");
        }
    }
    true // no w:val attribute means enabled
}

/// Read `w:val` attribute as i64.
fn get_val_attr(e: &BytesStart) -> Option<i64> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"w:val"
            && let Ok(val) = std::str::from_utf8(&attr.value)
        {
            return val.parse().ok();
        }
    }
    None
}

/// Read `w:val` attribute as String.
fn get_val_attr_string(e: &BytesStart) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"w:val"
            && let Ok(val) = std::str::from_utf8(&attr.value)
        {
            return Some(val.to_string());
        }
    }
    None
}

/// Map heading style name to markdown heading level.
fn heading_level_from_style(style: &str) -> Option<u8> {
    match style {
        "Title" => Some(1),
        s if s.starts_with("Heading") || s.starts_with("heading") => {
            let num_part = s.trim_start_matches("Heading").trim_start_matches("heading");
            if let Ok(n) = num_part.parse::<u8>()
                && (1..=6).contains(&n)
            {
                // Title is h1, so Heading1 becomes h2, etc. Clamp to 6 (max markdown heading level).
                return Some((n + 1).min(6));
            }
            None
        }
        _ => None,
    }
}

// --- Impls ---

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extract_text(&self) -> String {
        let mut text = String::new();

        for paragraph in &self.paragraphs {
            let para_text = paragraph.to_text();
            if !para_text.is_empty() {
                text.push_str(&para_text);
                text.push('\n');
            }
        }

        for table in &self.tables {
            for row in &table.rows {
                for cell in &row.cells {
                    for paragraph in &cell.paragraphs {
                        let para_text = paragraph.to_text();
                        if !para_text.is_empty() {
                            text.push_str(&para_text);
                            text.push('\t');
                        }
                    }
                }
                text.push('\n');
            }
            text.push('\n');
        }

        text
    }

    /// Render the document as markdown.
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();
        let mut list_counters: HashMap<(i64, i64), usize> = HashMap::new();
        let mut prev_was_list = false;

        // Use elements ordering if populated, otherwise fall back to paragraphs-only
        if !self.elements.is_empty() {
            for element in &self.elements {
                match element {
                    DocumentElement::Paragraph(idx) => {
                        let paragraph = &self.paragraphs[*idx];
                        self.append_paragraph_markdown(paragraph, &mut output, &mut list_counters, &mut prev_was_list);
                    }
                    DocumentElement::Table(idx) => {
                        let table = &self.tables[*idx];
                        // Ensure blank line separation before table
                        if !output.is_empty() && !output.ends_with("\n\n") {
                            if output.ends_with('\n') {
                                output.push('\n');
                            } else {
                                output.push_str("\n\n");
                            }
                        }
                        output.push_str(&table.to_markdown());
                        prev_was_list = false;
                    }
                }
            }
        } else {
            for paragraph in &self.paragraphs {
                self.append_paragraph_markdown(paragraph, &mut output, &mut list_counters, &mut prev_was_list);
            }
        }

        // Footnotes
        if !self.footnotes.is_empty() {
            output.push_str("\n\n");
            for note in &self.footnotes {
                let note_text: String = note
                    .paragraphs
                    .iter()
                    .map(|p| p.runs_to_markdown())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !note_text.is_empty() {
                    output.push_str(&format!("[^{}]: {}\n", note.id, note_text));
                }
            }
        }

        // Endnotes
        if !self.endnotes.is_empty() {
            output.push_str("\n\n");
            for note in &self.endnotes {
                let note_text: String = note
                    .paragraphs
                    .iter()
                    .map(|p| p.runs_to_markdown())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !note_text.is_empty() {
                    output.push_str(&format!("[^{}]: {}\n", note.id, note_text));
                }
            }
        }

        output.trim().to_string()
    }

    /// Helper: append a paragraph's markdown to output, managing list transitions.
    fn append_paragraph_markdown(
        &self,
        paragraph: &Paragraph,
        output: &mut String,
        list_counters: &mut HashMap<(i64, i64), usize>,
        prev_was_list: &mut bool,
    ) {
        let para_text = paragraph.to_text();
        let is_list = paragraph.numbering_id.is_some();

        // Add blank line before list block when transitioning from non-list
        if is_list && !*prev_was_list && !output.is_empty() && !output.ends_with("\n\n") {
            if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }

        // Add blank line after list block when transitioning to non-list
        if !is_list && *prev_was_list && !output.is_empty() && !output.ends_with("\n\n") {
            if output.ends_with('\n') {
                output.push('\n');
            } else {
                output.push_str("\n\n");
            }
        }

        let md = paragraph.to_markdown(&self.numbering_defs, list_counters);
        if md.is_empty() && para_text.is_empty() {
            *prev_was_list = is_list;
            return;
        }

        if is_list {
            // List items separated by single newline
            if *prev_was_list {
                output.push('\n');
            }
            output.push_str(&md);
        } else {
            // Non-list paragraphs separated by blank lines
            if !output.is_empty() && !output.ends_with("\n\n") {
                if output.ends_with('\n') {
                    output.push('\n');
                } else {
                    output.push_str("\n\n");
                }
            }
            output.push_str(&md);
        }

        *prev_was_list = is_list;
    }
}

impl Paragraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Concatenate text runs to produce paragraph text.
    ///
    /// In DOCX, whitespace between words is stored inside `<w:t>` elements
    /// (e.g. `<w:t>Hello </w:t><w:t>World</w:t>`), so runs are joined
    /// directly without adding extra separators. The parser must use
    /// `trim_text(false)` to preserve this whitespace.
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        for run in &self.runs {
            text.push_str(&run.text);
        }
        text
    }

    /// Render inline runs as markdown (no paragraph-level wrapping).
    pub fn runs_to_markdown(&self) -> String {
        let mut text = String::new();
        for run in &self.runs {
            text.push_str(&run.to_markdown());
        }
        text
    }

    /// Render as markdown with heading/list context.
    pub fn to_markdown(
        &self,
        numbering_defs: &HashMap<(i64, i64), ListType>,
        list_counters: &mut HashMap<(i64, i64), usize>,
    ) -> String {
        let inline = self.runs_to_markdown();

        // Check for heading style
        if let Some(ref style) = self.style
            && let Some(level) = heading_level_from_style(style)
        {
            let hashes = "#".repeat(level as usize);
            return format!("{} {}", hashes, inline);
        }

        // Check for list item
        if let (Some(num_id), Some(level)) = (self.numbering_id, self.numbering_level) {
            let indent = "  ".repeat(level as usize);
            let key = (num_id, level);
            let list_type = numbering_defs.get(&key).cloned().unwrap_or(ListType::Bullet);

            match list_type {
                ListType::Bullet => {
                    return format!("{}- {}", indent, inline);
                }
                ListType::Numbered => {
                    let counter = list_counters.entry(key).or_insert(0);
                    *counter += 1;
                    return format!("{}{}. {}", indent, *counter, inline);
                }
            }
        }

        // Plain paragraph
        inline
    }

    pub fn add_run(&mut self, run: Run) {
        self.runs.push(run);
    }
}

impl Run {
    pub fn new(text: String) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }

    /// Render this run as markdown with formatting markers.
    pub fn to_markdown(&self) -> String {
        if self.text.is_empty() {
            return String::new();
        }

        let mut formatted = self.text.clone();

        // Apply formatting: innermost first
        if self.bold && self.italic {
            formatted = format!("***{}***", formatted);
        } else if self.bold {
            formatted = format!("**{}**", formatted);
        } else if self.italic {
            formatted = format!("*{}*", formatted);
        }

        if self.strikethrough {
            formatted = format!("~~{}~~", formatted);
        }

        // Hyperlink wraps everything
        if let Some(ref url) = self.hyperlink_url {
            formatted = format!("[{}]({})", formatted, url);
        }

        formatted
    }
}

impl Table {
    pub fn new() -> Self {
        Self::default()
    }

    /// Render this table as a markdown table.
    pub fn to_markdown(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }

        let cells: Vec<Vec<String>> = self
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| {
                        cell.paragraphs
                            .iter()
                            .map(|para| para.runs_to_markdown())
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string()
                    })
                    .collect()
            })
            .collect();

        if cells.is_empty() {
            return String::new();
        }

        let num_cols = cells.iter().map(|r| r.len()).max().unwrap_or(0);
        if num_cols == 0 {
            return String::new();
        }

        // Calculate column widths
        let mut col_widths = vec![3usize; num_cols];
        for row in &cells {
            for (i, cell) in row.iter().enumerate() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }

        let mut md = String::new();

        // Header row
        if let Some(header) = cells.first() {
            md.push('|');
            for (i, cell) in header.iter().enumerate() {
                let width = col_widths.get(i).copied().unwrap_or(3);
                md.push_str(&format!(" {:width$} |", cell, width = width));
            }
            // Pad missing columns
            for i in header.len()..num_cols {
                let width = col_widths.get(i).copied().unwrap_or(3);
                md.push_str(&format!(" {:width$} |", "", width = width));
            }
            md.push('\n');

            // Separator row
            md.push('|');
            for i in 0..num_cols {
                let width = col_widths.get(i).copied().unwrap_or(3);
                md.push_str(&format!(" {} |", "-".repeat(width)));
            }
            md.push('\n');
        }

        // Data rows
        for row in cells.iter().skip(1) {
            md.push('|');
            for (i, cell) in row.iter().enumerate() {
                let width = col_widths.get(i).copied().unwrap_or(3);
                md.push_str(&format!(" {:width$} |", cell, width = width));
            }
            // Pad missing columns
            for i in row.len()..num_cols {
                let width = col_widths.get(i).copied().unwrap_or(3);
                md.push_str(&format!(" {:width$} |", "", width = width));
            }
            md.push('\n');
        }

        md.trim_end().to_string()
    }
}

impl HeaderFooter {
    pub fn extract_text(&self) -> String {
        let mut text = String::new();

        for paragraph in &self.paragraphs {
            let para_text = paragraph.to_text();
            if !para_text.is_empty() {
                text.push_str(&para_text);
                text.push('\n');
            }
        }

        for table in &self.tables {
            for row in &table.rows {
                for cell in &row.cells {
                    for paragraph in &cell.paragraphs {
                        let para_text = paragraph.to_text();
                        if !para_text.is_empty() {
                            text.push_str(&para_text);
                            text.push('\t');
                        }
                    }
                }
                text.push('\n');
            }
        }

        text
    }
}

// --- Parser ---

struct DocxParser<R: Read + Seek> {
    archive: zip::ZipArchive<R>,
    relationships: HashMap<String, String>,
}

impl<R: Read + Seek> DocxParser<R> {
    fn new(reader: R) -> Result<Self, DocxParseError> {
        let archive = zip::ZipArchive::new(reader)?;
        Ok(Self {
            archive,
            relationships: HashMap::new(),
        })
    }

    fn parse(mut self) -> Result<Document, DocxParseError> {
        let mut document = Document::new();

        // Parse relationships first for hyperlink URL resolution
        if let Ok(rels_xml) = self.read_file("word/_rels/document.xml.rels") {
            self.relationships = Self::parse_relationships_xml(&rels_xml);
        }

        let document_xml = self.read_file("word/document.xml")?;
        self.parse_document_xml(&document_xml, &mut document)?;

        if let Ok(numbering_xml) = self.read_file("word/numbering.xml") {
            let numbering_defs = self.parse_numbering(&numbering_xml)?;
            self.process_lists(&mut document, &numbering_defs);
            document.numbering_defs = numbering_defs;
        }

        self.parse_headers_footers(&mut document)?;

        if let Ok(footnotes_xml) = self.read_file("word/footnotes.xml") {
            self.parse_notes(&footnotes_xml, &mut document.footnotes, NoteType::Footnote)?;
        }

        if let Ok(endnotes_xml) = self.read_file("word/endnotes.xml") {
            self.parse_notes(&endnotes_xml, &mut document.endnotes, NoteType::Endnote)?;
        }

        Ok(document)
    }

    /// Parse relationship file to get rId → URL mappings (for hyperlinks).
    fn parse_relationships_xml(xml: &str) -> HashMap<String, String> {
        let mut rels = HashMap::new();
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) if e.name().as_ref() == b"Relationship" => {
                    let mut id = None;
                    let mut target = None;
                    let mut rel_type = None;
                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"Id" => id = std::str::from_utf8(&attr.value).ok().map(String::from),
                            b"Target" => {
                                target = std::str::from_utf8(&attr.value).ok().map(String::from);
                            }
                            b"Type" => {
                                rel_type = std::str::from_utf8(&attr.value).ok().map(String::from);
                            }
                            _ => {}
                        }
                    }
                    // Only include hyperlink relationships
                    if let (Some(id_val), Some(target_val)) = (id, target)
                        && rel_type.as_ref().is_some_and(|t| t.contains("hyperlink"))
                    {
                        rels.insert(id_val, target_val);
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        rels
    }

    fn read_file(&mut self, path: &str) -> Result<String, DocxParseError> {
        let mut file = self
            .archive
            .by_name(path)
            .map_err(|_| DocxParseError::FileNotFound(path.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn parse_document_xml(&self, xml: &str, document: &mut Document) -> Result<(), DocxParseError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut current_paragraph: Option<Paragraph> = None;
        let mut table_paragraph: Option<Paragraph> = None;
        let mut current_run: Option<Run> = None;
        let mut current_table: Option<Table> = None;
        let mut current_row: Option<TableRow> = None;
        let mut current_cell: Option<TableCell> = None;
        let mut in_text = false;
        let mut in_table = false;
        let mut current_hyperlink_url: Option<String> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"w:p" => {
                        if in_table {
                            table_paragraph = Some(Paragraph::new());
                        } else {
                            current_paragraph = Some(Paragraph::new());
                        }
                    }
                    b"w:r" => {
                        let mut run = Run::default();
                        // Inherit hyperlink URL from context
                        if let Some(ref url) = current_hyperlink_url {
                            run.hyperlink_url = Some(url.clone());
                        }
                        current_run = Some(run);
                    }
                    b"w:t" => {
                        in_text = true;
                    }
                    b"w:tbl" => {
                        in_table = true;
                        current_table = Some(Table::new());
                    }
                    b"w:tr" => {
                        current_row = Some(TableRow::default());
                    }
                    b"w:tc" => {
                        current_cell = Some(TableCell::default());
                    }
                    b"w:b" => {
                        if let Some(ref mut run) = current_run {
                            run.bold = is_format_enabled(e);
                        }
                    }
                    b"w:i" => {
                        if let Some(ref mut run) = current_run {
                            run.italic = is_format_enabled(e);
                        }
                    }
                    b"w:u" => {
                        if let Some(ref mut run) = current_run {
                            run.underline = is_format_enabled(e);
                        }
                    }
                    b"w:strike" | b"w:dstrike" => {
                        if let Some(ref mut run) = current_run {
                            run.strikethrough = is_format_enabled(e);
                        }
                    }
                    b"w:pStyle" => {
                        let para = if in_table {
                            table_paragraph.as_mut()
                        } else {
                            current_paragraph.as_mut()
                        };
                        if let Some(para) = para {
                            para.style = get_val_attr_string(e);
                        }
                    }
                    b"w:ilvl" => {
                        let para = if in_table {
                            table_paragraph.as_mut()
                        } else {
                            current_paragraph.as_mut()
                        };
                        if let Some(para) = para {
                            para.numbering_level = get_val_attr(e);
                        }
                    }
                    b"w:numId" => {
                        let para = if in_table {
                            table_paragraph.as_mut()
                        } else {
                            current_paragraph.as_mut()
                        };
                        if let Some(para) = para {
                            para.numbering_id = get_val_attr(e);
                        }
                    }
                    b"w:hyperlink" => {
                        // Look up r:id in relationships
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"r:id"
                                && let Ok(rid) = std::str::from_utf8(&attr.value)
                            {
                                current_hyperlink_url = self.relationships.get(rid).cloned();
                            }
                        }
                    }
                    _ => {}
                },
                Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    b"w:b" => {
                        if let Some(ref mut run) = current_run {
                            run.bold = is_format_enabled(e);
                        }
                    }
                    b"w:i" => {
                        if let Some(ref mut run) = current_run {
                            run.italic = is_format_enabled(e);
                        }
                    }
                    b"w:u" => {
                        if let Some(ref mut run) = current_run {
                            run.underline = is_format_enabled(e);
                        }
                    }
                    b"w:strike" | b"w:dstrike" => {
                        if let Some(ref mut run) = current_run {
                            run.strikethrough = is_format_enabled(e);
                        }
                    }
                    b"w:pStyle" => {
                        let para = if in_table {
                            table_paragraph.as_mut()
                        } else {
                            current_paragraph.as_mut()
                        };
                        if let Some(para) = para {
                            para.style = get_val_attr_string(e);
                        }
                    }
                    b"w:ilvl" => {
                        let para = if in_table {
                            table_paragraph.as_mut()
                        } else {
                            current_paragraph.as_mut()
                        };
                        if let Some(para) = para {
                            para.numbering_level = get_val_attr(e);
                        }
                    }
                    b"w:numId" => {
                        let para = if in_table {
                            table_paragraph.as_mut()
                        } else {
                            current_paragraph.as_mut()
                        };
                        if let Some(para) = para {
                            para.numbering_id = get_val_attr(e);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Text(e)) => {
                    if in_text && let Some(ref mut run) = current_run {
                        let text = e.decode()?.into_owned();
                        run.text.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => match e.name().as_ref() {
                    b"w:t" => {
                        in_text = false;
                    }
                    b"w:r" => {
                        if let Some(run) = current_run.take() {
                            if in_table {
                                if let Some(ref mut para) = table_paragraph {
                                    para.add_run(run);
                                } else if let Some(ref mut cell) = current_cell {
                                    if cell.paragraphs.is_empty() {
                                        cell.paragraphs.push(Paragraph::new());
                                    }
                                    if let Some(para) = cell.paragraphs.last_mut() {
                                        para.add_run(run);
                                    }
                                }
                            } else if let Some(ref mut para) = current_paragraph {
                                para.add_run(run);
                            }
                        }
                    }
                    b"w:p" => {
                        if in_table {
                            if let Some(para) = table_paragraph.take()
                                && let Some(ref mut cell) = current_cell
                            {
                                cell.paragraphs.push(para);
                            }
                        } else if let Some(para) = current_paragraph.take() {
                            let idx = document.paragraphs.len();
                            document.paragraphs.push(para);
                            document.elements.push(DocumentElement::Paragraph(idx));
                        }
                    }
                    b"w:tc" => {
                        if let Some(cell) = current_cell.take()
                            && let Some(ref mut row) = current_row
                        {
                            row.cells.push(cell);
                        }
                    }
                    b"w:tr" => {
                        if let Some(row) = current_row.take()
                            && let Some(ref mut table) = current_table
                        {
                            table.rows.push(row);
                        }
                    }
                    b"w:tbl" => {
                        in_table = false;
                        if let Some(table) = current_table.take() {
                            let idx = document.tables.len();
                            document.tables.push(table);
                            document.elements.push(DocumentElement::Table(idx));
                        }
                    }
                    b"w:hyperlink" => {
                        current_hyperlink_url = None;
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    fn parse_numbering(&self, xml: &str) -> Result<HashMap<(i64, i64), ListType>, DocxParseError> {
        let mut numbering_defs: HashMap<(i64, i64), ListType> = HashMap::new();
        let mut abstract_num_formats: HashMap<i64, HashMap<i64, ListType>> = HashMap::new();
        let mut num_to_abstract: HashMap<i64, i64> = HashMap::new();

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut current_abstract_num_id: Option<i64> = None;
        let mut current_num_id: Option<i64> = None;
        let mut current_lvl: Option<i64> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"w:abstractNum" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:abstractNumId"
                                && let Ok(id_str) = std::str::from_utf8(&attr.value)
                            {
                                current_abstract_num_id = id_str.parse().ok();
                            }
                        }
                    }
                    b"w:num" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:numId"
                                && let Ok(id_str) = std::str::from_utf8(&attr.value)
                            {
                                current_num_id = id_str.parse().ok();
                            }
                        }
                    }
                    b"w:lvl" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:ilvl"
                                && let Ok(id_str) = std::str::from_utf8(&attr.value)
                            {
                                current_lvl = id_str.parse().ok();
                            }
                        }
                    }
                    b"w:numFmt" => {
                        if let (Some(abstract_id), Some(lvl)) = (current_abstract_num_id, current_lvl) {
                            let fmt = get_val_attr_string(e);
                            let list_type = match fmt.as_deref() {
                                Some("decimal") | Some("decimalZero") | Some("lowerLetter") | Some("upperLetter")
                                | Some("lowerRoman") | Some("upperRoman") => ListType::Numbered,
                                _ => ListType::Bullet,
                            };
                            abstract_num_formats
                                .entry(abstract_id)
                                .or_default()
                                .insert(lvl, list_type);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    b"w:abstractNumId" => {
                        if let Some(num_id) = current_num_id
                            && let Some(abstract_id) = get_val_attr(e)
                        {
                            num_to_abstract.insert(num_id, abstract_id);
                        }
                    }
                    b"w:numFmt" => {
                        if let (Some(abstract_id), Some(lvl)) = (current_abstract_num_id, current_lvl) {
                            let fmt = get_val_attr_string(e);
                            let list_type = match fmt.as_deref() {
                                Some("decimal") | Some("decimalZero") | Some("lowerLetter") | Some("upperLetter")
                                | Some("lowerRoman") | Some("upperRoman") => ListType::Numbered,
                                _ => ListType::Bullet,
                            };
                            abstract_num_formats
                                .entry(abstract_id)
                                .or_default()
                                .insert(lvl, list_type);
                        }
                    }
                    _ => {}
                },
                Ok(Event::End(ref e)) => match e.name().as_ref() {
                    b"w:abstractNum" => {
                        current_abstract_num_id = None;
                        current_lvl = None;
                    }
                    b"w:lvl" => {
                        current_lvl = None;
                    }
                    b"w:num" => {
                        current_num_id = None;
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        // Build final numbering_defs by resolving num → abstractNum references
        for (num_id, abstract_id) in &num_to_abstract {
            if let Some(formats) = abstract_num_formats.get(abstract_id) {
                for (lvl, list_type) in formats {
                    numbering_defs.insert((*num_id, *lvl), list_type.clone());
                }
            }
        }

        Ok(numbering_defs)
    }

    fn process_lists(&self, document: &mut Document, numbering_defs: &HashMap<(i64, i64), ListType>) {
        for paragraph in &document.paragraphs {
            if let (Some(num_id), Some(level)) = (paragraph.numbering_id, paragraph.numbering_level) {
                let key = (num_id, level);
                let list_type = numbering_defs.get(&key).cloned().unwrap_or(ListType::Bullet);

                let list_item = ListItem {
                    level: level as u32,
                    list_type,
                    number: None,
                    text: paragraph.to_text(),
                };

                document.lists.push(list_item);
            }
        }
    }

    fn parse_headers_footers(&mut self, document: &mut Document) -> Result<(), DocxParseError> {
        for i in 1..=3 {
            let header_path = format!("word/header{}.xml", i);
            if let Ok(header_xml) = self.read_file(&header_path) {
                let mut header = HeaderFooter::default();
                self.parse_header_footer_content(&header_xml, &mut header)?;
                document.headers.push(header);
            }

            let footer_path = format!("word/footer{}.xml", i);
            if let Ok(footer_xml) = self.read_file(&footer_path) {
                let mut footer = HeaderFooter::default();
                self.parse_header_footer_content(&footer_xml, &mut footer)?;
                document.footers.push(footer);
            }
        }

        Ok(())
    }

    fn parse_header_footer_content(&self, xml: &str, header_footer: &mut HeaderFooter) -> Result<(), DocxParseError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut current_paragraph: Option<Paragraph> = None;
        let mut current_run: Option<Run> = None;
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"w:p" => current_paragraph = Some(Paragraph::new()),
                    b"w:r" => current_run = Some(Run::default()),
                    b"w:t" => in_text = true,
                    b"w:b" => {
                        if let Some(ref mut run) = current_run {
                            run.bold = is_format_enabled(e);
                        }
                    }
                    b"w:i" => {
                        if let Some(ref mut run) = current_run {
                            run.italic = is_format_enabled(e);
                        }
                    }
                    b"w:u" => {
                        if let Some(ref mut run) = current_run {
                            run.underline = is_format_enabled(e);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    b"w:b" => {
                        if let Some(ref mut run) = current_run {
                            run.bold = is_format_enabled(e);
                        }
                    }
                    b"w:i" => {
                        if let Some(ref mut run) = current_run {
                            run.italic = is_format_enabled(e);
                        }
                    }
                    b"w:u" => {
                        if let Some(ref mut run) = current_run {
                            run.underline = is_format_enabled(e);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Text(e)) => {
                    if in_text && let Some(ref mut run) = current_run {
                        let text = e.decode()?.into_owned();
                        run.text.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => match e.name().as_ref() {
                    b"w:t" => in_text = false,
                    b"w:r" => {
                        if let Some(run) = current_run.take()
                            && let Some(ref mut para) = current_paragraph
                        {
                            para.add_run(run);
                        }
                    }
                    b"w:p" => {
                        if let Some(para) = current_paragraph.take() {
                            header_footer.paragraphs.push(para);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    fn parse_notes(&self, xml: &str, notes: &mut Vec<Note>, note_type: NoteType) -> Result<(), DocxParseError> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut buf = Vec::new();
        let mut current_note: Option<Note> = None;
        let mut current_paragraph: Option<Paragraph> = None;
        let mut current_run: Option<Run> = None;
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"w:footnote" | b"w:endnote" => {
                        let mut id = String::new();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"w:id" {
                                id = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                        current_note = Some(Note {
                            id,
                            note_type: note_type.clone(),
                            paragraphs: Vec::new(),
                        });
                    }
                    b"w:p" => current_paragraph = Some(Paragraph::new()),
                    b"w:r" => current_run = Some(Run::default()),
                    b"w:t" => in_text = true,
                    b"w:b" => {
                        if let Some(ref mut run) = current_run {
                            run.bold = is_format_enabled(e);
                        }
                    }
                    b"w:i" => {
                        if let Some(ref mut run) = current_run {
                            run.italic = is_format_enabled(e);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    b"w:b" => {
                        if let Some(ref mut run) = current_run {
                            run.bold = is_format_enabled(e);
                        }
                    }
                    b"w:i" => {
                        if let Some(ref mut run) = current_run {
                            run.italic = is_format_enabled(e);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Text(e)) => {
                    if in_text && let Some(ref mut run) = current_run {
                        let text = e.decode()?.into_owned();
                        run.text.push_str(&text);
                    }
                }
                Ok(Event::End(ref e)) => match e.name().as_ref() {
                    b"w:t" => in_text = false,
                    b"w:r" => {
                        if let Some(run) = current_run.take()
                            && let Some(ref mut para) = current_paragraph
                        {
                            para.add_run(run);
                        }
                    }
                    b"w:p" => {
                        if let Some(para) = current_paragraph.take()
                            && let Some(ref mut note) = current_note
                        {
                            note.paragraphs.push(para);
                        }
                    }
                    b"w:footnote" | b"w:endnote" => {
                        if let Some(note) = current_note.take()
                            && note.id != "-1"
                            && note.id != "0"
                        {
                            notes.push(note);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }
}

// --- Error ---

#[derive(Debug, thiserror::Error)]
enum DocxParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("Required file not found in DOCX: {0}")]
    FileNotFound(String),
}

// quick-xml's unescape returns an encoding error type
impl From<quick_xml::encoding::EncodingError> for DocxParseError {
    fn from(e: quick_xml::encoding::EncodingError) -> Self {
        DocxParseError::Xml(quick_xml::Error::Encoding(e))
    }
}

// --- Public API ---

/// Parse a DOCX document from bytes and return the structured document.
pub fn parse_document(bytes: &[u8]) -> crate::error::Result<Document> {
    let cursor = Cursor::new(bytes);
    let parser = DocxParser::new(cursor)
        .map_err(|e| crate::error::KreuzbergError::parsing(format!("DOCX parsing failed: {}", e)))?;
    parser
        .parse()
        .map_err(|e| crate::error::KreuzbergError::parsing(format!("DOCX parsing failed: {}", e)))
}

/// Extract text from DOCX bytes.
pub fn extract_text_from_bytes(bytes: &[u8]) -> crate::error::Result<String> {
    let doc = parse_document(bytes)?;
    Ok(doc.extract_text())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs are concatenated directly; whitespace comes from the XML text content.
    #[test]
    fn test_paragraph_to_text_concatenates_runs() {
        let mut para = Paragraph::new();
        para.add_run(Run::new("Hello ".to_string()));
        para.add_run(Run::new("World".to_string()));
        assert_eq!(para.to_text(), "Hello World");
    }

    /// Mid-word run splits (e.g. drop caps) must not insert extra spaces.
    #[test]
    fn test_paragraph_to_text_mid_word_split() {
        let mut para = Paragraph::new();
        para.add_run(Run::new("S".to_string()));
        para.add_run(Run::new("ermocination".to_string()));
        assert_eq!(para.to_text(), "Sermocination");
    }

    #[test]
    fn test_paragraph_to_text_single_run() {
        let mut para = Paragraph::new();
        para.add_run(Run::new("Hello".to_string()));
        assert_eq!(para.to_text(), "Hello");
    }

    #[test]
    fn test_paragraph_to_text_no_runs() {
        let para = Paragraph::new();
        assert_eq!(para.to_text(), "");
    }

    /// Whitespace between words is stored in the run text, not added by join.
    #[test]
    fn test_paragraph_to_text_whitespace_in_runs() {
        let mut para = Paragraph::new();
        para.add_run(Run::new("The ".to_string()));
        para.add_run(Run::new("quick ".to_string()));
        para.add_run(Run::new("fox".to_string()));
        assert_eq!(para.to_text(), "The quick fox");
    }

    // --- Markdown rendering unit tests ---

    #[test]
    fn test_run_bold_to_markdown() {
        let run = Run {
            text: "hello".to_string(),
            bold: true,
            ..Default::default()
        };
        assert_eq!(run.to_markdown(), "**hello**");
    }

    #[test]
    fn test_run_italic_to_markdown() {
        let run = Run {
            text: "hello".to_string(),
            italic: true,
            ..Default::default()
        };
        assert_eq!(run.to_markdown(), "*hello*");
    }

    #[test]
    fn test_run_bold_italic_to_markdown() {
        let run = Run {
            text: "hello".to_string(),
            bold: true,
            italic: true,
            ..Default::default()
        };
        assert_eq!(run.to_markdown(), "***hello***");
    }

    #[test]
    fn test_run_strikethrough_to_markdown() {
        let run = Run {
            text: "hello".to_string(),
            strikethrough: true,
            ..Default::default()
        };
        assert_eq!(run.to_markdown(), "~~hello~~");
    }

    #[test]
    fn test_run_hyperlink_to_markdown() {
        let run = Run {
            text: "click here".to_string(),
            hyperlink_url: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(run.to_markdown(), "[click here](https://example.com)");
    }

    #[test]
    fn test_run_bold_hyperlink_to_markdown() {
        let run = Run {
            text: "click".to_string(),
            bold: true,
            hyperlink_url: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(run.to_markdown(), "[**click**](https://example.com)");
    }

    #[test]
    fn test_run_empty_text_to_markdown() {
        let run = Run {
            text: String::new(),
            bold: true,
            ..Default::default()
        };
        assert_eq!(run.to_markdown(), "");
    }

    #[test]
    fn test_paragraph_heading_to_markdown() {
        let mut para = Paragraph::new();
        para.style = Some("Title".to_string());
        para.add_run(Run::new("My Title".to_string()));
        let defs = HashMap::new();
        let mut counters = HashMap::new();
        assert_eq!(para.to_markdown(&defs, &mut counters), "# My Title");
    }

    #[test]
    fn test_paragraph_heading1_to_markdown() {
        let mut para = Paragraph::new();
        para.style = Some("Heading1".to_string());
        para.add_run(Run::new("Section".to_string()));
        let defs = HashMap::new();
        let mut counters = HashMap::new();
        assert_eq!(para.to_markdown(&defs, &mut counters), "## Section");
    }

    #[test]
    fn test_paragraph_heading2_to_markdown() {
        let mut para = Paragraph::new();
        para.style = Some("Heading2".to_string());
        para.add_run(Run::new("Subsection".to_string()));
        let defs = HashMap::new();
        let mut counters = HashMap::new();
        assert_eq!(para.to_markdown(&defs, &mut counters), "### Subsection");
    }

    #[test]
    fn test_paragraph_bullet_list_to_markdown() {
        let mut para = Paragraph::new();
        para.numbering_id = Some(1);
        para.numbering_level = Some(0);
        para.add_run(Run::new("Item".to_string()));
        let mut defs = HashMap::new();
        defs.insert((1, 0), ListType::Bullet);
        let mut counters = HashMap::new();
        assert_eq!(para.to_markdown(&defs, &mut counters), "- Item");
    }

    #[test]
    fn test_paragraph_numbered_list_to_markdown() {
        let mut para = Paragraph::new();
        para.numbering_id = Some(2);
        para.numbering_level = Some(0);
        para.add_run(Run::new("Item".to_string()));
        let mut defs = HashMap::new();
        defs.insert((2, 0), ListType::Numbered);
        let mut counters = HashMap::new();
        assert_eq!(para.to_markdown(&defs, &mut counters), "1. Item");
    }

    #[test]
    fn test_paragraph_nested_list_to_markdown() {
        let mut para = Paragraph::new();
        para.numbering_id = Some(1);
        para.numbering_level = Some(1);
        para.add_run(Run::new("Nested".to_string()));
        let mut defs = HashMap::new();
        defs.insert((1, 1), ListType::Bullet);
        let mut counters = HashMap::new();
        assert_eq!(para.to_markdown(&defs, &mut counters), "  - Nested");
    }

    #[test]
    fn test_heading_level_from_style() {
        assert_eq!(heading_level_from_style("Title"), Some(1));
        assert_eq!(heading_level_from_style("Heading1"), Some(2));
        assert_eq!(heading_level_from_style("Heading2"), Some(3));
        assert_eq!(heading_level_from_style("Heading3"), Some(4));
        assert_eq!(heading_level_from_style("Heading6"), Some(6)); // clamped to max markdown level
        assert_eq!(heading_level_from_style("Normal"), None);
    }

    #[test]
    fn test_is_format_enabled_no_val() {
        // <w:b/> - no w:val attribute means enabled
        let xml = r#"<w:b/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        if let Ok(Event::Empty(ref e)) = reader.read_event_into(&mut buf) {
            assert!(is_format_enabled(e));
        }
    }
}
