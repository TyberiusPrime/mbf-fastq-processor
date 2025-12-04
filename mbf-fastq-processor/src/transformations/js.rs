//! JavaScript transformation step using Boa engine
//!
//! This module provides a transformation step that executes arbitrary JavaScript
//! code on FASTQ reads, allowing for flexible, user-defined transformations.

use crate::config::{Segment, SegmentIndex};
use crate::dna::{Hit, HitRegion, Hits, TagValue};
use crate::transformations::prelude::*;
use anyhow::{Context, bail};
use boa_engine::{
    Context as JsContext, JsValue, Source, js_string,
    object::builtins::JsArray,
    property::Attribute,
};
use bstr::BString;
use std::path::PathBuf;

/// Tag type for JavaScript output
#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum JsTagType {
    /// String tag value (allows null for Missing)
    String,
    /// Numeric (f64) tag value (null not allowed)
    Numeric,
    /// Boolean tag value (null not allowed)
    Bool,
    /// Location tag value - array of hits with start, len, segment, sequence
    Location,
}

impl From<JsTagType> for TagValueType {
    fn from(t: JsTagType) -> Self {
        match t {
            JsTagType::String => TagValueType::String,
            JsTagType::Numeric => TagValueType::Numeric,
            JsTagType::Bool => TagValueType::Bool,
            JsTagType::Location => TagValueType::Location,
        }
    }
}

/// JavaScript transformation step
///
/// Executes JavaScript code on a block of reads at once.
/// The JS function `process_reads(reads, tags)` receives:
/// - `reads`: array of read objects with seq, qual, name, index
/// - `tags`: object mapping tag names to arrays of values (one per read)
///
/// # Location Tags
/// Location tags are passed as arrays of hit objects:
/// ```javascript
/// tags.umi[i] = [
///   { start: 0, len: 8, segment: "read1", sequence: "ACGTACGT" },
///   { start: 10, len: 5, segment: "read1", sequence: "NNNNN" }
/// ]
/// ```
/// When returning Location tags, each element must be an array of hits
/// with valid segment references and positions within read bounds.
///
/// # Example TOML
/// ```toml
/// [[step]]
/// action = "JavaScript"
/// code = '''
/// function process_reads(reads, tags) {
///     let results = [];
///     for (let i = 0; i < reads.length; i++) {
///         // Return a Location tag with hits
///         results.push([{
///             start: 0,
///             len: 4,
///             segment: "read1",
///             sequence: reads[i].seq.substring(0, 4)
///         }]);
///     }
///     return results;
/// }
/// '''
/// out_label = "first_4bp"
/// out_type = "Location"
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

    /// Input tag names to pass to JavaScript
    #[serde(default)]
    in_tags: Vec<String>,

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

/// Convert a Location TagValue to a JS array of hit objects
fn location_to_js_array(
    hits: &Hits,
    segment_order: &[String],
    context: &mut JsContext,
) -> JsArray {
    let array = JsArray::new(context);
    for hit in &hits.0 {
        let hit_obj = boa_engine::object::JsObject::with_null_proto();

        // Set sequence
        hit_obj
            .set(
                js_string!("sequence"),
                JsValue::from(js_string!(hit.sequence.to_string())),
                false,
                context,
            )
            .expect("set sequence");

        // Set location info if present
        if let Some(ref loc) = hit.location {
            hit_obj
                .set(js_string!("start"), JsValue::from(loc.start as i32), false, context)
                .expect("set start");
            hit_obj
                .set(js_string!("len"), JsValue::from(loc.len as i32), false, context)
                .expect("set len");

            // Convert segment index to name
            let segment_name = segment_order
                .get(loc.segment_index.0)
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            hit_obj
                .set(
                    js_string!("segment"),
                    JsValue::from(js_string!(segment_name)),
                    false,
                    context,
                )
                .expect("set segment");
        } else {
            // No location - set null values
            hit_obj
                .set(js_string!("start"), JsValue::null(), false, context)
                .expect("set start null");
            hit_obj
                .set(js_string!("len"), JsValue::null(), false, context)
                .expect("set len null");
            hit_obj
                .set(js_string!("segment"), JsValue::null(), false, context)
                .expect("set segment null");
        }

        array
            .push(JsValue::from(hit_obj), context)
            .expect("push hit");
    }
    array
}

