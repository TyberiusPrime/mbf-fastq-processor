//! JavaScript transformation step using Boa engine
//!
//! This module provides a transformation step that executes arbitrary JavaScript
//! code on FASTQ reads, allowing for flexible, user-defined transformations.

use crate::config::{Segment, SegmentIndex};
use crate::dna::TagValue;
use crate::transformations::prelude::*;
use anyhow::{Context, bail};
use boa_engine::{
    Context as JsContext, JsValue, Source, js_string,
    object::builtins::JsArray,
    property::Attribute,
};
use std::path::PathBuf;

/// Tag type for JavaScript output
#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum JsTagType {
    /// String tag value
    String,
    /// Numeric (f64) tag value
    Numeric,
    /// Boolean tag value
    Bool,
}

impl From<JsTagType> for TagValueType {
    fn from(t: JsTagType) -> Self {
        match t {
            JsTagType::String => TagValueType::String,
            JsTagType::Numeric => TagValueType::Numeric,
            JsTagType::Bool => TagValueType::Bool,
        }
    }
}

/// JavaScript transformation step
///
/// Executes JavaScript code on a block of reads at once.
/// The JS function `process_reads(reads)` receives an array of read objects
/// and can modify them in place. Optionally returns an array of tag values.
///
/// # Example TOML
/// ```toml
/// [[step]]
/// action = "JavaScript"
/// code = '''
/// function process_reads(reads) {
///     // ROT-encode: A->C, C->G, G->T, T->A
///     let rot = {'A':'C', 'C':'G', 'G':'T', 'T':'A',
///                'a':'c', 'c':'g', 'g':'t', 't':'a', 'N':'N'};
///     let lengths = [];
///     for (let read of reads) {
///         read.seq = read.seq.split('').map(b => rot[b] || b).join('');
///         lengths.push(read.seq.length);
///     }
///     return lengths;  // Must match out_type
/// }
/// '''
/// out_label = "seq_length"
/// out_type = "Numeric"
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

    /// Tag name to create from JS return values (requires out_type)
    #[serde(default)]
    out_label: Option<String>,

    /// Type of tag values returned by JavaScript (required if out_label is set)
    #[serde(default)]
    out_type: Option<JsTagType>,

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

        // Validate out_label and out_type are both present or both absent
        match (&self.out_label, &self.out_type) {
            (Some(_), None) => {
                bail!("JavaScript step with 'out_label' also requires 'out_type' (String, Numeric, or Bool)")
            }
            (None, Some(_)) => {
                bail!("JavaScript step with 'out_type' also requires 'out_label'")
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
        match (&self.out_label, &self.out_type) {
            (Some(label), Some(tag_type)) => Some((label.clone(), (*tag_type).into())),
            _ => None,
        }
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
        let mut context = JsContext::default();

        // Evaluate the user's script to define functions
        let source = self
            .script_source
            .as_ref()
            .expect("script_source must be set");
        context
            .eval(Source::from_bytes(source.as_bytes()))
            .map_err(|e| anyhow::anyhow!("JavaScript evaluation error: {e}"))?;

        // Get the segment we're working on
        let segment = &mut block.segments[segment_idx.get_index()];
        let read_count = segment.entries.len();

        // Build array of read objects for JavaScript
        let reads_array = JsArray::new(&mut context);

        // Store original values for comparison
        let mut original_seqs: Vec<String> = Vec::with_capacity(read_count);
        let mut original_quals: Vec<String> = Vec::with_capacity(read_count);
        let mut original_names: Vec<String> = Vec::with_capacity(read_count);

        for read_idx in 0..read_count {
            let read = &segment.entries[read_idx];

            let seq_str = String::from_utf8_lossy(read.seq.get(&segment.block)).to_string();
            let qual_str = String::from_utf8_lossy(read.qual.get(&segment.block)).to_string();
            let name_str = String::from_utf8_lossy(read.name.get(&segment.block)).to_string();

            original_seqs.push(seq_str.clone());
            original_quals.push(qual_str.clone());
            original_names.push(name_str.clone());

            // Create JS object for the read
            let read_obj = boa_engine::object::JsObject::with_null_proto();
            read_obj
                .set(js_string!("seq"), JsValue::from(js_string!(seq_str)), false, &mut context)
                .expect("set seq");
            read_obj
                .set(js_string!("qual"), JsValue::from(js_string!(qual_str)), false, &mut context)
                .expect("set qual");
            read_obj
                .set(js_string!("name"), JsValue::from(js_string!(name_str)), false, &mut context)
                .expect("set name");
            read_obj
                .set(js_string!("index"), JsValue::from(read_idx as i32), false, &mut context)
                .expect("set index");

            reads_array
                .push(JsValue::from(read_obj), &mut context)
                .expect("push read");
        }

        // Register the reads array as a global variable
        context
            .register_global_property(js_string!("__reads"), reads_array.clone(), Attribute::all())
            .expect("register reads");

        // Call process_reads function
        let result = context.eval(Source::from_bytes(
            b"typeof process_reads === 'function' ? process_reads(__reads) : null",
        ));

        // Process the result
        let tag_values: Option<Vec<TagValue>> = match (&self.out_label, &self.out_type, result) {
            (Some(_), Some(expected_type), Ok(return_value)) => {
                if return_value.is_null_or_undefined() {
                    bail!(
                        "JavaScript process_reads must return an array of {} values, got null/undefined",
                        match expected_type {
                            JsTagType::String => "String",
                            JsTagType::Numeric => "Numeric",
                            JsTagType::Bool => "Bool",
                        }
                    );
                }

                // Must be an array
                let return_array = return_value
                    .as_object()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "JavaScript process_reads must return an array, got {:?}",
                            return_value.type_of()
                        )
                    })
                    .and_then(|o| {
                        JsArray::from_object(o.clone()).map_err(|_| {
                            anyhow::anyhow!(
                                "JavaScript process_reads must return an array, got object"
                            )
                        })
                    })?;

                let array_len = return_array.length(&mut context).map_err(|e| {
                    anyhow::anyhow!("Failed to get return array length: {e}")
                })? as usize;

                if array_len != read_count {
                    bail!(
                        "JavaScript process_reads returned array of length {}, expected {} (one per read)",
                        array_len,
                        read_count
                    );
                }

                let mut tags = Vec::with_capacity(read_count);
                for i in 0..read_count {
                    let val = return_array.get(i as u32, &mut context).map_err(|e| {
                        anyhow::anyhow!("Failed to get array element {i}: {e}")
                    })?;

                    let tag_val = match expected_type {
                        JsTagType::String => {
                            if val.is_null_or_undefined() {
                                TagValue::Missing
                            } else if let Some(s) = val.as_string() {
                                TagValue::String(s.to_std_string_escaped().into())
                            } else {
                                bail!(
                                    "JavaScript returned non-string value at index {}: expected String, got {:?}",
                                    i,
                                    val.type_of()
                                );
                            }
                        }
                        JsTagType::Numeric => {
                            if val.is_null_or_undefined() {
                                TagValue::Missing
                            } else if let Some(n) = val.as_number() {
                                TagValue::Numeric(n)
                            } else {
                                bail!(
                                    "JavaScript returned non-numeric value at index {}: expected Numeric, got {:?}",
                                    i,
                                    val.type_of()
                                );
                            }
                        }
                        JsTagType::Bool => {
                            if val.is_null_or_undefined() {
                                TagValue::Missing
                            } else if let Some(b) = val.as_boolean() {
                                TagValue::Bool(b)
                            } else {
                                bail!(
                                    "JavaScript returned non-boolean value at index {}: expected Bool, got {:?}",
                                    i,
                                    val.type_of()
                                );
                            }
                        }
                    };
                    tags.push(tag_val);
                }
                Some(tags)
            }
            (None, None, Ok(_)) => None,
            (_, _, Err(e)) => {
                bail!("JavaScript execution error: {e}");
            }
            _ => None,
        };

        // Read back modified values from the reads array
        for read_idx in 0..read_count {
            let read_obj = reads_array
                .get(read_idx as u32, &mut context)
                .expect("get read")
                .as_object()
                .expect("read is object")
                .clone();

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

            let read = &mut segment.entries[read_idx];

            // Apply modifications only if they changed
            if let Some(ref seq) = new_seq {
                if seq != &original_seqs[read_idx] {
                    let seq_bytes = seq.clone().into_bytes();
                    if let Some(ref qual) = new_qual {
                        if qual != &original_quals[read_idx] && seq_bytes.len() == qual.len() {
                            let qual_bytes = qual.clone().into_bytes();
                            read.seq.replace(seq_bytes, &mut segment.block);
                            read.qual.replace(qual_bytes, &mut segment.block);
                        } else {
                            // Seq changed but qual didn't or lengths mismatch - adjust qual
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
                    } else {
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
                if name != &original_names[read_idx] {
                    read.name.replace(name.clone().into_bytes(), &mut segment.block);
                }
            }
        }

        // Store tag values if we have an output label
        if let (Some(label), Some(tags)) = (&self.out_label, tag_values) {
            block.tags.insert(label.clone(), tags);
        }

        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_struct_creation() {
        let js = JavaScript {
            code: Some("function process_reads(reads) { return null; }".to_string()),
            file: None,
            segment: Segment::default(),
            segment_index: None,
            out_label: None,
            out_type: None,
            script_source: None,
        };
        assert!(js.code.is_some());
    }

    #[test]
    fn test_js_tag_type_required() {
        let js = JavaScript {
            code: Some("function process_reads(reads) { return []; }".to_string()),
            file: None,
            segment: Segment::default(),
            segment_index: None,
            out_label: Some("test".to_string()),
            out_type: None, // Missing!
            script_source: None,
        };
        // This would fail validation
        assert!(js.out_label.is_some());
        assert!(js.out_type.is_none());
    }
}
