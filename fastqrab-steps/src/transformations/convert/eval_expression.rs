use fasteval::{Compiler, Evaler, Parser, Slab};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::transformations::prelude::*;
use fastqrab_dna::dna::TagValue;

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
    pub out_label: TagLabel,
    /// The arithmetic expression to evaluate
    /// Variables in the expression should match existing numeric tag names
    #[tpd(alias = "expr")]
    #[tpd(alias = "query")]
    pub expression: String,

    #[tpd(alias = "output_type")]
    #[tpd(alias = "out_type")]
    pub result_type: ResultType,

    #[tpd(skip)]
    #[schemars(skip)]
    compiled: CompiledExpression,

    #[tpd(skip)]
    #[schemars(skip)]
    var_name_to_tag: BTreeMap<String, TagLabel>,
}

impl VerifyIn<PartialConfig> for PartialEvalExpression {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
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
                    self.expression.state = TomlValueState::new_validation_failed("Syntax error");
                    self.expression.help = Some(help_message.clone());
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

        Ok(())
    }
}

// cov:excl-start
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
// cov:excl-stop

#[derive(Debug, Clone, Copy, PartialEq, Default, JsonSchema)]
#[tpd]
pub enum ResultType {
    #[default]
    Numeric,
    Bool,
}

impl TagUser for PartialTaggedVariant<Box<PartialEvalExpression>> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            // Extract variable names and declare them as numeric tags
            // Since we support both numeric and bool tags in expressions,
            // we use TagValueType::Any for flexibility
            //
            let mut var_name_to_tag = BTreeMap::new();
            let used_tags = if let Some(compiled) = inner.compiled.as_ref() {
                let var_names = &compiled.var_names;
                {
                    let mut used_tags = Vec::new();
                    let expr_span = inner.expression.span();
                    let toml_source = Rc::new(RefCell::new((
                        &mut inner.expression.state,
                        &mut inner.expression.help,
                    )));

                    for name in var_names {
                        let mut tv: TomlValue<MustAdapt<String, TagLabel>> = TomlValue {
                            state: TomlValueState::NeedsFurtherValidation,
                            span: expr_span.clone(),
                            value: Some(MustAdapt::PreVerify(name.clone())),
                            context: None,
                            help: None,
                        };
                        tv.validate_incoming_tag_label(tags_available, segment_order);
                        match tv.value.take().expect("just set") {
                            MustAdapt::PreVerify(_) => {
                                *toml_source.borrow_mut().0 = tv.state;
                                if let Some(help) = tv.help {
                                    if toml_source.borrow().1.is_some() {
                                        *toml_source.borrow_mut().1 = Some(format!(
                                            "{}\n{help}",
                                            toml_source.borrow().1.as_ref().unwrap()
                                        ));
                                    } else {
                                        *toml_source.borrow_mut().1 = Some(help);
                                    }
                                }
                            }
                            MustAdapt::PostVerify(tag_label) => {
                                let accepted_tag_types = &[
                                    TagValueType::Bool,
                                    TagValueType::Numeric((None, None)),
                                    TagValueType::String,
                                    TagValueType::Location,
                                ];
                                var_name_to_tag.insert(name.clone(), tag_label.clone());
                                used_tags.push(Some(UsedTag {
                                    name: tag_label,
                                    accepted_tag_types,
                                    toml_source: toml_source.clone(),
                                    further_help: None,
                                }));
                            }
                        }
                    }

                    used_tags
                }
            } else {
                Default::default()
            };

            inner.var_name_to_tag = Some(var_name_to_tag);
            Some(TagUsageInfo {
                used_tags,
                declared_tag: inner.out_label.to_declared_tag(
                    match inner.result_type.as_ref().unwrap_or(&ResultType::Numeric)// user forgot result_type, or mistype 
                    {
                        ResultType::Numeric => TagValueType::Numeric((None, None)),
                        ResultType::Bool => TagValueType::Bool,
                    },
                ),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Box<EvalExpression> {
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Parse and compile the expression for better performance
        let eval = &self.compiled;
        let slab = &eval.slab;
        let compiled = &eval.instruction;
        let var_names = &eval.var_names;

        // Get all tag data for the variables we need
        let mut tag_data: Vec<(&str, &Vec<TagValue>)> = Vec::new();

        for var_name in var_names {
            let tag = self.var_name_to_tag.get(var_name).expect("variable name should have a corresponding tag label - should have been set in get_tag_usage");
            if let Some(tag_values) = block.tags.get(tag) {
                tag_data.push((var_name.as_str(), tag_values));
            } else {
                // cov:excl-start
                panic!(
                    "EvalExpression: variable '{}' (tag: {:?}) in expression '{}' does not match any available tag. This should have been caught earlier. Bug. Available tags:{:?}",
                    var_name,
                    tag,
                    self.expression,
                    block.tags.keys()
                );
                // cov:excl-stop
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
                // cov:excl-start
                // when would this occur? We have already checked for
                // undefined variables.
                Err(e) => bail!(
                    "EvalExpression: error evaluating expression '{}' for read {}: {}",
                    self.expression,
                    read_idx,
                    e
                ),
                // cov:excl-stop
            };

            // Convert result to TagValue based on result_type
            let tag_value = match self.result_type {
                ResultType::Numeric => TagValue::Numeric(result),
                ResultType::Bool => {
                    // Treat 0.0 as false, any other value as true
                    TagValue::Bool(result.abs() >= f64::EPSILON)
                }
            };

            results.push(tag_value);
        }

        // Store the results
        block.tags.insert(self.out_label.clone(), results);

        Ok((block, true))
    }
}
