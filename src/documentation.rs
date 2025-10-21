//We can't 'enumerate' our Transformations enum,
//so we opted to parse the template.toml instead

use std::borrow::Cow;
/// Get the template for any step or section in template.toml
#[must_use]
pub fn get_template(step: Option<&str>) -> Option<Cow<'static, str>> {
    let template = include_str!("template.toml");
    match step {
        None => Some(Cow::Borrowed(template)),
        Some(step) => {
            let query = format!("==== {} ====\n", step.to_lowercase());
            if let Some(result) = search_query(&query, template) {
                Some(result)
            } else {
                let query = format!("== {} ==\n", step.to_lowercase());
                search_query(&query, template)
            }
        }
    }
}

fn search_query(query: &str, template: &'static str) -> Option<Cow<'static, str>> {
    if let Some(start_idx) = template.to_lowercase().find(query) {
        let rest = &template[start_idx + query.len()..];
        if let Some(end_idx) = rest.find("# =") {
            let res = &rest[..end_idx].trim();
            if res.starts_with('#') {
                Some(Cow::Owned(remove_leading_comment(res)))
            } else {
                Some(Cow::Borrowed(res))
            }
        } else {
            Some(Cow::Borrowed(rest.trim()))
        }
    } else {
        None
    }
}

fn remove_leading_comment(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            if let Some(stripped) = line.strip_prefix("# ") {
                stripped
            } else if let Some(stripped) = line.strip_prefix("#") {
                stripped
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}
