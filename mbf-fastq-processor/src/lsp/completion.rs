//! Completion provider for TOML configuration files

use schemars::schema_for;
use tower_lsp::lsp_types::*;

use mbf_fastq_processor::transformations::Transformation;

/// Provides auto-completion suggestions
pub struct CompletionProvider {
    step_actions: Vec<(String, String)>,
}

impl CompletionProvider {
    pub fn new() -> Self {
        let step_actions = Self::extract_step_actions();
        Self { step_actions }
    }

    /// Extract all step action names and descriptions from the schema
    fn extract_step_actions() -> Vec<(String, String)> {
        let schema = schema_for!(Transformation);
        let mut actions = Vec::new();

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
                                        actions.push((action_name.to_string(), description));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        actions.sort_by(|a, b| a.0.cmp(&b.0));
        actions
    }

    /// Get completions for the current position
    pub fn get_completions(&self, text: &str, position: Position) -> Vec<CompletionItem> {
        let line_index = position.line as usize;
        let lines: Vec<&str> = text.lines().collect();

        if line_index >= lines.len() {
            return Vec::new();
        }

        let current_line = lines[line_index];
        let context = Self::analyze_context(current_line, position.character as usize);

        match context {
            CompletionContext::StepAction => self.get_step_action_completions(),
            CompletionContext::SectionHeader => self.get_section_header_completions(),
            CompletionContext::InputKey => self.get_input_key_completions(),
            CompletionContext::OutputKey => self.get_output_key_completions(),
            CompletionContext::StepKey => self.get_step_key_completions(),
            CompletionContext::OptionsKey => self.get_options_key_completions(),
            CompletionContext::Unknown => Vec::new(),
        }
    }

    /// Analyze the current line to determine what kind of completion we should provide
    fn analyze_context(line: &str, _char_pos: usize) -> CompletionContext {
        let trimmed = line.trim();

        // Check for section headers
        if trimmed.starts_with('[') && !trimmed.contains(']') {
            return CompletionContext::SectionHeader;
        }

        // Check if we're in a [[step]] section context
        if trimmed.contains("action") {
            return CompletionContext::StepAction;
        }

        // Determine section by looking at previous lines
        // This is simplified - in a real implementation, we'd parse the full TOML
        if trimmed.contains("=") {
            let key = trimmed.split('=').next().unwrap_or("").trim();
            if key == "action" {
                return CompletionContext::StepAction;
            }
        }

        // Check for input, output, or step sections
        // This is simplified - we'd need to track the current section
        if line.contains("read1")
            || line.contains("read2")
            || line.contains("index1")
            || line.contains("index2")
        {
            return CompletionContext::InputKey;
        }

        if line.contains("prefix") || line.contains("format") || line.contains("compression") {
            return CompletionContext::OutputKey;
        }

        CompletionContext::Unknown
    }

