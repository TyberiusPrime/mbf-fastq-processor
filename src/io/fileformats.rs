#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
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
