use crate::transformations::prelude::*;

use fasteval::{Compiler, Evaler, Parser, Slab};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::Ordering,
};

use crate::{dna::TagValue, io};

#[derive(Debug)]
struct CompiledExpression {
    slab: Slab,
    instruction: fasteval::Instruction,
    var_names: BTreeSet<String>,
}

/// Evaluate an equation on tags

#[derive(JsonSchema)]
#[tpd]
pub struct EvalExpression {
    /// The tag label to store the result
    pub out_label: String,
    /// The arithmetic expression to evaluate
    /// Variables in the expression should match existing numeric tag names
    #[tpd(alias = "expr")]
    pub expression: String,

    #[tpd(alias = "output_type")]
    pub result_type: ResultType,

    #[tpd(skip)]
    #[schemars(skip)]
    compiled: CompiledExpression,

    #[tpd(skip)]
    #[schemars(skip)]
    segment_names: Vec<String>, //todo: why do we need this copy of the segment order?

    #[tpd(skip)]
    #[schemars(skip)]
    next_index: std::sync::atomic::AtomicU64, // for read_no
}

impl VerifyIn<PartialConfig> for PartialEvalExpression {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.out_label.verify(|v| {
            if v.trim().is_empty() {
                return Err(ValidationFailure::new(
                    "out_label cannot be empty",
                    Some("Provide a label to store the result under"),
                ));
            }
            Ok(())
        });
        self.expression.verify(|v| {
            if v.trim().is_empty() {
                return Err(ValidationFailure::new(
                    "expression cannot be empty",
                    Some("Provide an expression to evaluate"),
                ));
            }
            Ok(())
        });
        if let Some(expression) = self.expression.as_ref() {
            // Try parsing the expression to catch syntax errors early
            let mut slab = Slab::new();
            let parser = Parser::new();
            match parser.parse(expression, &mut slab.ps) {
                Err(e) => {
                    let help_message = format!("Inner error message {e}");
                    return Err(ValidationFailure::new(
                        "Syntax error".to_string(),
                        Some(help_message),
                    ));
                }
                Ok(parsed) => {
                    let instruction = parsed.from(&slab.ps).compile(&slab.ps, &mut slab.cs);
                    self.compiled = Some(CompiledExpression {
                        var_names: instruction.var_names(&slab),
                        slab,
                        instruction,
                    });
                }
            }
        }

        if let Some(input_def) = parent.input.as_ref() {
            self.segment_names = Some(
                input_def
                    .get_segment_order()
                    .iter()
                    .map(Clone::clone)
                    .collect(),
            );
        }
        self.next_index = Some(std::sync::atomic::AtomicU64::new(0));

        Ok(())
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for EvalExpression {
    #[mutants::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvalExpression")
            .field("label", &self.out_label)
            .field("expression", &self.expression)
            .field("result_type", &self.result_type)
            .finish()
    }
}

impl Clone for EvalExpression {
    fn clone(&self) -> Self {
        panic!("No cloning needs_serial steps")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, JsonSchema)]
#[tpd]
pub enum ResultType {
    #[default]
    Numeric,
    Bool,
}

impl Step for EvalExpression {
    fn validate_segments(&mut self, _input_def: &crate::config::Input) -> Result<()> {
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        let tag_type = match self.result_type {
            ResultType::Numeric => TagValueType::Numeric,
            ResultType::Bool => TagValueType::Bool,
        };
        Some((self.out_label.clone(), tag_type))
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        // Extract variable names and declare them as numeric tags
        // Since we support both numeric and bool tags in expressions,
        // we use TagValueType::Any for flexibility
        let var_names = &self.compiled.var_names;
        if var_names.is_empty() {
            None
        } else {
            let mut out = Vec::new();
            for name in var_names {
                if let Some(suffix) = name.strip_prefix("len_") {
                    if !self.segment_names.iter().any(|x| x == suffix) {
                        out.push((
                            suffix.to_string(),
                            &[TagValueType::String, TagValueType::Location][..],
                        ));
                    }
                } else if name == "read_no" {
                    // read_no is virtual, no tag needed
                } else {
                    out.push((
                        name.clone(),
                        &[
                            TagValueType::Bool,
                            TagValueType::Numeric,
                            TagValueType::String,
                            TagValueType::Location,
                        ][..],
                    ));
                }
            }
            Some(out)
        }
    }

