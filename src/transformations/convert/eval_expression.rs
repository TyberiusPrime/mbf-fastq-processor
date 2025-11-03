use crate::transformations::prelude::*;

use fasteval::{Compiler, Evaler};
use std::collections::BTreeMap;

use crate::{dna::TagValue, io};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct EvalExpression {
    /// The tag label to store the result
    pub label: String,
    /// The arithmetic expression to evaluate
    /// Variables in the expression should match existing numeric tag names
    pub expression: String,
    /// Optional: specify the result type (defaults to Numeric)
    #[serde(default)]
    pub result_type: ResultType,
}

#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResultType {
    #[default]
    Numeric,
    Bool,
}

impl Step for EvalExpression {
    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        let tag_type = match self.result_type {
            ResultType::Numeric => TagValueType::Numeric,
            ResultType::Bool => TagValueType::Bool,
        };
        Some((self.label.clone(), tag_type))
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        // Extract variable names and declare them as numeric tags
        // Since we support both numeric and bool tags in expressions,
        // we use TagValueType::Any for flexibility
        let var_names = extract_variable_names(&self.expression);
        if var_names.is_empty() {
            None
        } else {
            Some(
                var_names
                    .into_iter()
                    .map(|name| {
                        (
                            name.to_string(),
                            &[TagValueType::Bool, TagValueType::Numeric][..],
                        )
                    })
                    .collect(),
            )
        }
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        // Try to parse the expression to catch syntax errors early
        let parser = fasteval::Parser::new();
        let mut slab = fasteval::Slab::new();
        match parser.parse(&self.expression, &mut slab.ps) {
            Ok(_) => Ok(()),
            Err(e) => bail!(
                "EvalExpression: invalid expression '{}': {}",
                self.expression,
                e
            ),
        }
    }

    fn apply(
        &mut self,
        mut block: io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(io::FastQBlocksCombined, bool)> {
        if block.tags.is_none() {
            bail!(
                "EvalExpression expects tags to be available for expression '{}'",
                self.expression
            );
        }

        // Parse and compile the expression for better performance
        let parser = fasteval::Parser::new();
        let mut slab = fasteval::Slab::new();
        let compiled = parser
            .parse(&self.expression, &mut slab.ps)?
            .from(&slab.ps)
            .compile(&slab.ps, &mut slab.cs);

        // Extract variable names from the expression
        let var_names = extract_variable_names(&self.expression);

        // Get all tag data for the variables we need
        let tags = block.tags.as_ref().unwrap();
        let mut tag_data: Vec<(&str, &Vec<TagValue>)> = Vec::new();

        for var_name in &var_names {
            if let Some(tag_values) = tags.get(var_name.as_str()) {
                tag_data.push((var_name.as_str(), tag_values));
            } else {
                bail!(
                    "EvalExpression: variable '{}' in expression '{}' does not match any available tag",
                    var_name,
                    self.expression
                );
            }
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
                        bail!(
                            "EvalExpression: tag '{}' is a location tag, which cannot be used in arithmetic expressions",
                            var_name
                        );
                    }
                    TagValue::String(_) => {
                        bail!(
                            "EvalExpression: tag '{}' is a string tag, which cannot be used in arithmetic expressions",
                            var_name
                        );
                    }
                    TagValue::Missing => {
                        bail!(
                            "EvalExpression: tag '{}' is missing for read {}",
                            var_name,
                            read_idx
                        );
                    }
                };

                vars.insert((*var_name).to_string(), numeric_value);
            }

            // Evaluate the expression
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
            .as_mut()
            .unwrap()
            .insert(self.label.clone(), results);

        Ok((block, true))
    }
}

/// Extract variable names from an expression string
/// Identifies all identifiers that are not built-in functions or constants
fn extract_variable_names(expression: &str) -> Vec<String> {
    let mut var_names = Vec::new();
    let mut current_word = String::new();
    let chars: Vec<char> = expression.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch.is_alphanumeric() || ch == '_' {
            current_word.push(ch);
        } else {
            if !current_word.is_empty() && !current_word.chars().all(|c| c.is_numeric()) {
                // Check if the next two characters are "::" which indicates namespace qualifier
                let is_namespace = i + 1 < chars.len() && chars[i] == ':' && chars[i + 1] == ':';

                // Only add as variable if it's not a namespace, builtin, or already in the list
                if !is_namespace
                    && !is_builtin_identifier(&current_word)
                    && !var_names.contains(&current_word)
                {
                    var_names.push(current_word.clone());
                }
            }
            current_word.clear();
        }
        i += 1;
    }

    // Handle the last word
    if !current_word.is_empty() && !current_word.chars().all(|c| c.is_numeric()) {
        if !is_builtin_identifier(&current_word) && !var_names.contains(&current_word) {
            var_names.push(current_word);
        }
    }

    var_names
}

/// Check if an identifier is a built-in constant or function
/// fasteval has built-in functions in the math:: namespace and some constants
fn is_builtin_identifier(id: &str) -> bool {
    matches!(
        id,
        "true"
            | "false"
            | "pi"
            | "e"
            | "math"
            | "min"
            | "max"
            | "floor"
            | "ceil"
            | "round"
            | "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "sinh"
            | "cosh"
            | "tanh"
            | "asinh"
            | "acosh"
            | "atanh"
            | "sqrt"
            | "exp"
            | "ln"
            | "log"
            | "log2"
            | "log10"
            | "abs"
            | "sign"
            | "int"
    )
}