    /// Get completions for step actions
    fn get_step_action_completions(&self) -> Vec<CompletionItem> {
        self.step_actions
            .iter()
            .map(|(action, description)| CompletionItem {
                label: action.clone(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("Step action".to_string()),
                documentation: Some(Documentation::String(description.clone())),
                insert_text: Some(format!("\"{}\"", action)),
                ..Default::default()
            })
            .collect()
    }

    /// Get completions for section headers
    fn get_section_header_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "[input]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Input section with template".to_string()),
                documentation: Some(Documentation::String(
                    "Define input files (read1, read2, index1, index2)".to_string(),
                )),
                insert_text: Some("[input]\n    read1 = ['${1:input_R1.fastq.gz}']$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "[[step]]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Transformation step with template".to_string()),
                documentation: Some(Documentation::String(
                    "Define a transformation step in the pipeline".to_string(),
                )),
                insert_text: Some("[[step]]\n    action = \"${1:Head}\"\n    ${2:n = 1000}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "[[step]] - Report".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Quality report step".to_string()),
                documentation: Some(Documentation::String(
                    "Generate a comprehensive quality report".to_string(),
                )),
                insert_text: Some("[[step]]\n    action = \"Report\"\n    name = \"${1:initial}\"\n    count = true\n    base_statistics = ${2:true}\n    length_distribution = ${3:true}\n    duplicate_count_per_read = ${4:false}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "[[step]] - ExtractRegions".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Extract UMI or barcode".to_string()),
                documentation: Some(Documentation::String(
                    "Extract regions from reads (e.g., UMI, barcodes)".to_string(),
                )),
                insert_text: Some("[[step]]\n    action = \"ExtractRegions\"\n    out_label = \"${1:umi}\"\n    regions = [{segment = '${2:read1}', start = ${3:0}, length = ${4:8}}]$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "[[step]] - Head".to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                detail: Some("Take first N reads".to_string()),
                documentation: Some(Documentation::String(
                    "Keep only the first N reads for testing".to_string(),
                )),
                insert_text: Some("[[step]]\n    action = \"Head\"\n    n = ${1:1000}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "[output]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Output section with template".to_string()),
                documentation: Some(Documentation::String(
                    "Define output format and options".to_string(),
                )),
                insert_text: Some("[output]\n    prefix = \"${1:output}\"\n    format = \"${2:FASTQ}\"\n    compression = \"${3:Gzip}\"\n    report_html = ${4:true}\n    report_json = ${5:false}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "[options]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Options section with template".to_string()),
                documentation: Some(Documentation::String(
                    "Configure processing options".to_string(),
                )),
                insert_text: Some("[options]\n    block_size = ${1:10000}\n    allow_overwrite = ${2:false}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "[barcodes.NAME]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Barcode section with template".to_string()),
                documentation: Some(Documentation::String(
                    "Define barcode mappings for demultiplexing".to_string(),
                )),
                insert_text: Some("[barcodes.${1:barcodes}]\n    ${2:ATCG} = \"${3:sample1}\"\n    ${4:GCTA} = \"${5:sample2}\"$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Get completions for input section keys
    fn get_input_key_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "read1".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Input file paths".to_string()),
                documentation: Some(Documentation::String(
                    "List of read1 (forward) input files".to_string(),
                )),
                insert_text: Some("read1 = ['${1:input_R1.fastq.gz}']$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "read2".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Input file paths".to_string()),
                documentation: Some(Documentation::String(
                    "List of read2 (reverse) input files".to_string(),
                )),
                insert_text: Some("read2 = ['${1:input_R2.fastq.gz}']$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "index1".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Input file paths".to_string()),
                documentation: Some(Documentation::String(
                    "List of index1 input files".to_string(),
                )),
                insert_text: Some("index1 = ['${1:input_I1.fastq.gz}']$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "index2".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Input file paths".to_string()),
                documentation: Some(Documentation::String(
                    "List of index2 input files".to_string(),
                )),
                insert_text: Some("index2 = ['${1:input_I2.fastq.gz}']$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Get completions for output section keys
    fn get_output_key_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "prefix".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Output prefix".to_string()),
                documentation: Some(Documentation::String(
                    "Prefix for output files".to_string(),
                )),
                insert_text: Some("prefix = \"${1:output}\"$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "format".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Output format".to_string()),
                documentation: Some(Documentation::String(
                    "Output format (FASTQ, BAM)".to_string(),
                )),
                insert_text: Some("format = \"${1|FASTQ,BAM|}\"$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "compression".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Compression format".to_string()),
                documentation: Some(Documentation::String(
                    "Compression format (Uncompressed, Gzip, Zstd)".to_string(),
                )),
                insert_text: Some("compression = \"${1|Uncompressed,Gzip,Zstd|}\"$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "report_html".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Generate HTML report".to_string()),
                documentation: Some(Documentation::String(
                    "Generate HTML quality report".to_string(),
                )),
                insert_text: Some("report_html = ${1|true,false|}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "report_json".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Generate JSON report".to_string()),
                documentation: Some(Documentation::String(
                    "Generate JSON quality report".to_string(),
                )),
                insert_text: Some("report_json = ${1|true,false|}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Get completions for step section keys
    fn get_step_key_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "action".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Step type".to_string()),
                documentation: Some(Documentation::String(
                    "The type of transformation to perform".to_string(),
                )),
                insert_text: Some("action = \"${1:Head}\"$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "segment".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Target segment".to_string()),
                documentation: Some(Documentation::String(
                    "Which segment to operate on (read1, read2, index1, index2, all)".to_string(),
                )),
                insert_text: Some("segment = \"${1|read1,read2,index1,index2,all|}\"$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "out_label".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Tag label".to_string()),
                documentation: Some(Documentation::String(
                    "Label for extracted/calculated tag".to_string(),
                )),
                insert_text: Some("out_label = \"${1:tag_name}\"$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "in_label".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Input tag label".to_string()),
                documentation: Some(Documentation::String(
                    "Label of tag to use as input".to_string(),
                )),
                insert_text: Some("in_label = \"${1:tag_name}\"$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Get completions for options section keys
    fn get_options_key_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "block_size".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Processing block size".to_string()),
                documentation: Some(Documentation::String(
                    "Number of reads to process at once".to_string(),
                )),
                insert_text: Some("block_size = ${1:10000}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "allow_overwrite".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Allow overwriting files".to_string()),
                documentation: Some(Documentation::String(
                    "Allow overwriting existing output files".to_string(),
                )),
                insert_text: Some("allow_overwrite = ${1|true,false|}$0".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }
}

impl Default for CompletionProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the context where completion is being requested
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionContext {
    StepAction,
    SectionHeader,
    InputKey,
    OutputKey,
    StepKey,
    OptionsKey,
    Unknown,
}
