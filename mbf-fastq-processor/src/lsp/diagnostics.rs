//! Diagnostics provider for validating TOML configuration

use tower_lsp::lsp_types::*;

/// Provides diagnostics (errors/warnings) for TOML documents
pub struct DiagnosticsProvider;

impl DiagnosticsProvider {
    pub fn new() -> Self {
        Self
    }

    /// Get diagnostics for a TOML document
    pub fn get_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // First, try to parse the TOML
        match toml::from_str::<toml::Value>(text) {
            Ok(_toml_value) => {
                // TOML is valid, now try to parse as Config and validate
                match eserde::toml::from_str::<mbf_fastq_processor::config::Config>(text) {
                    Ok(mut config) => {
                        // Run validation (skip file existence checks)
                        if let Err(e) = config.check_for_validation() {
                            // Parse error message and create diagnostic
                            let error_msg = format!("{:?}", e);
                            diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                source: Some("mbf-fastq-processor".to_string()),
                                message: error_msg,
                                ..Default::default()
                            });
                        }
                    }
                    Err(e) => {
                        // Config parsing error
                        let (line, message) = Self::parse_toml_error(&e);
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line.saturating_sub(1) as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: line.saturating_sub(1) as u32,
                                    character: u32::MAX,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("mbf-fastq-processor".to_string()),
                            message,
                            ..Default::default()
                        });
                    }
                }
            }
            Err(e) => {
                // TOML syntax error
                let (line, message) = Self::parse_toml_error(&e);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line.saturating_sub(1) as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line.saturating_sub(1) as u32,
                            character: u32::MAX,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("toml".to_string()),
                    message,
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// Parse TOML error message to extract line number and message
    fn parse_toml_error<E: std::fmt::Display>(error: &E) -> (usize, String) {
        let error_str = error.to_string();

        // Try to extract line number from error message
        // Format is usually "... at line X ..." or "... line X ..."
        if let Some(line_pos) = error_str.find("line") {
            let after_line = &error_str[line_pos + 4..];
            if let Some(num_str) = after_line
                .trim()
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<usize>().ok())
            {
                return (num_str, error_str);
            }
        }

        // Default to line 1 if we can't parse the line number
        (1, error_str)
    }
}

impl Default for DiagnosticsProvider {
    fn default() -> Self {
        Self::new()
    }
}
