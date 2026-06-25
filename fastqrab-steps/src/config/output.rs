use schemars::JsonSchema;
use std::num::NonZero;
use toml_pretty_deser::prelude::*;

#[must_use]
pub fn default_ix_separator() -> String {
    "_".to_string()
}

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Output {
    pub prefix: String,

    #[schemars(default)]
    pub ix_separator: String,

    #[schemars(with = "Option<usize>")]
    pub compression_threads: NonZero<usize>,
}

impl VerifyIn<super::PartialConfig> for PartialOutput {
    fn verify(
        &mut self,
        _parent: &super::PartialConfig,
        _options: &VerifyOptions,
    ) -> Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.ix_separator.verify(|ix_separator| {
            if ix_separator.contains('/')
                || ix_separator.contains('\\')
                || ix_separator.contains(':')
            {
                Err(ValidationFailure::new(
                    "Invalid value",
                    Some("Must not contain '/', '\\' or ':'"),
                ))
            } else if ix_separator.is_empty() {
                Err(ValidationFailure::new(
                    "Invalid value",
                    Some("Must not be empty."),
                ))
            } else {
                Ok(())
            }
        });
        self.prefix.verify(|prefix| {
            if prefix.contains("/../")
                || prefix.contains("\\..\\")
                || prefix.contains(':')
                || prefix.starts_with('/')
                || prefix.starts_with('\\')
                || prefix.starts_with("../")
                || prefix.starts_with("..\\")
            {
                Err(ValidationFailure::new(
                    "Invalid value",
                    Some(
                        "Must not contain '/../', '\\..\\' or ':', nor be an absolute path. \
                        fastqrab only outputs below the current directory.",
                    ),
                ))
            } else if prefix.is_empty() {
                Err(ValidationFailure::new(
                    "Invalid value",
                    Some("Must not be empty."),
                ))
            } else {
                Ok(())
            }
        });

        self.ix_separator.or_with(default_ix_separator);
        self.compression_threads
            .or_with(|| NonZero::<usize>::MIN);

        Ok(())
    }
}
