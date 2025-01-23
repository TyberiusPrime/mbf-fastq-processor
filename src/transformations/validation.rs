use super::{apply_in_place_wrapped, u8_from_string, ConfigTransformTarget, Target};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformValidate {
    #[serde(deserialize_with = "u8_from_string")]
    pub allowed: Vec<u8>,
    pub target: Target,
}
pub fn transform_validate_seq(
    config: &mut ConfigTransformValidate,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_in_place_wrapped(
        config.target,
        |read| {
            assert!(
                !read.seq().iter().any(|x| !config.allowed.contains(x)),
                "Invalid base found in sequence: {:?} {:?}",
                std::str::from_utf8(read.name()),
                std::str::from_utf8(read.seq())
            );
        },
        &mut block,
    );

    (block, true)
}

pub fn transform_validate_phred(
    config: &mut ConfigTransformTarget,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_in_place_wrapped(
        config.target,
        |read| {
            assert!(
                !read.qual().iter().any(|x| *x < 33 || *x > 74),
                "Invalid phred quality found. Expected 33..=74 (!..J) : {:?} {:?}",
                std::str::from_utf8(read.name()),
                std::str::from_utf8(read.qual())
            );
        },
        &mut block,
    );

    (block, true)
}
