use crate::transformations::prelude::*;

use fasteval::{Compiler, Evaler, Parser, Slab};
use std::collections::{BTreeMap, BTreeSet};

use crate::{dna::TagValue, io};

struct CompiledExpression {
    slab: Slab,
    instruction: fasteval::Instruction,
    var_names: BTreeSet<String>,
}

#[derive(eserde::Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvalExpression {
    /// The tag label to store the result
    pub out_label: String,
    /// The arithmetic expression to evaluate
    /// Variables in the expression should match existing numeric tag names
    pub expression: String,
    pub result_type: ResultType,

    #[serde(default)]
    #[serde(skip)]
    compiled: Option<CompiledExpression>,

    #[serde(default)]
    #[serde(skip)]
    segment_names: Option<Vec<String>>,
}

impl std::fmt::Debug for EvalExpression {
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

#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ResultType {
    #[default]
    Numeric,
    Bool,
}

impl Step for EvalExpression {
    fn needs_serial(&self) -> bool {
        true //otherwise move_inited doesn't get called correctly.
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        let mut slab = Slab::new();
        let parser = Parser::new();
        let instruction = parser
            .parse(&self.expression, &mut slab.ps)
            .with_context(|| format!("EvalExpression: invalid expression '{}'", self.expression))?
            .from(&slab.ps)
            .compile(&slab.ps, &mut slab.cs);
        self.compiled = Some(CompiledExpression {
            var_names: instruction.var_names(&slab),
            slab,
            instruction,
        });
        self.segment_names = Some(
            input_def
                .get_segment_order()
                .iter()
                .map(|x| x.clone())
                .collect(),
        );

        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        let tag_type = match self.result_type {
            ResultType::Numeric => TagValueType::Numeric,
            ResultType::Bool => TagValueType::Bool,
        };
        Some((self.out_label.clone(), tag_type))
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        // Extract variable names and declare them as numeric tags
        // Since we support both numeric and bool tags in expressions,
        // we use TagValueType::Any for flexibility
        let var_names = &self.compiled.as_ref().unwrap().var_names;
        if var_names.is_empty() {
            None
        } else {
            let mut out = Vec::new();
            for name in var_names {
                if let Some(suffix) = name.strip_prefix("len_") {
                    if !self
                        .segment_names
                        .as_ref()
                        .unwrap()
                        .iter()
                        .any(|x| x == suffix)
                    {
                        out.push((
                            suffix.to_string(),
                            &[TagValueType::String, TagValueType::Location][..],
                        ));
                    }
                } else {
                    out.push((
                        name.to_string(),
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
        &mut self,
        mut block: io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(io::FastQBlocksCombined, bool)> {

        // Parse and compile the expression for better performance
        let eval = &self.compiled.as_ref().unwrap();
        let slab = &eval.slab;
        let compiled = &eval.instruction;
        let var_names = &eval.var_names;

        // Get all tag data for the variables we need
        let mut tag_data: Vec<(&str, &Vec<TagValue>)> = Vec::new();
        let mut virtual_tags: Vec<(&str, Vec<TagValue>)> = Vec::new();

        for var_name in var_names {
            if var_name.starts_with("len_") {
                let mut tag_values = Vec::new();
                let suffix = var_name.split_once('_').unwrap().1;
                if let Some(segment_index) = self
                    .segment_names
                    .as_ref()
                    .unwrap()
                    .iter()
                    .position(|x| x == suffix)
                {
                    for read in block.segments[segment_index].entries.iter() {
                        tag_values.push(TagValue::Numeric(read.seq.len() as f64));
                    }
                } else {
                    let str_tag_values = block.tags.get(suffix).expect(
                        "Named tag requested but not found. should have been caught earlier. Bug",
                    );
                    for tag_value in str_tag_values {
                        let len = match tag_value {
                            TagValue::String(s) => s.len() as f64,
                            TagValue::Location(locs) => locs.covered_len() as f64,
                            TagValue::Missing => 0.0,
                            _ => panic!(
                                "EvalExpression: 'len_' variable '{}' expects String or Location tag type, but found other type. This should have been caught earlier. Bug",
                                suffix
                            ),
                        };
                        tag_values.push(TagValue::Numeric(len));
                    }
                }
                virtual_tags.push((var_name.as_str(), tag_values));
            } else {
                if let Some(tag_values) = block.tags.get(var_name.as_str()) {
                    tag_data.push((var_name.as_str(), tag_values));
                } else {
                    panic!(
                        "EvalExpression: variable '{}' in expression '{}' does not match any available tag. This should have been caught earlier. Bug",
                        var_name, self.expression
                    );
                }
            }
        }
        for (var_name, tag_values) in &virtual_tags {
            tag_data.push((var_name, &tag_values));
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
                    TagValue::Location(_) => {
                        1.0 //any location is true
                    }
                    TagValue::String(_) => {
                        1.0 //any string is true
                    }
                    TagValue::Missing => {
                        0.0 //any not set locatio/string is false
                    }
                };

                vars.insert((*var_name).to_string(), numeric_value);
            }

            let result = match compiled.eval(&slab, &mut vars) {
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
        block
            .tags
            .insert(self.out_label.clone(), results);

        Ok((block, true))
    }
}
