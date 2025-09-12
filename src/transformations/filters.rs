mod by_bool_tag;
mod by_numeric_tag;
mod by_tag;
mod empty;
mod head;
mod sample;
mod skip;

// Re-export all public structs
pub use by_bool_tag::ByBoolTag;
pub use by_numeric_tag::ByNumericTag;
pub use by_tag::ByTag;
pub use empty::Empty;
pub use head::Head;
pub use sample::Sample;
pub use skip::Skip;

use super::{Step, TagValueType, Transformation};
use anyhow::Result;

pub fn validate_tag_set_and_type(
    all_transforms: &[Transformation],
    this_transforms_index: usize,
    label: &str,
    supposed_tag_type: TagValueType,
) -> Result<()> {
    // Check that the required tag is declared as Numeric by an upstream step
    let mut found_tag_declaration = false;
    for (i, transform) in all_transforms.iter().enumerate() {
        if i >= this_transforms_index {
            break; // Only check upstream steps
        }
        if let Some((tag_name, tag_type)) = transform.declares_tag_type() {
            if tag_name == label {
                found_tag_declaration = true;
                if tag_type != supposed_tag_type {
                    return Err(anyhow::anyhow!(
                            "Step expects tag type {supposed_tag_type:?} tag for '{label}', but earlier step declares {tag_type:?} tag",
                        ));
                }
                break;
            }
        }
    }

    if !found_tag_declaration {
        return Err(anyhow::anyhow!(
            "Step expects {supposed_tag_type:?} tag named '{}', but no earlier step declares this tag",
            label
        ));
    }
    Ok(())
}
