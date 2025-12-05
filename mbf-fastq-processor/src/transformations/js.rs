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
use std::collections::{BTreeMap, HashMap};
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
/// The JS function `process_reads(reads, tags, state)` receives:
/// - `reads`: array of read objects with seq, qual, name, index
/// - `tags`: object mapping tag names to arrays of values (one per read)
/// - `state`: object containing state from previous block (null on first block)
///
/// # Return Value
/// With single output tag (out_label/out_type): return array of values
/// With multiple output tags (out_tags): return object with tag names as keys
///
/// To persist state across blocks, include `_state` key in return object.
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
/// # Example TOML (single tag)
/// ```toml
/// [[step]]
/// action = "JavaScript"
/// code = '''
/// function process_reads(reads, tags, state) {
///     let results = [];
///     for (let read of reads) {
///         results.push(read.seq.length);
///     }
///     return results;
/// }
/// '''
/// out_label = "seq_length"
/// out_type = "Numeric"
/// ```
///
/// # Example TOML (multiple tags with state)
/// ```toml
/// [[step]]
/// action = "JavaScript"
/// code = '''
/// function process_reads(reads, tags, state) {
///     let count = state ? state.count : 0;
///     let lengths = [];
///     let names = [];
///     for (let read of reads) {
///         lengths.push(read.seq.length);
///         names.push(read.name);
///         count++;
///     }
///     return {
///         seq_length: lengths,
///         read_name: names,
///         _state: { count: count }
///     };
/// }
/// '''
/// out_tags = { seq_length = "Numeric", read_name = "String" }
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

    /// Single output tag name (requires out_type, mutually exclusive with out_tags)
    #[serde(default)]
    out_label: Option<String>,

    /// Type of single output tag (required if out_label is set)
    #[serde(default)]
    out_type: Option<JsTagType>,

    /// Multiple output tags: maps tag names to their types
    /// Mutually exclusive with out_label/out_type
    #[serde(default)]
    out_tags: BTreeMap<String, JsTagType>,

    /// Cached script source (loaded from file during validation)
    #[serde(skip)]
    script_source: Option<String>,

    /// Persisted state across blocks (stored as JSON)
    #[serde(skip)]
    state: Option<serde_json::Value>,
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

/// Convert serde_json::Value to JsValue
fn json_to_js(value: &serde_json::Value, context: &mut JsContext) -> JsValue {
    match value {
        serde_json::Value::Null => JsValue::null(),
        serde_json::Value::Bool(b) => JsValue::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsValue::from(i as f64)
            } else if let Some(f) = n.as_f64() {
                JsValue::from(f)
            } else {
                JsValue::null()
            }
        }
        serde_json::Value::String(s) => JsValue::from(js_string!(s.clone())),
        serde_json::Value::Array(arr) => {
            let js_arr = JsArray::new(context);
            for item in arr {
                js_arr
                    .push(json_to_js(item, context), context)
                    .expect("push array item");
            }
            JsValue::from(js_arr)
        }
        serde_json::Value::Object(obj) => {
            let js_obj = boa_engine::object::JsObject::with_null_proto();
            for (key, val) in obj {
                js_obj
                    .set(js_string!(key.clone()), json_to_js(val, context), false, context)
                    .expect("set object property");
            }
            JsValue::from(js_obj)
        }
    }
}

/// Convert JsValue to serde_json::Value
fn js_to_json(value: &JsValue, context: &mut JsContext) -> serde_json::Value {
    if value.is_null_or_undefined() {
        serde_json::Value::Null
    } else if let Some(b) = value.as_boolean() {
        serde_json::Value::Bool(b)
    } else if let Some(n) = value.as_number() {
        serde_json::json!(n)
    } else if let Some(s) = value.as_string() {
        serde_json::Value::String(s.to_std_string_escaped())
    } else if let Some(obj) = value.as_object() {
        if let Ok(arr) = JsArray::from_object(obj.clone()) {
            let len = arr.length(context).unwrap_or(0) as usize;
            let mut result = Vec::with_capacity(len);
            for i in 0..len {
                if let Ok(item) = arr.get(i as u32, context) {
                    result.push(js_to_json(&item, context));
                }
            }
            serde_json::Value::Array(result)
        } else {
            // Regular object
            let mut map = serde_json::Map::new();
            // Get own property keys
            if let Ok(keys) = obj.own_property_keys(context) {
                for key in keys {
                    let key_str = key.to_string();
                    if let Ok(val) = obj.get(key, context) {
                        map.insert(key_str, js_to_json(&val, context));
                    }
                }
            }
            serde_json::Value::Object(map)
        }
    } else {
        serde_json::Value::Null
    }
}