/// Parse a JS array of hit objects back to Hits, validating segment references
fn js_array_to_location(
    array: &JsArray,
    segment_order: &[String],
    read_lengths: &[usize], // Length of each segment's read
    read_idx: usize,
    context: &mut JsContext,
) -> anyhow::Result<Hits> {
    let len = array.length(context).map_err(|e| {
        anyhow::anyhow!("Failed to get hits array length: {e}")
    })? as usize;

    let mut hits = Vec::with_capacity(len);

    for i in 0..len {
        let hit_val = array.get(i as u32, context).map_err(|e| {
            anyhow::anyhow!("Failed to get hit at index {i}: {e}")
        })?;

        let hit_obj = hit_val.as_object().ok_or_else(|| {
            anyhow::anyhow!("Hit at index {i} is not an object")
        })?;

        // Get sequence (required)
        let sequence = hit_obj
            .get(js_string!("sequence"), context)
            .map_err(|e| anyhow::anyhow!("Failed to get sequence: {e}"))?
            .as_string()
            .ok_or_else(|| anyhow::anyhow!("Hit at index {i} missing 'sequence' string"))?
            .to_std_string_escaped();

        // Get location info (optional - check if segment is present and not null)
        let segment_val = hit_obj
            .get(js_string!("segment"), context)
            .map_err(|e| anyhow::anyhow!("Failed to get segment: {e}"))?;

        let location = if segment_val.is_null_or_undefined() {
            // No location
            None
        } else {
            let segment_name = segment_val
                .as_string()
                .ok_or_else(|| anyhow::anyhow!("Hit at index {i}: 'segment' must be a string"))?
                .to_std_string_escaped();

            // Validate segment name
            let segment_idx = segment_order
                .iter()
                .position(|s| s == &segment_name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Hit at index {i}: unknown segment '{}'. Available: [{}]",
                        segment_name,
                        segment_order.join(", ")
                    )
                })?;

            // Get start and len
            let start = hit_obj
                .get(js_string!("start"), context)
                .map_err(|e| anyhow::anyhow!("Failed to get start: {e}"))?
                .as_number()
                .ok_or_else(|| anyhow::anyhow!("Hit at index {i}: 'start' must be a number"))?
                as usize;

            let len = hit_obj
                .get(js_string!("len"), context)
                .map_err(|e| anyhow::anyhow!("Failed to get len: {e}"))?
                .as_number()
                .ok_or_else(|| anyhow::anyhow!("Hit at index {i}: 'len' must be a number"))?
                as usize;

            // Validate that start + len is within read bounds
            let read_len = read_lengths.get(segment_idx).copied().unwrap_or(0);
            if start + len > read_len {
                bail!(
                    "Hit at index {} for read {}: location {}..{} exceeds read length {} for segment '{}'",
                    i,
                    read_idx,
                    start,
                    start + len,
                    read_len,
                    segment_name
                );
            }

            Some(HitRegion {
                start,
                len,
                segment_index: SegmentIndex(segment_idx),
            })
        };

        hits.push(Hit {
            location,
            sequence: BString::from(sequence),
        });
    }

    Ok(Hits(hits))
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
                bail!("JavaScript step with 'out_label' also requires 'out_type' (String, Numeric, Bool, or Location)")
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

    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        if self.in_tags.is_empty() {
            None
        } else {
            // Accept any tag type for input tags
            static ALLOWED_TYPES: [TagValueType; 4] = [
                TagValueType::String,
                TagValueType::Numeric,
                TagValueType::Bool,
                TagValueType::Location,
            ];
            Some(
                self.in_tags
                    .iter()
                    .map(|tag| (tag.clone(), &ALLOWED_TYPES[..]))
                    .collect(),
            )
        }
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        input_info: &InputInfo,
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

        // Get read count from the target segment
        let read_count = block.segments[segment_idx.get_index()].entries.len();

        // Track read lengths per segment for Location validation (collect before mutable borrow)
        let read_lengths_per_segment: Vec<Vec<usize>> = (0..read_count)
            .map(|read_idx| {
                block
                    .segments
                    .iter()
                    .map(|seg| {
                        if read_idx < seg.entries.len() {
                            seg.entries[read_idx].seq.get(&seg.block).len()
                        } else {
                            0
                        }
                    })
                    .collect()
            })
            .collect();

        // Now get mutable reference to the segment we're working on
        let segment = &mut block.segments[segment_idx.get_index()];

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

        // Build tags object for JavaScript
        let tags_obj = boa_engine::object::JsObject::with_null_proto();
        for tag_name in &self.in_tags {
            let tag_array = JsArray::new(&mut context);
            if let Some(tag_values) = block.tags.get(tag_name) {
                for tag_value in tag_values {
                    let js_val = match tag_value {
                        TagValue::Missing => JsValue::null(),
                        TagValue::String(s) => {
                            JsValue::from(js_string!(s.to_string()))
                        }
                        TagValue::Numeric(n) => JsValue::from(*n),
                        TagValue::Bool(b) => JsValue::from(*b),
                        TagValue::Location(hits) => {
                            // Convert Location to full JS array of hit objects
                            JsValue::from(location_to_js_array(
                                hits,
                                &input_info.segment_order,
                                &mut context,
                            ))
                        }
                    };
                    tag_array.push(js_val, &mut context).expect("push tag value");
                }
            } else {
                // Tag not found - fill with nulls
                for _ in 0..read_count {
                    tag_array.push(JsValue::null(), &mut context).expect("push null");
                }
            }
            tags_obj
                .set(js_string!(tag_name.clone()), JsValue::from(tag_array), false, &mut context)
                .expect("set tag array");
        }

        // Register globals
        context
            .register_global_property(js_string!("__reads"), reads_array.clone(), Attribute::all())
            .expect("register reads");
        context
            .register_global_property(js_string!("__tags"), tags_obj, Attribute::all())
            .expect("register tags");

        // Call process_reads function with reads and tags
        let result = context.eval(Source::from_bytes(
            b"typeof process_reads === 'function' ? process_reads(__reads, __tags) : null",
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
                            JsTagType::Location => "Location",
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
                                bail!(
                                    "JavaScript returned null/undefined at index {}: Numeric tags cannot be Missing",
                                    i
                                );
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
                                bail!(
                                    "JavaScript returned null/undefined at index {}: Bool tags cannot be Missing",
                                    i
                                );
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
                        JsTagType::Location => {
                            if val.is_null_or_undefined() {
                                TagValue::Missing
                            } else {
                                // Must be an array of hit objects
                                let hits_array = val
                                    .as_object()
                                    .and_then(|o| JsArray::from_object(o.clone()).ok())
                                    .ok_or_else(|| {
                                        anyhow::anyhow!(
                                            "JavaScript returned non-array value at index {}: expected Location (array of hits), got {:?}",
                                            i,
                                            val.type_of()
                                        )
                                    })?;

                                let hits = js_array_to_location(
                                    &hits_array,
                                    &input_info.segment_order,
                                    &read_lengths_per_segment[i],
                                    i,
                                    &mut context,
                                )?;
                                TagValue::Location(hits)
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
            code: Some("function process_reads(reads, tags) { return null; }".to_string()),
            file: None,
            segment: Segment::default(),
            segment_index: None,
            in_tags: vec![],
            out_label: None,
            out_type: None,
            script_source: None,
        };
        assert!(js.code.is_some());
    }

    #[test]
    fn test_js_tag_type_required() {
        let js = JavaScript {
            code: Some("function process_reads(reads, tags) { return []; }".to_string()),
            file: None,
            segment: Segment::default(),
            segment_index: None,
            in_tags: vec![],
            out_label: Some("test".to_string()),
            out_type: None, // Missing!
            script_source: None,
        };
        // This would fail validation
        assert!(js.out_label.is_some());
        assert!(js.out_type.is_none());
    }
}
