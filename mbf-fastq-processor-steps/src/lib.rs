pub mod config;
pub mod demultiplex;
pub mod transformations;

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
    }
    result
}
