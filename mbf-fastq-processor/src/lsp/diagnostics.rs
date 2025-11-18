//! Diagnostics provider for validating TOML configuration

use tower_lsp::lsp_types::*;
use toml_edit::DocumentMut;

/// Provides diagnostics (errors/warnings) for TOML documents
pub struct DiagnosticsProvider;

impl DiagnosticsProvider {
    pub fn new() -> Self {
        Self
    }

    /// Get diagnostics for a TOML document
    pub fn get_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // First, try to parse with toml_edit for better position information
        match text.parse::<DocumentMut>() {
            Ok(_doc) => {
                // TOML is syntactically valid, now try to parse as Config and validate
                match eserde::toml::from_str::<mbf_fastq_processor::config::Config>(text) {
                    Ok(mut config) => {
                        // Run validation (skip file existence checks)
                        if let Err(e) = config.check_for_validation() {
                            // Try to find the relevant section in the error message
                            let error_msg = format!("{:?}", e);
                            let (range, clean_msg) = Self::extract_location_from_error(text, &error_msg);

                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::ERROR),
                                source: Some("mbf-fastq-processor".to_string()),
                                message: clean_msg,
                                ..Default::default()
                            });
                        }
                    }
                    Err(e) => {
                        // Config parsing error - try to get position info
                        let error_str = e.to_string();
                        let (range, message) = Self::parse_error_with_position(text, &error_str);

                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("mbf-fastq-processor".to_string()),
                            message,
                            ..Default::default()
                        });
                    }
                }
            }
            Err(e) => {
                // TOML syntax error - toml_edit errors include span information
                let error_str = e.to_string();
                let (range, message) = Self::parse_toml_edit_error(text, &error_str);

                diagnostics.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("toml".to_string()),
                    message,
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// Extract location information from validation error messages
    /// Looks for patterns like "[Step 0", "[input]", "[output]", etc.
    fn extract_location_from_error(text: &str, error_msg: &str) -> (Range, String) {
        // Try to find section references like [Step 0], [input], [output], etc.
        let patterns = [
            (r"\[Step (\d+)", "[[step]]"),
            (r"\(input\)", "[input]"),
            (r"\(output\)", "[output]"),
            (r"\[barcodes\.([^\]]+)\]", "[barcodes."),
        ];

        for (pattern_regex, search_str) in patterns {
            if let Some(caps) = regex::Regex::new(pattern_regex).ok().and_then(|re| re.captures(error_msg)) {
                let search_text = if search_str == "[[step]]" {
                    // For steps, we need to find the Nth occurrence
                    if let Some(step_num) = caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok()) {
                        if let Some(pos) = Self::find_nth_step(text, step_num) {
                            return (pos, error_msg.to_string());
                        }
                    }
                    search_str
                } else {
                    search_str
                };

                if let Some(range) = Self::find_section_position(text, search_text) {
                    return (range, error_msg.to_string());
                }
            }
        }

        // Default to top of file
        (
            Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 0 },
            },
            error_msg.to_string(),
        )
    }

    /// Find the position of the Nth [[step]] section
    fn find_nth_step(text: &str, n: usize) -> Option<Range> {
        let mut count = 0;
        for (line_num, line) in text.lines().enumerate() {
            if line.trim().starts_with("[[step]]") {
                if count == n {
                    return Some(Range {
                        start: Position {
                            line: line_num as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    });
                }
                count += 1;
            }
        }
        None
    }

    /// Find the position of a section header in the text
    fn find_section_position(text: &str, section: &str) -> Option<Range> {
        for (line_num, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with(section) {
                return Some(Range {
                    start: Position {
                        line: line_num as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: line.len() as u32,
                    },
                });
            }
        }
        None
    }

    /// Parse toml_edit error which may include line:column information
    fn parse_toml_edit_error(text: &str, error_str: &str) -> (Range, String) {
        // toml_edit errors often include "at line X column Y" or similar
        let (line, col) = Self::extract_line_col(error_str);

        if line > 0 {
            let line_idx = line.saturating_sub(1);
            let char_pos = col.saturating_sub(1);

            // Try to get the actual line length
            let line_len = text.lines().nth(line_idx).map(|l| l.len()).unwrap_or(0);

            return (
                Range {
                    start: Position {
                        line: line_idx as u32,
                        character: char_pos as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: line_len.min(char_pos + 10) as u32, // Highlight a small region
                    },
                },
                error_str.to_string(),
            );
        }

        Self::parse_error_with_position(text, error_str)
    }

    /// Extract line and column from error message
    fn extract_line_col(error_str: &str) -> (usize, usize) {
        // Try patterns like "at line 5 column 10" or "line 5, column 10" or "5:10"

        // Pattern 1: "line X column Y"
        if let Some(line_pos) = error_str.find("line") {
            let after_line = &error_str[line_pos + 4..];
            if let Some(line_num) = after_line
                .trim()
                .split_whitespace()
                .next()
                .and_then(|s| s.trim_matches(|c: char| !c.is_numeric()).parse::<usize>().ok())
            {
                if let Some(col_pos) = after_line.find("column") {
                    if let Some(col_num) = after_line[col_pos + 6..]
                        .trim()
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.trim_matches(|c: char| !c.is_numeric()).parse::<usize>().ok())
                    {
                        return (line_num, col_num);
                    }
                }
                return (line_num, 1);
            }
        }

        // Pattern 2: "X:Y" format
        if let Some(colon_pos) = error_str.find(':') {
            let before = &error_str[..colon_pos];
            let after = &error_str[colon_pos + 1..];

            if let Some(line_num) = before.split_whitespace().last()
                .and_then(|s| s.parse::<usize>().ok()) {
                if let Some(col_num) = after.split_whitespace().next()
                    .and_then(|s| s.trim_matches(|c: char| !c.is_numeric()).parse::<usize>().ok()) {
                    return (line_num, col_num);
                }
            }
        }

        (0, 0)
    }

    /// Parse error with basic position extraction
    fn parse_error_with_position(text: &str, error_str: &str) -> (Range, String) {
        let (line, col) = Self::extract_line_col(error_str);

        if line > 0 {
            let line_idx = line.saturating_sub(1);
            let line_len = text.lines().nth(line_idx).map(|l| l.len()).unwrap_or(0);

            (
                Range {
                    start: Position {
                        line: line_idx as u32,
                        character: col.saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: line_idx as u32,
                        character: line_len as u32,
                    },
                },
                error_str.to_string(),
            )
        } else {
            // Default to line 0
            (
                Range {
                    start: Position { line: 0, character: 0 },
                    end: Position { line: 0, character: 0 },
                },
                error_str.to_string(),
            )
        }
    }
}

impl Default for DiagnosticsProvider {
    fn default() -> Self {
        Self::new()
    }
}
