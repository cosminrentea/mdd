use crate::parse::Section;

/// Check if a section matches the given content predicates.
/// All provided predicates must match (AND logic).
pub fn section_matches(
    section: &Section,
    heading: Option<&str>,
    has_table: bool,
    has_code: Option<&str>,
) -> bool {
    if let Some(h) = heading {
        if !section.title.to_lowercase().contains(&h.to_lowercase()) {
            return false;
        }
    }

    if has_table && !section.features.has_table {
        return false;
    }

    if let Some(lang) = has_code {
        if !section.features.has_code_block {
            return false;
        }
        if !lang.is_empty()
            && !section
                .features
                .code_languages
                .iter()
                .any(|l| l.eq_ignore_ascii_case(lang))
        {
            return false;
        }
    }

    true
}
