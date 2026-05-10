use crate::parse::frontmatter::field_as_string;
use crate::parse::DocEntry;

/// Check if a document entry's frontmatter matches the given type and topic filters.
/// Returns true if all provided filters match (AND logic).
/// Returns true if no filters are provided.
pub fn entry_matches_frontmatter(
    entry: &DocEntry,
    type_filter: Option<&str>,
    topic: Option<&str>,
) -> bool {
    if type_filter.is_none() && topic.is_none() {
        return true;
    }

    let fm = match &entry.frontmatter {
        Some(fm) => fm,
        None => return false,
    };

    if let Some(t) = type_filter {
        match field_as_string(fm, "type") {
            Some(val) => {
                if !val.eq_ignore_ascii_case(t) {
                    return false;
                }
            }
            None => return false,
        }
    }

    if let Some(t) = topic {
        match field_as_string(fm, "topic") {
            Some(val) => {
                if !val.to_lowercase().contains(&t.to_lowercase()) {
                    return false;
                }
            }
            None => return false,
        }
    }

    true
}
