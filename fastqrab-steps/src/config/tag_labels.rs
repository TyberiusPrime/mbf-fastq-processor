use fastqrab_config::{TagLabel, segments::SegmentIndexOrAll, validate_tag_name};
use indexmap::IndexMap;
use toml_pretty_deser::{
    MustAdapt, TomlValue, TomlValueState, ValidationFailure, suggest_alternatives,
};

use crate::config::TagMetadata;

pub trait ValidateTagLabel {
    fn validate_incoming_tag_label(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        segment_order: &[String],
    );
}

impl ValidateTagLabel for TomlValue<MustAdapt<String, TagLabel>> {
    /// validate a (virtual) tag based on segment names and co.
    /// must be called in get_used_tags, not in verify
    fn validate_incoming_tag_label(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        segment_order: &[String],
    ) {
        if self.is_needs_further_validation()
            && let Some(must_adapt) = self.value.as_mut()
        {
            let resolved = match must_adapt {
                MustAdapt::PreVerify(value) => {
                    if value == "read_no" {
                        Ok(TagLabel::ReadNo)
                    } else if let Some(segment_name) = value.strip_prefix("len_") {
                        if segment_name.eq_ignore_ascii_case("all") {
                            Ok(TagLabel::Length(SegmentIndexOrAll::All, value.to_string()))
                        } else if let Some(position) =
                            segment_order.iter().position(|x| x == segment_name)
                        {
                            Ok(TagLabel::Length(
                                SegmentIndexOrAll::Indexed(position),
                                value.to_string(),
                            ))
                        } else if tags_available.keys().any(|tag_label| match tag_label {
                            TagLabel::Normal(name) => name == segment_name,
                            _ => false,
                        }) {
                            Ok(TagLabel::TagLength(
                                segment_name.to_string(),
                                value.to_string(),
                            ))
                        } else {
                            let mut available: Vec<String> =
                                segment_order.iter().map(|x| format!("len_{x}")).collect();
                            available.push("len_all".to_string());
                            available.extend(tags_available.keys().filter_map(|k| {
                                if let TagLabel::Normal(name) = k {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            }));

                            Err(ValidationFailure::new(
                                "Unknown length tag label".to_string(),
                                Some(format!(
                                    "'{segment_name}' is neither a segment nor a tag name. Choose an existing name.\n{}",
                                    suggest_alternatives(segment_name, &available)
                                )),
                            ))
                        }
                    } else if let Some(incoming_tag_name) = value.strip_prefix("location_") {
                        if tags_available.keys().any(|tag_label| match tag_label {
                            TagLabel::Normal(name) => name == incoming_tag_name,
                            _ => false,
                        }) {
                            Ok(TagLabel::TagLocation {
                                source: incoming_tag_name.to_string(),
                                definition: value.to_string(),
                            })
                        } else {
                            let available: Vec<String> = tags_available
                                .keys()
                                .filter_map(|k| {
                                    if let TagLabel::Normal(name) = k {
                                        Some(name.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            Err(ValidationFailure::new(
                                "Unknown location tag label".to_string(),
                                Some(format!(
                                    "'{incoming_tag_name}' is not a tag name. Choose an existing name.\n{}",
                                    suggest_alternatives(incoming_tag_name, &available)
                                )),
                            ))
                        }
                    } else {
                        match validate_tag_name(value) {
                            Ok(()) => Ok(TagLabel::Normal(value.to_string())),
                            Err(e) => Err(ValidationFailure::new(
                                "Invalid label".to_string(),
                                Some(format!("{e}")),
                            )),
                        }
                    }
                }
                MustAdapt::PostVerify(_) => unreachable!("validate_tag_label called twice"),
            };

            match resolved {
                Ok(resolved) => {
                    *self = TomlValue::new_ok(MustAdapt::PostVerify(resolved), self.span.clone());
                }
                Err(validation_err) => {
                    self.state = TomlValueState::ValidationFailed {
                        message: validation_err.message,
                    };
                    self.help = validation_err.help;
                }
            }
        }
    }
}