    fn apply(
        &self,
        mut block: io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(io::FastQBlocksCombined, bool)> {
        // Parse and compile the expression for better performance
        let eval = &self.compiled;
        let slab = &eval.slab;
        let compiled = &eval.instruction;
        let var_names = &eval.var_names;

        // Get all tag data for the variables we need
        let mut tag_data: Vec<(&str, &Vec<TagValue>)> = Vec::new();
        let mut virtual_tags: Vec<(&str, Vec<TagValue>)> = Vec::new();

        let Some(first_segment) = block.segments.first() else {
            return Ok((block, true));
        };
        let read_count = first_segment.entries.len();
        let base_index = self
            .next_index
            .fetch_add(read_count as u64, Ordering::Relaxed);

        for var_name in var_names {
            if var_name.starts_with("len_") {
                let mut tag_values = Vec::new();
                let suffix = var_name
                    .split_once('_')
                    .expect("var_name must have underscore separator")
                    .1;
                if let Some(segment_index) = self.segment_names.iter().position(|x| x == suffix) {
                    #[allow(clippy::cast_precision_loss)]
                    for read in &block.segments[segment_index].entries {
                        tag_values.push(TagValue::Numeric(read.seq.len() as f64));
                    }
                } else {
                    let str_tag_values = block.tags.get(suffix).expect(
                        "Named tag requested but not found. should have been caught earlier. Bug",
                    );
                    #[allow(clippy::cast_precision_loss)]
                    for tag_value in str_tag_values {
                        let len = match tag_value {
                            TagValue::String(s) => s.len() as f64,
                            TagValue::Location(locs) => locs.covered_len() as f64,
                            TagValue::Missing => 0.0,
                            _ => panic!(
                                "EvalExpression: 'len_{suffix}' (a derived length variable) expects String or Location tag type, but found other type. This should have been caught earlier. Bug",
                            ),
                        };
                        tag_values.push(TagValue::Numeric(len));
                    }
                }
                virtual_tags.push((var_name.as_str(), tag_values));
            } else if var_name == "read_no" {
                let mut tag_values = Vec::new();
                for read_idx in 0..block.len() {
                    tag_values.push(TagValue::Numeric((base_index + read_idx as u64) as f64));
                }
                virtual_tags.push((var_name.as_str(), tag_values));
            } else if let Some(tag_values) = block.tags.get(var_name.as_str()) {
                tag_data.push((var_name.as_str(), tag_values));
            } else {
                panic!(
                    "EvalExpression: variable '{}' in expression '{}' does not match any available tag. This should have been caught earlier. Bug",
                    var_name, self.expression
                );
            }
        }
        for (var_name, tag_values) in &virtual_tags {
            tag_data.push((var_name, tag_values));
        }

        // Evaluate expression for each read
        let mut results: Vec<TagValue> = Vec::with_capacity(block.len());

        for read_idx in 0..block.len() {
            let mut vars = BTreeMap::new();

            // Populate vars with tag values for this read
            for (var_name, tag_values) in &tag_data {
                let tag_value = &tag_values[read_idx];

                // Convert TagValue to f64
                let numeric_value = match tag_value {
                    TagValue::Numeric(n) => *n,
                    TagValue::Bool(b) => {
                        if *b {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    TagValue::Location(_) |  //any location is true
                    TagValue::String(_) => { //any string is true
                        1.0
                    }
                    TagValue::Missing => {
                        0.0 //any not set locatio/string is false
                    }
                };

                vars.insert((*var_name).to_string(), numeric_value);
            }

            let result = match compiled.eval(slab, &mut vars) {
                Ok(val) => val,
                Err(e) => bail!(
                    "EvalExpression: error evaluating expression '{}' for read {}: {}",
                    self.expression,
                    read_idx,
                    e
                ),
            };

            // Convert result to TagValue based on result_type
            let tag_value = match self.result_type {
                ResultType::Numeric => TagValue::Numeric(result),
                ResultType::Bool => {
                    // Treat 0.0 as false, any other value as true
                    TagValue::Bool(result.abs() > f64::EPSILON)
                }
            };

            results.push(tag_value);
        }

        // Store the results
        block.tags.insert(self.out_label.clone(), results);

        Ok((block, true))
    }
}
