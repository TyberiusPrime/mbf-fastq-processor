use toml_pretty_deser::ValidationFailure;

pub mod config;
pub mod demultiplex;
pub mod input_files;
pub mod transformations;

#[must_use]
pub fn link_docs(step_name: &str) -> String {
    format!(
        "{}v{}/docs/redirects/{}",
        env!("CARGO_PKG_HOMEPAGE"),
        env!("CARGO_PKG_VERSION"),
        step_name
    )
}

pub fn join_nonempty<'a>(parts: impl IntoIterator<Item = &'a str>, separator: &str) -> String {
    let mut iter = parts.into_iter().filter(|part| !part.is_empty());
    let mut result = String::new();
    if let Some(first) = iter.next() {
        result.push_str(first);
        for part in iter {
            result.push_str(separator);
            result.push_str(part);
        }
    } // cov:excl-line
    result
}

#[must_use]
pub fn no_barcode_infix() -> &'static str {
    "nobarcode"
}

#[expect(
    clippy::ref_option,
    reason = "Must be this way to slot into tpd, which takes a &T for verify"
)]
fn verify_opt_path_component(suffix: &Option<String>) -> Result<(), ValidationFailure> {
    if let Some(path) = suffix.as_ref() {
        verify_path_component(path)
    } else {
        Ok(())
    }
}

#[expect(clippy::ptr_arg, reason = "TPD needs this signature")]
fn verify_path_component(suffix: &String) -> Result<(), ValidationFailure> {
    if suffix.contains('/') || suffix.contains('\\') || suffix.contains(':') {
        Err(ValidationFailure::new(
            "Invalid value",
            Some("Must not contain '/', '\\' or ':'."),
        ))
    } else if suffix.is_empty() {
        Err(ValidationFailure::new(
            "Invalid value",
            Some("Must not be empty."),
        ))
    } else {
        Ok(())
    }
}
