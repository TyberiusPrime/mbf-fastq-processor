use schemars::JsonSchema;

use crate::config::deser::{ErrorCollectorExt, FromToml};

#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq, Copy, JsonSchema)]
pub enum PhredEncoding {
    #[serde(alias = "sanger")]
    #[serde(alias = "illumina_1_8")] //ilummina 1.8+ is sanger.
    #[serde(alias = "Illumina_1_8")] //ilummina 1.8+ is sanger.
    #[serde(alias = "illumina1.8")] //ilummina 1.8+ is sanger.
    #[serde(alias = "Illumina1.8")] //ilummina 1.8+ is sanger.
    Sanger, //33..=126, offset 33
    #[serde(alias = "solexa")]
    Solexa, //59..=126, offset 64
    #[serde(alias = "illumina_1_3")]
    #[serde(alias = "Illumina_1_3")]
    #[serde(alias = "illumina1.3")]
    #[serde(alias = "Illumina1.3")]
    Illumina13, //64..=126, offset 64
}

impl PhredEncoding {
    #[must_use]
    pub fn limits(&self) -> (u8, u8) {
        match self {
            PhredEncoding::Sanger => (33, 126),
            PhredEncoding::Solexa => (59, 126),
            PhredEncoding::Illumina13 => (64, 126),
        }
    }
}

impl FromToml for PhredEncoding {
    fn from_toml(
        value: &toml_edit::Item,
        collector: &crate::config::deser::ErrorCollector,
    ) -> crate::transformations::prelude::TomlResult<Self>
    where
        Self: Sized,
    {
        if let toml_edit::Item::Value(toml_edit::Value::String(s)) = value {
            let sl = s.value().to_lowercase();
            match &sl[..] {
                "sanger" | "illumina_1_8" | "illumina1.8" => {
                    return Ok(PhredEncoding::Sanger);
                }
                "solexa" => {
                    return Ok(PhredEncoding::Solexa);
                }
                "illumina_1_3" | "illumina1.3" => {
                    return Ok(PhredEncoding::Illumina13);
                }
                _ => { //fall through
                }
            }
        }

        collector.add_item(
            value,
            "Invalid value",
            "Expected a string containing 'Sanger', 'Solexa', 'Illumina13'",
        )
    }
}
