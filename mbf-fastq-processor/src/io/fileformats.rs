use schemars::JsonSchema;
use toml_pretty_deser::prelude::*;

#[derive(Clone, PartialEq, Eq, Copy, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub enum PhredEncoding {
    #[tpd(alias = "sanger")]
    #[tpd(alias = "illumina1.8")] //ilummina 1.8+ is sanger.
    Sanger, //33..=126, offset 33
    #[tpd(alias = "solexa")]
    Solexa, //59..=126, offset 64
    #[tpd(alias = "illumina1.3")]
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
