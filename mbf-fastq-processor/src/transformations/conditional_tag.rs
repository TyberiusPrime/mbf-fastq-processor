/// A conditional tag with optional inversion
/// Serialized as `tag_name` or !`tag_name`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalTag {
    pub tag: String,
    pub invert: bool,
}

impl ConditionalTag {
    /* #[must_use]
    pub fn new(tag: String, invert: bool) -> Self {
        Self { tag, invert }
    } */

    #[must_use]
    pub fn from_string(s: String) -> Self {
        if let Some(tag) = s.strip_prefix('!') {
            ConditionalTag {
                tag: tag.to_string(),
                invert: true,
            }
        } else {
            ConditionalTag {
                tag: s,
                invert: false,
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for ConditionalTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ConditionalTag::from_string(s))
    }
}

impl serde::Serialize for ConditionalTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = if self.invert {
            format!("!{}", self.tag)
        } else {
            self.tag.clone()
        };
        serializer.serialize_str(&s)
    }
}
