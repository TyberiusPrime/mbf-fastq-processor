use crate::config::deser::TagLabel;

/// A conditional tag with optional inversion
/// Serialized as `tag_name` or !`tag_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalTag {
    pub tag: TagLabel,
    pub invert: bool,
}

impl ConditionalTag {
    /* #[must_use]
    pub fn new(tag: String, invert: bool) -> Self {
        Self { tag, invert }
    } */

    #[must_use]
    pub fn from_tag_label(s: &TagLabel) -> Self {
        if let Some(tag) = s.0.strip_prefix('!') {
            ConditionalTag {
                tag: TagLabel(tag.to_string()),
                invert: true,
            }
        } else {
            ConditionalTag {
                tag: TagLabel(s.0.clone()),
                invert: false,
            }
        }
    }
}
