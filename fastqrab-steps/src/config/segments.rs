use toml_pretty_deser::prelude::*;

use crate::config::PartialConfig;
use fastqrab_config::{
    TagLabel, offer_alternatives,
    segments::{
        ResolvedSourceAll, ResolvedSourceNoAll, SegmentIndex, SegmentIndexOrAll, SegmentOrNameIndex,
    },
};

pub trait ValidateSegment {
    fn validate_segment(&mut self, config: &PartialConfig);
}

impl ValidateSegment for TomlValue<MustAdapt<String, SegmentIndex>> {
    #[track_caller]
    fn validate_segment(&mut self, config: &PartialConfig) {
        if let Some(input_def) = config.input.as_ref() {
            let segment_order = input_def.get_segment_order();
            let span = self.span.clone();
            if self.is_needs_further_validation()
                && let Some(must_adapt) = self.value.as_ref()
            {
                match must_adapt {
                    MustAdapt::PreVerify(str_segment) => {
                        *self = if str_segment.eq_ignore_ascii_case("all") {
                            TomlValue::new_validation_failed(
                                span,
                                "'all' segments not valid in this position".to_string(),
                                Some(offer_alternatives("", segment_order)),
                            )
                        } else {
                            let segment_index = segment_order.iter().position(|x| x == str_segment);
                            match segment_index {
                                Some(idx) => TomlValue::new_ok(
                                    MustAdapt::PostVerify(SegmentIndex(idx)),
                                    span,
                                ),
                                None => TomlValue::new_validation_failed(
                                    span,
                                    "Segment not present in [input] section".to_string(),
                                    Some(offer_alternatives(str_segment, segment_order)),
                                ),
                            }
                        }
                    }
                    MustAdapt::PostVerify(_) => {
                        // cov:excl-start
                        panic!("validate_segment called on an already validated segment")
                        // cov:excl-stop
                    }
                }
            } else if self.is_missing() {
                if segment_order.len() == 1 {
                    *self = TomlValue::new_ok(
                        MustAdapt::PostVerify(SegmentIndex(0)),
                        self.span.clone(),
                    );
                } else {
                    let segment_names = segment_order.join(", ");
                    *self = TomlValue::new_validation_failed(
                        self.span.clone(),
                        "Segment not specified but multiple segments available".to_string(),
                        Some(format!("Available segments: {segment_names}")),
                    );
                }
            } // cov:excl-line
        } // cov:excl-line
    }
}

impl ValidateSegment for TomlValue<MustAdapt<String, SegmentIndexOrAll>> {
    #[track_caller]
    fn validate_segment(&mut self, config: &PartialConfig) {
        if let Some(input_def) = config.input.as_ref() {
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
                                    Some(offer_alternatives(str_segment, segment_order)),
                                ),
                            }
                        }
                    }
                    MustAdapt::PostVerify(_) => {
                        // cov:excl-start
                        panic!("validate_segment called on an already validated segment")
                        // cov:excl-stop
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
                        self.span.clone(),
                        "Segment not specified but multiple segments available".to_string(),
                        Some(format!("Available segments: {segment_names}")),
                    );
                }
            }
            //other errors passed on as is.
        }
    }
}

impl ValidateSegment for TomlValue<MustAdapt<String, SegmentOrNameIndex>> {
    #[track_caller]
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
                    *self = if str_segment.eq_ignore_ascii_case("all") {
                        TomlValue::new_validation_failed(
                            span,
                            "'all' segments not valid in this position".to_string(),
                            Some(offer_alternatives("", segment_order)),
                        )
                    } else if let Some(query) = str_segment.strip_prefix("name:") {
                        let segment_index = segment_order.iter().position(|x| x == query);
                        match segment_index {
                            Some(idx) => TomlValue::new_ok(
                                MustAdapt::PostVerify(SegmentOrNameIndex::Name(SegmentIndex(idx))),
                                span,
                            ),
                            None => TomlValue::new_validation_failed(
                                span,
                                "Segment not present in [input] section".to_string(),
                                Some(offer_alternatives(str_segment, segment_order)),
                            ),
                        }
                    } else {
                        let segment_index = segment_order.iter().position(|x| x == str_segment);
                        match segment_index {
                            Some(idx) => TomlValue::new_ok(
                                MustAdapt::PostVerify(SegmentOrNameIndex::Sequence(SegmentIndex(
                                    idx,
                                ))),
                                span,
                            ),
                            None => TomlValue::new_validation_failed(
                                span,
                                "Segment not present in [input] section".to_string(),
                                Some(offer_alternatives(str_segment, segment_order)),
                            ),
                        }
                    }
                }
                MustAdapt::PostVerify(_) => {
                    // cov:excl-start
                    panic!("validate_segment called on an already validated segment")
                    // cov:excl-stop
                }
            }
        }
        //no default for missing.
    }
}

