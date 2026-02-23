use schemars::JsonSchema;
use toml_pretty_deser::{prelude::*, suggest_alternatives};

use crate::config::PartialConfig;

// #[derive(Clone, Eq, PartialEq, JsonSchema)]
// #[tpd]
// #[derive(Debug)]
// pub struct Segment(pub String);
//
//
// impl Default for Segment {
//     fn default() -> Self {
//         Segment(":::first_and_only_segment".to_string())
//     }
// }
//
// #[derive(Clone, Eq, PartialEq, JsonSchema)]
// #[tpd]
// #[derive(Debug)]
// pub struct SegmentOrAll(pub String);
//
// impl Default for SegmentOrAll {
//     fn default() -> Self {
//         SegmentOrAll(":::first_and_only_segment".to_string())
//     }
// }
//
pub trait ValidateSegment {
    fn validate_segment(&mut self, config: &PartialConfig);
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct SegmentIndex(pub usize);

impl ValidateSegment for TomlValue<MustAdapt<String, SegmentIndex>> {
    fn validate_segment(&mut self, config: &PartialConfig) {
        let input_def = config
            .input
            .as_ref()
            .expect("validate_segment called before input definition was read");
        let segment_order = input_def.get_segment_order();
        let span = self.span.clone();
        if self.is_needs_further_validation()
            && let Some(must_adapt) = self.value.as_ref()
        {
            match must_adapt {
                MustAdapt::PreVerify(str_segment) => {
                    let segment_index = segment_order.iter().position(|x| x == str_segment);
                    *self = match segment_index {
                        Some(idx) => {
                            TomlValue::new_ok(MustAdapt::PostVerify(SegmentIndex(idx)), span)
                        }
                        None => TomlValue::new_validation_failed(
                            span,
                            "Segment not present in [input] section".to_string(),
                            Some(suggest_alternatives(str_segment, segment_order)),
                        ),
                    }
                }
                MustAdapt::PostVerify(_) => {
                    panic!("validate_segment called on an already validated segment")
                }
            }
        } else if self.is_missing() {
            if segment_order.len() == 1 {
                *self =
                    TomlValue::new_ok(MustAdapt::PostVerify(SegmentIndex(0)), self.span.clone());
            } else {
                let segment_names = segment_order.join(", ");
                *self = TomlValue::new_validation_failed(
                    self.span.clone(), //todo: is this on the right place (parent span?)
                    "Segment not specified but multiple segments available".to_string(),
                    Some(format!("Available segments: {segment_names}")),
                );
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy, JsonSchema)]
pub enum SegmentIndexOrAll {
    All,
    Indexed(usize),
}

impl ValidateSegment for TomlValue<MustAdapt<String, SegmentIndexOrAll>> {
    fn validate_segment(&mut self, config: &PartialConfig) {
        let input_def = config
            .input
            .as_ref()
            .expect("validate_segment called before input definition was read");
        let segment_order = input_def.get_segment_order();
        let span = self.span();
        if self.is_needs_further_validation()
            && let Some(must_adapt) = self.value.as_ref()
        {
            match must_adapt {
                MustAdapt::PreVerify(str_segment) => {
                    if str_segment == "all" || str_segment == "All" {
                        *self = TomlValue::new_ok(
                            MustAdapt::PostVerify(SegmentIndexOrAll::All),
                            self.span.clone(),
                        );
                    } else {
                        let segment_index = segment_order.iter().position(|x| x == str_segment);
                        *self = match segment_index {
                            Some(idx) => TomlValue::new_ok(
                                MustAdapt::PostVerify(SegmentIndexOrAll::Indexed(idx)),
                                span,
                            ),
                            None => TomlValue::new_validation_failed(
                                span,
                                "Segment not present in [input] section".to_string(),
                                Some(suggest_alternatives(str_segment, segment_order)),
                            ),
                        }
                    }
                }
                MustAdapt::PostVerify(_) => {
                    panic!("validate_segment called on an already validated segment")
                }
            }
        } else if self.is_missing() {
            if segment_order.len() == 1 {
                *self = TomlValue::new_ok(
                    MustAdapt::PostVerify(SegmentIndexOrAll::Indexed(0)),
                    self.span.clone(),
                );
            } else {
                let segment_names = segment_order.join(", ");
                *self = TomlValue::new_validation_failed(
                    self.span.clone(), //todo: is this on the right place (parent span?)
                    "Segment not specified but multiple segments available".to_string(),
                    Some(format!("Available segments: {segment_names}")),
                );
            }
        }
        //other errors passed on as is.
    }
}

// impl Segment {
//     /// validate and turn into an indexed segment
//     pub(crate) fn validate(&self, input_def: &crate::config::Input) -> Result<SegmentIndex> {
//         if self.0 == ":::first_and_only_segment" {
//             if input_def.segment_count() == 1 {
//                 return Ok(SegmentIndex(0));
//             } else {
//                 let segment_names = input_def.get_segment_order().join(", ");
//                 bail!(
//                     "Segment not specified but multiple segments available: [{segment_names}]. \
//                      Please specify which segment to use with 'segment = \"segment_name\"'",
//                 );
//             }
//         }
//         if self.0 == "all" || self.0 == "All" {
//             bail!(
//                 "'all' (or 'All') is not a valid segment in this position. Choose one of these: [{}]",
//                 input_def.get_segment_order().join(", ")
//             );
//         }
//         let name = &self.0;
//         let idx = input_def.index(name).with_context(|| {
//             let segment_names = input_def.get_segment_order().join(", ");
//             format!("Unknown segment: {name}. Available [{segment_names}]")
//         })?;
//         Ok(SegmentIndex(idx))
//     }
// }

// impl SegmentOrAll {
//     /// validate and turn into an indexed segment
//     pub(crate) fn validate(
//         &mut self,
//         input_def: &crate::config::Input,
//     ) -> Result<SegmentIndexOrAll> {
//         if self.0 == ":::first_and_only_segment" {
//             if input_def.segment_count() == 1 {
//                 return Ok(SegmentIndexOrAll::Indexed(0));
//             } else {
//                 let segment_names = input_def.get_segment_order().join(", ");
//                 bail!(
//                     "Segment not specified but multiple segments available: [{segment_names}]. Also 'all' is valid here. \
//                      Please specify which segment to use with 'segment = \"segment_name\"'",
//                 );
//             }
//         }
//         if self.0 == "all" || self.0 == "All" {
//             return Ok(SegmentIndexOrAll::All);
//         }
//         let name = &self.0;
//         let idx = input_def
//             .index(name)
//             .with_context(|| format!("Unknown segment: {name}"))?;
//         Ok(SegmentIndexOrAll::Indexed(idx))
//     }
// }

impl SegmentIndex {
    #[must_use]
    pub fn get_index(&self) -> usize {
        self.0
    }
}

impl TryInto<SegmentIndex> for SegmentIndexOrAll {
    type Error = ();

    fn try_into(self) -> std::prelude::v1::Result<SegmentIndex, Self::Error> {
        match self {
            SegmentIndexOrAll::Indexed(idx) => Ok(SegmentIndex(idx)),
            SegmentIndexOrAll::All => Err(()),
        }
    }
}

// #[derive(Clone, Eq, PartialEq, JsonSchema)]
// #[tpd]
// #[derive(Debug)]
// pub struct SegmentSequenceOrName(pub String);

/* impl Default for SegmentSequenceOrName {
    fn default() -> Self {
        SegmentSequenceOrName(":::first_and_only_segment".to_string())
    }
} */

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum SegmentOrNameIndex {
    Sequence(SegmentIndex),
    Name(SegmentIndex),
}

impl ValidateSegment for TomlValue<MustAdapt<String, SegmentOrNameIndex>> {
    fn validate_segment(&mut self, config: &PartialConfig) {
        let input_def = config
            .input
            .as_ref()
            .expect("validate_segment called before input definition was read");
        let segment_order = input_def.get_segment_order();
        let span = self.span.clone();
        if self.is_needs_further_validation()
            && let Some(must_adapt) = self.value.as_ref()
        {
            match must_adapt {
                MustAdapt::PreVerify(str_segment) => {
                    if let Some(query) = str_segment.strip_prefix("name:") {
                        let segment_index = segment_order.iter().position(|x| x == query);
                        *self = match segment_index {
                            Some(idx) => TomlValue::new_ok(
                                MustAdapt::PostVerify(SegmentOrNameIndex::Name(SegmentIndex(idx))),
                                span,
                            ),
                            None => TomlValue::new_validation_failed(
                                span,
                                "Segment not present in [input] section".to_string(),
                                Some(suggest_alternatives(str_segment, segment_order)),
                            ),
                        }
                    } else {
                        let segment_index = segment_order.iter().position(|x| x == str_segment);
                        *self = match segment_index {
                            Some(idx) => TomlValue::new_ok(
                                MustAdapt::PostVerify(SegmentOrNameIndex::Sequence(SegmentIndex(
                                    idx,
                                ))),
                                span,
                            ),
                            None => TomlValue::new_validation_failed(
                                span,
                                "Segment not present in [input] section".to_string(),
                                Some(suggest_alternatives(str_segment, segment_order)),
                            ),
                        }
                    }
                }
                MustAdapt::PostVerify(_) => {
                    panic!("validate_segment called on an already validated segment")
                }
            }
        }
        //no default for missing.
    }
}

// impl SegmentSequenceOrName {
//     /// validate and turn into an indexed segment (either sequence or name)
//     pub(crate) fn validate(
//         &mut self,
//         input_def: &crate::config::Input,
//     ) -> Result<SegmentOrNameIndex> {
//         /* if self.0 == ":::first_and_only_segment" {
//             if input_def.segment_count() == 1 {
//                 return Ok(SegmentOrNameIndex::Sequence(SegmentIndex(0)));
//             } else {
//                 let segment_names = input_def.get_segment_order().join(", ");
//                 bail!(
//                     "Source (segment/name) not specified but multiple segments available: [{segment_names}]. \
//                      Please specify which segment to use with 'source = \"segment_name\"' or 'source = \"name:segment_name\"'",
//                 );
//             }
//         } */
//         if self.0 == "all" || self.0 == "All" {
//             bail!(
//                 "'all' (or 'All') is not a valid segment in this position. Choose one of these: [{}]",
//                 input_def.get_segment_order().join(", ")
//             );
//         }
//
//         // Check for name: prefix
//         if let Some(segment_name) = self.0.strip_prefix("name:") {
//             let idx = input_def.index(segment_name).with_context(|| {
//                 let segment_names = input_def.get_segment_order().join(", ");
//                 format!("Unknown segment in 'name:{segment_name}'. Available [{segment_names}]")
//             })?;
//             Ok(SegmentOrNameIndex::Name(SegmentIndex(idx)))
//         } else {
//             // Regular segment reference (sequence)
//             let idx = input_def.index(&self.0).with_context(|| {
//                 let segment_names = input_def.get_segment_order().join(", ");
//                 format!("Unknown segment: {}. Available [{segment_names}]", self.0)
//             })?;
//             Ok(SegmentOrNameIndex::Sequence(SegmentIndex(idx)))
//         }
//     }
// }

impl SegmentOrNameIndex {
    #[must_use]
    pub fn get_segment_index(&self) -> SegmentIndex {
        match self {
            SegmentOrNameIndex::Sequence(idx) | SegmentOrNameIndex::Name(idx) => *idx,
        }
    }

    #[must_use]
    pub fn is_name(&self) -> bool {
        matches!(self, SegmentOrNameIndex::Name(_))
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedSourceNoAll {
    Segment(SegmentIndex),
    Tag(String),
    Name {
        segment_index: SegmentIndex,
        split_character: u8,
    },
}

impl ValidateSegment for TomlValue<MustAdapt<String, ResolvedSourceNoAll>> {
    fn validate_segment(&mut self, config: &PartialConfig) {
        let input_def = config
            .input
            .as_ref()
            .expect("validate_segment called before input definition was read");
        let input_options = input_def
            .options
            .as_ref()
            .expect("Options should have been set at this point");
        let segment_order = input_def.get_segment_order();
        if self.is_needs_further_validation()
            && let Some(must_adapt) = self.value.as_ref()
        {
            match must_adapt {
                MustAdapt::PreVerify(source) => {
                    let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
                        let trimmed = tag_name.trim();
                        if trimmed.is_empty() {
                            Err(ValidationFailure::new(
                                "Must not be empty",
                                Some("Please provide a name after 'tag:'."),
                            ))
                        } else {
                            Ok(ResolvedSourceNoAll::Tag(trimmed.to_string()))
                        }
                    } else if let Some(segment_name) = source.strip_prefix("name:") {
                        let trimmed = segment_name.trim();
                        if trimmed.is_empty() {
                            Err(ValidationFailure::new(
                                "Must not be empty",
                                Some("Please provide a segment name after 'name:'."),
                            ))
                        } else if let Some(segment_index) = input_def
                            .get_segment_order()
                            .iter()
                            .position(|x| x == trimmed)
                        {
                            Ok(ResolvedSourceNoAll::Name {
                                segment_index: SegmentIndex(segment_index),
                                split_character: *input_options
                                    .read_comment_character
                                    .as_ref()
                                    .expect("read_comment_character should have been set"),
                            })
                        } else {
                            Err(ValidationFailure::new(
                                "Segment not found".to_string(),
                                Some(format!(
                                    "Available segments: [{}]",
                                    segment_order.join(", ")
                                )),
                            ))
                        }
                    } else if let Some(segment_index) = input_def
                        .get_segment_order()
                        .iter()
                        .position(|x| x == source)
                    {
                        Ok(ResolvedSourceNoAll::Segment(SegmentIndex(segment_index)))
                    } else {
                        Err(ValidationFailure::new(
                            "Segment not found".to_string(),
                            Some(format!(
                                "Available segments: [{}]",
                                segment_order.join(", ")
                            )),
                        ))
                    };
                    match resolved {
                        Ok(resolved) => {
                            *self = TomlValue::new_ok(
                                MustAdapt::PostVerify(resolved),
                                self.span.clone(),
                            );
                        }
                        Err(validation_err) => {
                            self.state = TomlValueState::ValidationFailed {
                                message: validation_err.message,
                            };
                            self.help = validation_err.help;
                        }
                    }
                }
                MustAdapt::PostVerify(_) => {
                    panic!("validate_segment called on an already validated segment")
                }
            }
            //no default for missing.
        }
    }
}

impl ResolvedSourceNoAll {
    //that's the ones we're going to use
    #[must_use]
    pub fn get_tags(&self) -> Option<Vec<(String, &[crate::transformations::TagValueType])>> {
        match &self {
            ResolvedSourceNoAll::Tag(tag_name) => Some(vec![(
                tag_name.clone(),
                &[
                    crate::transformations::TagValueType::String,
                    crate::transformations::TagValueType::Location,
                ],
            )]),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResolvedSourceAll {
    Segment(SegmentIndexOrAll),
    Tag(String),
    Name {
        segment_index_or_all: SegmentIndexOrAll,
        split_character: u8,
    },
}
impl ValidateSegment for TomlValue<MustAdapt<String, ResolvedSourceAll>> {
    fn validate_segment(&mut self, config: &PartialConfig) {
        let input_def = config
            .input
            .as_ref()
            .expect("validate_segment called before input definition was read");
        let input_options = input_def
            .options
            .as_ref()
            .expect("Options should have been set at this point");
        let segment_order = input_def.get_segment_order();
        if self.is_needs_further_validation()
            && let Some(must_adapt) = self.value.as_mut()
        {
            match must_adapt {
                MustAdapt::PreVerify(source) => {
                    let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
                        let trimmed = tag_name.trim();
                        if trimmed.is_empty() {
                            Err(ValidationFailure::new(
                                "Must not be empty",
                                Some("Please provide a name after 'tag:'."),
                            ))
                        } else {
                            Ok(ResolvedSourceAll::Tag(trimmed.to_string()))
                        }
                    } else if let Some(segment_name) = source.strip_prefix("name:") {
                        let trimmed = segment_name.trim();
                        if trimmed.is_empty() {
                            Err(ValidationFailure::new(
                                "Must not be empty",
                                Some("Please provide a segment name after 'name:'."),
                            ))
                        } else if trimmed.eq_ignore_ascii_case("all") {
                            Ok(ResolvedSourceAll::Name {
                                segment_index_or_all: SegmentIndexOrAll::All,
                                split_character: *input_options
                                    .read_comment_character
                                    .as_ref()
                                    .expect("read_comment_character should have been set"),
                            })
                        } else if let Some(segment_index) = input_def
                            .get_segment_order()
                            .iter()
                            .position(|x| x == trimmed)
                        {
                            Ok(ResolvedSourceAll::Name {
                                segment_index_or_all: SegmentIndexOrAll::Indexed(segment_index),
                                split_character: *input_options
                                    .read_comment_character
                                    .as_ref()
                                    .expect("read_comment_character should have been set"),
                            })
                        } else {
                            Err(ValidationFailure::new(
                                "Segment not found".to_string(),
                                Some(format!(
                                    "Available segments: [{}]",
                                    segment_order.join(", ")
                                )),
                            ))
                        }
                    } else if source.eq_ignore_ascii_case("all") {
                        Ok(ResolvedSourceAll::Segment(SegmentIndexOrAll::All))
                    } else if let Some(segment_index) = input_def
                        .get_segment_order()
                        .iter()
                        .position(|x| x == source)
                    {
                        Ok(ResolvedSourceAll::Segment(SegmentIndexOrAll::Indexed(
                            segment_index,
                        )))
                    } else {
                        Err(ValidationFailure::new(
                            "Segment not found".to_string(),
                            Some(format!(
                                "Available segments: [{}]",
                                segment_order.join(", ")
                            )),
                        ))
                    };
                    match resolved {
                        Ok(resolved) => {
                            *self = TomlValue::new_ok(
                                MustAdapt::PostVerify(resolved),
                                self.span.clone(),
                            );
                        }
                        Err(validation_err) => {
                            self.state = TomlValueState::ValidationFailed {
                                message: validation_err.message,
                            };
                            self.help = validation_err.help;
                        }
                    }
                }
                MustAdapt::PostVerify(_) => {
                    panic!("validate_segment called on an already validated segment")
                }
            }
            //no default for missing.
        } else {
            if self.is_missing() {
                //if we have exactly one segment, and no tags,
                //we default to the one and only segment.
                //Todo: this is not fully implemented, we're not checking the tags,
                //since we're not yet buildng them in verify
                if let Some(input_def) = config.input.as_ref() {
                    let segment_count = input_def.get_segment_order().len();
                    if segment_count == 1 {
                        // && input_def.tag_at_this_step().is_empty() {
                        self.value = Some(MustAdapt::PostVerify(ResolvedSourceAll::Segment(
                            SegmentIndexOrAll::Indexed(0),
                        )));
                    }
                }
            }
            //still missing? error message
            if self.is_missing() {
                self.help = Some(format!(
                    "Please provide a source, that is a <segment name>, a <name:segment_name> or tag name. Use 'all' to refer to all <segment_name>s. Available segments: {}",
                    toml_pretty_deser::format_quoted_list(
                        &(config
                            .input
                            .as_ref()
                            .map(|input_def| input_def
                                .get_segment_order()
                                .iter()
                                .map(|x| x.as_str())
                                .collect())
                            .unwrap_or_else(|| vec![""]))
                    )
                ));
            }
        }
    }
}

impl ResolvedSourceAll {
    pub fn get_name(&self, segment_order: &[String]) -> String {
        match self {
            ResolvedSourceAll::Segment(SegmentIndexOrAll::Indexed(idx)) => {
                segment_order.get(*idx).cloned().unwrap_or_else(|| {
                    panic!(
                        "Segment index {idx} out of bounds for segment order: [{segment_order:?}]"
                    )
                })
            }
            ResolvedSourceAll::Segment(SegmentIndexOrAll::All) => "all".to_string(),
            ResolvedSourceAll::Tag(name) => format!("tag:{name}"),
            ResolvedSourceAll::Name {
                segment_index_or_all,
                ..
            } => format!(
                "name:{}",
                match segment_index_or_all {
                    SegmentIndexOrAll::Indexed(idx) => {
                        segment_order.get(*idx).cloned().unwrap_or_else(|| {
                        panic!("Segment index {idx} out of bounds for segment order: [{segment_order:?}]")
                    })
                    }
                    SegmentIndexOrAll::All => "all".to_string(),
                }
            ),
        }
    }

    // pub fn parse(
    //     source: &str,
    //     input_def: &config::Input,
    // ) -> Result<ResolvedSourceAll, anyhow::Error> {
    //     let source = source.trim();
    //     let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
    //         let trimmed = tag_name.trim();
    //         if trimmed.is_empty() {
    //             bail!("Source/target tag:<name> may not have an empty name.");
    //         }
    //         ResolvedSourceAll::Tag(trimmed.to_string())
    //     } else if let Some(segment_name) = source.strip_prefix("name:") {
    //         let trimmed = segment_name.trim();
    //         if trimmed.is_empty() {
    //             bail!("Source/target name:<segment> requires a segment name");
    //         }
    //         let mut segment = SegmentOrAll(trimmed.to_string());
    //         let segment_index_or_all = segment.validate(input_def)?;
    //         ResolvedSourceAll::Name {
    //             segment_index_or_all,
    //             split_character: input_def.options.read_comment_character,
    //         }
    //     } else {
    //         let mut segment = SegmentOrAll(source.to_string());
    //         ResolvedSourceAll::Segment(segment.validate(input_def)?)
    //     };
    //     Ok(resolved)
    // }

    //that's the ones we're going to use
    #[must_use]
    pub fn get_tags(&self) -> Option<Vec<(String, &[crate::transformations::TagValueType])>> {
        match &self {
            ResolvedSourceAll::Tag(tag_name) => Some(vec![(
                tag_name.clone(),
                &[
                    crate::transformations::TagValueType::String,
                    crate::transformations::TagValueType::Location,
                ],
            )]),
            _ => None,
        }
    }
}
