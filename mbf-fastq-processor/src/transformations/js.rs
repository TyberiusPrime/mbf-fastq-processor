//! JavaScript transformation step using Boa engine
//!
//! This module provides a transformation step that executes arbitrary JavaScript
//! code on FASTQ reads, allowing for flexible, user-defined transformations.

use crate::config::{Segment, SegmentIndex};
use crate::dna::TagValue;
use crate::transformations::prelude::*;
use anyhow::{Context, bail};
use boa_engine::{Context as JsContext, JsValue, Source, js_string, property::Attribute};
use std::path::PathBuf;

/// JavaScript transformation step
///
/// Executes JavaScript code on each read (or block of reads).
/// The JS code has access to read data and can modify sequences, qualities, and names.
///
/// # Example TOML
/// ```toml
/// [[step]]
/// action = "JavaScript"
/// code = '''
/// function process_read(read) {
///     // ROT-encode: A->C, C->G, G->T, T->A
///     let rot = { 'A': 'C', 'C': 'G', 'G': 'T', 'T': 'A',
///                 'a': 'c', 'c': 'g', 'g': 't', 't': 'a' };
///     read.seq = read.seq.split('').map(b => rot[b] || b).join('');
///     return read.seq.length;  // Optional: return value becomes a tag
/// }
/// '''
/// out_label = "seq_length"  # Optional: store return value as tag
/// ```
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JavaScript {
    /// Inline JavaScript code to execute
    #[serde(default)]
    code: Option<String>,

    /// Path to a JavaScript file to execute
    #[serde(default)]
    file: Option<String>,

    /// Which segment to operate on (default: read1)
    #[serde(default)]
    segment: Segment,

    /// Internal: resolved segment index
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    /// Optional tag name to create from JS return values
    #[serde(default)]
    out_label: Option<String>,

    /// Cached script source (loaded from file during validation)
    #[serde(skip)]
    script_source: Option<String>,
}