impl ValidateSegment for TomlValue<MustAdapt<String, ResolvedSourceNoAll>> {
    #[track_caller]
    fn validate_segment(&mut self, config: &PartialConfig) {
        if let Some(input_def) = config.input.as_ref() {
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
                        let resolved = if source.eq_ignore_ascii_case("all") {
                            Err(ValidationFailure::new(
                                "'all' segments not valid in this position".to_string(),
                                Some(offer_alternatives("", segment_order)),
                            ))
                        } else if let Some(tag_name) = source.strip_prefix("tag:") {
                            let trimmed = tag_name.trim();
                            if trimmed.is_empty() {
                                Err(ValidationFailure::new(
                                    "Must not be empty",
                                    Some("Please provide a name after 'tag:'."),
                                ))
                            } else {
                                Ok(ResolvedSourceNoAll::Tag(TagLabel(trimmed.to_string())))
                            }
                        } else if let Some(segment_name) = source.strip_prefix("name:") {
                            let trimmed = segment_name.trim();
                            if trimmed.is_empty() {
                                if segment_order.len() == 1 {
                                    Ok(ResolvedSourceNoAll::Name {
                                        segment_index: SegmentIndex(0),
                                        split_character: *input_options
                                            .read_comment_character
                                            .as_ref()
                                            .expect("read_comment_character should have been set"),
                                    })
                                } else {
                                    Err(ValidationFailure::new(
                                        "Must not be empty",
                                        Some(
                                            "Please provide a segment name after 'name:', for there were multiple segments to choose from.",
                                        ),
                                    ))
                                }
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
                        // cov:excl-start
                        panic!("validate_segment called on an already validated segment")
                        // cov:excl-stop
                    }
                }
                //no default for missing.
            } // cov:excl-line
        }
    }
}

impl ValidateSegment for TomlValue<MustAdapt<String, ResolvedSourceAll>> {
    #[allow(clippy::too_many_lines)]
    #[track_caller]
    fn validate_segment(&mut self, config: &PartialConfig) {
        if let Some(input_def) = config.input.as_ref() {
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
                                Ok(ResolvedSourceAll::Tag(TagLabel(trimmed.to_string())))
                            }
                        } else if let Some(segment_name) = source.strip_prefix("name:") {
                            let trimmed = segment_name.trim();
                            if trimmed.is_empty() {
                                if segment_order.len() == 1 {
                                    Ok(ResolvedSourceAll::Name {
                                        segment_index_or_all: SegmentIndexOrAll::Indexed(0),
                                        split_character: *input_options
                                            .read_comment_character
                                            .as_ref()
                                            .expect("read_comment_character should have been set"),
                                    })
                                } else {
                                    Err(ValidationFailure::new(
                                        "Must not be empty",
                                        Some(
                                            "Please provide a segment name after 'name:', for there were multiple segments to choose from.",
                                        ),
                                    ))
                                }
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
                        // cov:excl-start
                        panic!("validate_segment called on an already validated segment")
                        // cov:excl-stop
                    }
                }
                //no default for missing.
            } else {
                // we can not reliably detect the 'no tags' and single segment case here -
                // tags are assembled after the transformations have been verified
                // and the segments validated.
                //
                // So we need the user to provide it.
                if self.is_missing() {
                    self.help = Some(format!(
                        "Could not automatically determine source. Please provide a source, that is a <segment name>, a <name:segment_name> or tag name. Use 'all' to refer to all <segment_name>s. Available segments: {}",
                        toml_pretty_deser::format_quoted_list(
                            &(config
                                .input
                                .as_ref()
                                .expect("can only be reached when input was ok")
                                .get_segment_order()
                                .iter()
                                .map(String::as_str)
                                .collect::<Vec<&str>>())
                        )
                    ));
                } // cov:excl-line
            }
        } // cov:excl-line
    }
}