/// Parse a JS array of values into TagValue vec, given the expected type
fn parse_tag_array(
    array: &JsArray,
    expected_type: JsTagType,
    tag_name: &str,
    read_count: usize,
    read_lengths_per_segment: &[Vec<usize>],
    segment_order: &[String],
    context: &mut JsContext,
) -> anyhow::Result<Vec<TagValue>> {
    let array_len = array.length(context).map_err(|e| {
        anyhow::anyhow!("Failed to get array length for tag '{}': {e}", tag_name)
    })? as usize;

    if array_len != read_count {
        bail!(
            "Tag '{}' has {} elements, expected {} (one per read)",
            tag_name,
            array_len,
            read_count
        );
    }

    let mut tags = Vec::with_capacity(read_count);
    for i in 0..read_count {
        let val = array.get(i as u32, context).map_err(|e| {
            anyhow::anyhow!("Failed to get element {} for tag '{}': {e}", i, tag_name)
        })?;

        let tag_val = match expected_type {
            JsTagType::String => {
                if val.is_null_or_undefined() {
                    TagValue::Missing
                } else if let Some(s) = val.as_string() {
                    TagValue::String(s.to_std_string_escaped().into())
                } else {
                    bail!(
                        "Tag '{}' at index {}: expected String, got {:?}",
                        tag_name,
                        i,
                        val.type_of()
                    );
                }
            }
            JsTagType::Numeric => {
                if val.is_null_or_undefined() {
                    bail!(
                        "Tag '{}' at index {}: Numeric tags cannot be null/undefined",
                        tag_name,
                        i
                    );
                } else if let Some(n) = val.as_number() {
                    TagValue::Numeric(n)
                } else {
                    bail!(
                        "Tag '{}' at index {}: expected Numeric, got {:?}",
                        tag_name,
                        i,
                        val.type_of()
                    );
                }
            }
            JsTagType::Bool => {
                if val.is_null_or_undefined() {
                    bail!(
                        "Tag '{}' at index {}: Bool tags cannot be null/undefined",
                        tag_name,
                        i
                    );
                } else if let Some(b) = val.as_boolean() {
                    TagValue::Bool(b)
                } else {
                    bail!(
                        "Tag '{}' at index {}: expected Bool, got {:?}",
                        tag_name,
                        i,
                        val.type_of()
                    );
                }
            }
            JsTagType::Location => {
                if val.is_null_or_undefined() {
                    TagValue::Missing
                } else {
                    let hits_array = val
                        .as_object()
                        .and_then(|o| JsArray::from_object(o.clone()).ok())
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Tag '{}' at index {}: expected Location (array of hits), got {:?}",
                                tag_name,
                                i,
                                val.type_of()
                            )
                        })?;

                    let hits = js_array_to_location(
                        &hits_array,
                        segment_order,
                        &read_lengths_per_segment[i],
                        i,
                        context,
                    )?;
                    TagValue::Location(hits)
                }
            }
        };
        tags.push(tag_val);
    }
    Ok(tags)
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

        // Validate output tag configuration
        let has_single_tag = self.out_label.is_some() || self.out_type.is_some();
        let has_multi_tags = !self.out_tags.is_empty();

        if has_single_tag && has_multi_tags {
            bail!("JavaScript step cannot use both out_label/out_type and out_tags. Use one or the other.");
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

    fn declares_tag_types(&self) -> Vec<(String, TagValueType)> {
        // If using out_tags (multi-tag mode), return all of them
        if !self.out_tags.is_empty() {
            self.out_tags
                .iter()
                .map(|(label, tag_type)| (label.clone(), (*tag_type).into()))
                .collect()
        } else {
            // Fall back to single tag mode
            self.declares_tag_type().into_iter().collect()
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

        // Register state (null if no previous state)
        let state_val = match &self.state {
            Some(json_state) => json_to_js(json_state, &mut context),
            None => JsValue::null(),
        };
        context
            .register_global_property(js_string!("__state"), state_val, Attribute::all())
            .expect("register state");

        // Call process_reads function with reads, tags, and state
        let result = context.eval(Source::from_bytes(
            b"typeof process_reads === 'function' ? process_reads(__reads, __tags, __state) : null",
        ));

        // Process the result - handle both single-tag mode and multi-tag mode
        let return_value = result.map_err(|e| anyhow::anyhow!("JavaScript execution error: {e}"))?;

        // HashMap to collect all tags (for both single and multi-tag modes)
        let mut all_tags: HashMap<String, Vec<TagValue>> = HashMap::new();

        // Determine which mode we're in
        let is_multi_tag_mode = !self.out_tags.is_empty();

        if is_multi_tag_mode {
            // Multi-tag mode: expect object return { tag1: [...], tag2: [...], _state: {...} }
            if return_value.is_null_or_undefined() {
                bail!("JavaScript process_reads must return an object with tag arrays, got null/undefined");
            }

            let return_obj = return_value.as_object().ok_or_else(|| {
                anyhow::anyhow!(
                    "JavaScript process_reads must return an object in multi-tag mode, got {:?}",
                    return_value.type_of()
                )
            })?;

            // Check if it's an array (not allowed in multi-tag mode)
            if JsArray::from_object(return_obj.clone()).is_ok() {
                bail!("JavaScript process_reads must return an object (not array) in multi-tag mode");
            }

            // Extract _state if present
            let state_key = js_string!("_state");
            if let Ok(state_val) = return_obj.get(state_key, &mut context) {
                if !state_val.is_null_or_undefined() {
                    self.state = Some(js_to_json(&state_val, &mut context));
                }
            }

            // Parse each declared output tag
            for (tag_name, expected_type) in &self.out_tags {
                let tag_array_val = return_obj
                    .get(js_string!(tag_name.clone()), &mut context)
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to get tag '{}' from result: {e}", tag_name)
                    })?;

                if tag_array_val.is_null_or_undefined() {
                    bail!(
                        "JavaScript result missing required tag '{}'. Return object must contain arrays for all declared out_tags.",
                        tag_name
                    );
                }

                let tag_array = tag_array_val
                    .as_object()
                    .and_then(|o| JsArray::from_object(o.clone()).ok())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Tag '{}' must be an array, got {:?}",
                            tag_name,
                            tag_array_val.type_of()
                        )
                    })?;

                let tags = parse_tag_array(
                    &tag_array,
                    *expected_type,
                    tag_name,
                    read_count,
                    &read_lengths_per_segment,
                    &input_info.segment_order,
                    &mut context,
                )?;
                all_tags.insert(tag_name.clone(), tags);
            }
        } else if let (Some(label), Some(expected_type)) = (&self.out_label, &self.out_type) {
            // Single-tag mode: expect array return [...] or object with _state
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

            let return_obj = return_value.as_object().ok_or_else(|| {
                anyhow::anyhow!(
                    "JavaScript process_reads must return an array or object, got {:?}",
                    return_value.type_of()
                )
            })?;

            // Check if it's an array or object with _state
            if let Ok(return_array) = JsArray::from_object(return_obj.clone()) {
                // Direct array return (backward compatible)
                let tags = parse_tag_array(
                    &return_array,
                    *expected_type,
                    label,
                    read_count,
                    &read_lengths_per_segment,
                    &input_info.segment_order,
                    &mut context,
                )?;
                all_tags.insert(label.clone(), tags);
            } else {
                // Object return - must have the tag name as key, may have _state
                let state_key = js_string!("_state");
                if let Ok(state_val) = return_obj.get(state_key, &mut context) {
                    if !state_val.is_null_or_undefined() {
                        self.state = Some(js_to_json(&state_val, &mut context));
                    }
                }

                let tag_array_val = return_obj
                    .get(js_string!(label.clone()), &mut context)
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to get tag '{}' from result: {e}", label)
                    })?;

                if tag_array_val.is_null_or_undefined() {
                    bail!(
                        "JavaScript result object missing tag '{}'. Either return an array directly or an object with the tag name as key.",
                        label
                    );
                }

                let tag_array = tag_array_val
                    .as_object()
                    .and_then(|o| JsArray::from_object(o.clone()).ok())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Tag '{}' must be an array, got {:?}",
                            label,
                            tag_array_val.type_of()
                        )
                    })?;

                let tags = parse_tag_array(
                    &tag_array,
                    *expected_type,
                    label,
                    read_count,
                    &read_lengths_per_segment,
                    &input_info.segment_order,
                    &mut context,
                )?;
                all_tags.insert(label.clone(), tags);
            }
        } else {
            // No output tags declared - but check for _state in return value if it's an object
            if let Some(return_obj) = return_value.as_object() {
                if JsArray::from_object(return_obj.clone()).is_err() {
                    // It's an object, not an array - check for _state
                    let state_key = js_string!("_state");
                    if let Ok(state_val) = return_obj.get(state_key, &mut context) {
                        if !state_val.is_null_or_undefined() {
                            self.state = Some(js_to_json(&state_val, &mut context));
                        }
                    }
                }
            }
        }

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

        // Store all collected tag values
        for (label, tags) in all_tags {
            block.tags.insert(label, tags);
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
            code: Some("function process_reads(reads, tags, state) { return null; }".to_string()),
            file: None,
            segment: Segment::default(),
            segment_index: None,
            in_tags: vec![],
            out_label: None,
            out_type: None,
            out_tags: BTreeMap::new(),
            script_source: None,
            state: None,
        };
        assert!(js.code.is_some());
    }

    #[test]
    fn test_js_tag_type_required() {
        let js = JavaScript {
            code: Some("function process_reads(reads, tags, state) { return []; }".to_string()),
            file: None,
            segment: Segment::default(),
            segment_index: None,
            in_tags: vec![],
            out_label: Some("test".to_string()),
            out_type: None, // Missing!
            out_tags: BTreeMap::new(),
            script_source: None,
            state: None,
        };
        // This would fail validation
        assert!(js.out_label.is_some());
        assert!(js.out_type.is_none());
    }

    #[test]
    fn test_js_multi_tag_mode() {
        let mut out_tags = BTreeMap::new();
        out_tags.insert("length".to_string(), JsTagType::Numeric);
        out_tags.insert("name".to_string(), JsTagType::String);

        let js = JavaScript {
            code: Some("function process_reads(reads, tags, state) { return { length: [], name: [] }; }".to_string()),
            file: None,
            segment: Segment::default(),
            segment_index: None,
            in_tags: vec![],
            out_label: None,
            out_type: None,
            out_tags,
            script_source: None,
            state: None,
        };
        assert_eq!(js.out_tags.len(), 2);
        assert!(js.out_label.is_none());
    }
}
