//! Hover documentation provider

use schemars::schema_for;
use tower_lsp::lsp_types::*;

use mbf_fastq_processor::transformations::Transformation;

/// Provides hover documentation
pub struct HoverProvider {
    step_docs: std::collections::HashMap<String, String>,
}

impl HoverProvider {
    pub fn new() -> Self {
        let step_docs = Self::extract_step_documentation();
        Self { step_docs }
    }

    /// Extract documentation for all step actions from the schema
    fn extract_step_documentation() -> std::collections::HashMap<String, String> {
        let schema = schema_for!(Transformation);
        let mut docs = std::collections::HashMap::new();

        if let Some(one_ofs) = schema.as_object().and_then(|o| o.get("oneOf")) {
            if let Some(variants) = one_ofs.as_array() {
                for variant in variants {
                    if let Some(obj) = variant.as_object() {
                        if let Some(props) = obj.get("properties").and_then(|p| p.as_object()) {
                            if let Some(action) = props.get("action") {
                                if let Some(const_val) =
                                    action.as_object().and_then(|a| a.get("const"))
                                {
                                    if let Some(action_name) = const_val.as_str() {
                                        let description = obj
                                            .get("description")
                                            .and_then(|d| d.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        if !description.is_empty() {
                                            docs.insert(action_name.to_string(), description);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        docs
    }

    /// Get hover information for the current position
    pub fn get_hover(&self, text: &str, position: Position) -> Option<Hover> {
        let line_index = position.line as usize;
        let lines: Vec<&str> = text.lines().collect();

        if line_index >= lines.len() {
            return None;
        }

        let current_line = lines[line_index];
        let char_pos = position.character as usize;

        // Try to find what word is under the cursor
        let word = Self::extract_word_at_position(current_line, char_pos)?;

        // Check if it's a step action
        if let Some(doc) = self.step_docs.get(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("## {} (Step Action)\n\n{}", word, doc),
                }),
                range: None,
            });
        }

        // Check if it's a known section or keyword
        self.get_keyword_hover(&word)
    }

    /// Extract the word at the given character position
    fn extract_word_at_position(line: &str, char_pos: usize) -> Option<String> {
        if char_pos > line.len() {
            return None;
        }

        // Find word boundaries
        let mut start = char_pos;
        let mut end = char_pos;

        let chars: Vec<char> = line.chars().collect();

        // Move start backwards to find word start
        while start > 0 && Self::is_word_char(chars.get(start.saturating_sub(1))) {
            start -= 1;
        }

        // Move end forwards to find word end
        while end < chars.len() && Self::is_word_char(chars.get(end)) {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    /// Check if a character is part of a word
    fn is_word_char(c: Option<&char>) -> bool {
        match c {
            Some(&ch) => ch.is_alphanumeric() || ch == '_' || ch == '-',
            None => false,
        }
    }

    /// Get hover documentation for known keywords
    fn get_keyword_hover(&self, word: &str) -> Option<Hover> {
        let doc = match word {
            "input" => "Input section: Define input files (read1, read2, index1, index2)",
            "output" => "Output section: Define output format, compression, and reporting options",
            "step" => "Transformation step: Define a processing step in the pipeline",
            "options" => "Options section: Configure processing behavior",
            "barcodes" => "Barcodes section: Define barcode mappings for demultiplexing",
            "action" => "Step type: The transformation to perform (e.g., Head, FilterMinQuality, Report)",
            "segment" => "Target segment: Which read to operate on (read1, read2, index1, index2, all)",
            "out_label" => "Tag label: Name for the extracted or calculated value",
            "in_label" => "Input tag: Name of tag to use as input",
            "prefix" => "Output prefix: Base name for output files",
            "format" => "Output format: File format (FASTQ or BAM)",
            "compression" => "Compression: Compression format (Uncompressed, Gzip, or Zstd)",
            "report_html" => "Generate HTML report with quality statistics",
            "report_json" => "Generate JSON report with quality statistics",
            "read1" => "Forward reads input files",
            "read2" => "Reverse reads input files",
            "index1" => "Index 1 (i7) input files",
            "index2" => "Index 2 (i5) input files",
            "block_size" => "Number of reads to process in each block (affects memory usage)",
            "allow_overwrite" => "Allow overwriting existing output files",
            _ => return None,
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\n{}", word, doc),
            }),
            range: None,
        })
    }
}

impl Default for HoverProvider {
    fn default() -> Self {
        Self::new()
    }
}