impl Step for JavaScript {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);

        // Validate that we have either code or file, but not both
        match (&self.code, &self.file) {
            (None, None) => bail!("JavaScript step requires either 'code' or 'file' parameter"),
            (Some(_), Some(_)) => {
                bail!("JavaScript step cannot have both 'code' and 'file' parameters")
            }
            _ => {}
        }

        // Load script source
        self.script_source = Some(match (&self.code, &self.file) {
            (Some(code), None) => code.clone(),
            (None, Some(path)) => {
                let path = PathBuf::from(path);
                std::fs::read_to_string(&path).with_context(|| {
                    format!("Failed to read JavaScript file: {}", path.display())
                })?
            }
            _ => unreachable!(),
        });

        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        self.out_label
            .as_ref()
            .map(|label| (label.clone(), TagValueType::String))
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_idx = self
            .segment_index
            .expect("segment_index must be set during initialization");

        // Create a fresh JS context for this block
        // (Boa contexts are not Send, so we can't cache them across thread boundaries)
        let mut context = JsContext::default();

        // Evaluate the user's script to define functions
        let source = self
            .script_source
            .as_ref()
            .expect("script_source must be set");
        context
            .eval(Source::from_bytes(source.as_bytes()))
            .map_err(|e| anyhow::anyhow!("JavaScript evaluation error: {e}"))?;

        // Prepare tag storage if needed
        let mut tag_values: Vec<TagValue> = Vec::new();
        let has_out_label = self.out_label.is_some();

        // Get the segment we're working on
        let segment = &mut block.segments[segment_idx.get_index()];
        let read_count = segment.entries.len();

        // Process each read
        for read_idx in 0..read_count {
            let read = &mut segment.entries[read_idx];

            // Convert read data to JS-accessible format
            let seq_bytes = read.seq.get(&segment.block).to_vec();
            let qual_bytes = read.qual.get(&segment.block).to_vec();
            let name_bytes = read.name.get(&segment.block).to_vec();

            let seq_str = String::from_utf8_lossy(&seq_bytes).to_string();
            let qual_str = String::from_utf8_lossy(&qual_bytes).to_string();
            let name_str = String::from_utf8_lossy(&name_bytes).to_string();

            // Create JS object for the read
            let read_obj = boa_engine::object::JsObject::with_null_proto();
            read_obj
                .set(
                    js_string!("seq"),
                    JsValue::from(js_string!(seq_str.clone())),
                    false,
                    &mut context,
                )
                .expect("set seq");
            read_obj
                .set(
                    js_string!("qual"),
                    JsValue::from(js_string!(qual_str.clone())),
                    false,
                    &mut context,
                )
                .expect("set qual");
            read_obj
                .set(
                    js_string!("name"),
                    JsValue::from(js_string!(name_str.clone())),
                    false,
                    &mut context,
                )
                .expect("set name");
            read_obj
                .set(
                    js_string!("index"),
                    JsValue::from(read_idx as i32),
                    false,
                    &mut context,
                )
                .expect("set index");

            // Set the read object as a global variable
            context
                .register_global_property(js_string!("read"), read_obj.clone(), Attribute::all())
                .expect("register read");

            // Call the process_read function if it exists
            let result = context.eval(Source::from_bytes(
                b"typeof process_read === 'function' ? process_read(read) : null",
            ));

            // Handle the result
            match result {
                Ok(return_value) => {
                    // Extract modified values from the read object
                    let new_seq = read_obj
                        .get(js_string!("seq"), &mut context)
                        .ok()
                        .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()));
                    let new_qual = read_obj
                        .get(js_string!("qual"), &mut context)
                        .ok()
                        .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()));
                    let new_name = read_obj
                        .get(js_string!("name"), &mut context)
                        .ok()
                        .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()));

                    // Apply modifications only if they changed
                    if let Some(ref seq) = new_seq {
                        if seq != &seq_str {
                            let seq_bytes = seq.clone().into_bytes();
                            if let Some(ref qual) = new_qual {
                                if qual != &qual_str {
                                    let qual_bytes = qual.clone().into_bytes();
                                    if seq_bytes.len() == qual_bytes.len() {
                                        read.seq.replace(seq_bytes, &mut segment.block);
                                        read.qual.replace(qual_bytes, &mut segment.block);
                                    }
                                } else {
                                    // Seq changed but qual didn't - adjust qual to match
                                    let original_qual = read.qual.get(&segment.block);
                                    let new_qual = if seq_bytes.len() <= original_qual.len() {
                                        original_qual[..seq_bytes.len()].to_vec()
                                    } else {
                                        let mut q = original_qual.to_vec();
                                        q.resize(seq_bytes.len(), b'I'); // Fill with high quality
                                        q
                                    };
                                    read.seq.replace(seq_bytes, &mut segment.block);
                                    read.qual.replace(new_qual, &mut segment.block);
                                }
                            } else {
                                // Seq changed but qual wasn't read back - keep original qual
                                let original_qual = read.qual.get(&segment.block);
                                let new_qual = if seq_bytes.len() <= original_qual.len() {
                                    original_qual[..seq_bytes.len()].to_vec()
                                } else {
                                    let mut q = original_qual.to_vec();
                                    q.resize(seq_bytes.len(), b'I');
                                    q
                                };
                                read.seq.replace(seq_bytes, &mut segment.block);
                                read.qual.replace(new_qual, &mut segment.block);
                            }
                        }
                    }

                    if let Some(ref name) = new_name {
                        if name != &name_str {
                            read.name
                                .replace(name.clone().into_bytes(), &mut segment.block);
                        }
                    }

                    // Handle tag output
                    if has_out_label {
                        let tag_val = if return_value.is_null_or_undefined() {
                            TagValue::Missing
                        } else if let Some(s) = return_value.as_string() {
                            TagValue::String(s.to_std_string_escaped().into())
                        } else if let Some(n) = return_value.as_number() {
                            TagValue::Numeric(n)
                        } else if let Some(b) = return_value.as_boolean() {
                            TagValue::Bool(b)
                        } else {
                            TagValue::String(
                                return_value
                                    .to_string(&mut context)
                                    .map(|s| s.to_std_string_escaped())
                                    .unwrap_or_default()
                                    .into(),
                            )
                        };
                        tag_values.push(tag_val);
                    }
                }
                Err(e) => {
                    // Log error but continue processing
                    log::warn!("JavaScript error on read {read_idx}: {e}");
                    if has_out_label {
                        tag_values.push(TagValue::Missing);
                    }
                }
            }
        }

        // Store tag values if we have an output label
        if let Some(label) = &self.out_label {
            block.tags.insert(label.clone(), tag_values);
        }

        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        // Run in serial mode for simplicity
        // (could potentially parallelize by creating context per thread)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_struct_creation() {
        // Verify the struct can be created
        let js = JavaScript {
            code: Some("function process_read(read) { return null; }".to_string()),
            file: None,
            segment: Segment::default(),
            segment_index: None,
            out_label: None,
            script_source: None,
        };
        assert!(js.code.is_some());
    }
}
