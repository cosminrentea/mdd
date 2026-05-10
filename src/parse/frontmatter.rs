// YAML frontmatter parsing with raw text preservation for round-trip edits.
//
// Strategy: use serde_yaml to parse the YAML text into a BTreeMap for queries,
// but also keep the raw string so `mdd fm --set key=val` can do line-level
// string surgery without reformatting the entire YAML block.

use std::collections::BTreeMap;

use super::Frontmatter;

/// Parse raw YAML text (the content between `---` delimiters, not including them)
/// into a Frontmatter struct.
///
/// Returns None if the YAML is empty or cannot be parsed as a mapping.
///
/// # Rust concept: Option vs Result
/// We return Option here (not Result) because malformed frontmatter in a single
/// file shouldn't abort parsing of the entire document. The caller decides
/// whether to warn (default) or fail (--strict mode).
pub fn parse_frontmatter(raw_yaml: &str, offset: usize) -> Option<Frontmatter> {
    // Trim the raw YAML -- some files have trailing whitespace after the last field.
    let trimmed = raw_yaml.trim();
    if trimmed.is_empty() {
        return None;
    }

    // serde_yaml::from_str can parse into various types. We want a BTreeMap
    // because it preserves key ordering (alphabetical) for deterministic output.
    //
    // Rust concept: turbofish syntax `::<Type>` tells the compiler what type
    // we want serde_yaml to deserialize into.
    let fields: BTreeMap<String, serde_yaml::Value> = serde_yaml::from_str(trimmed).ok()?;

    Some(Frontmatter {
        fields,
        offset,
        raw: raw_yaml.to_owned(),
    })
}

/// Extract the value of a specific field from frontmatter as a string.
///
/// Handles common YAML value types: strings, numbers, bools.
/// Returns None if the field doesn't exist or is a complex type (sequence/mapping).
pub fn field_as_string(fm: &Frontmatter, key: &str) -> Option<String> {
    let value = fm.fields.get(key)?;
    match value {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        serde_yaml::Value::Null => Some("null".to_owned()),
        // Sequences and mappings aren't representable as a simple string
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_frontmatter() {
        let yaml = "type: reference\ntopic: testing\nweight: 0.8\n";
        let fm = parse_frontmatter(yaml, 1).expect("should parse");

        assert_eq!(fm.offset, 1);
        assert_eq!(fm.raw, yaml);
        assert_eq!(field_as_string(&fm, "type"), Some("reference".to_owned()));
        assert_eq!(field_as_string(&fm, "topic"), Some("testing".to_owned()));
        assert_eq!(field_as_string(&fm, "weight"), Some("0.8".to_owned()));
    }

    #[test]
    fn returns_none_for_empty() {
        assert!(parse_frontmatter("", 1).is_none());
        assert!(parse_frontmatter("   \n  ", 1).is_none());
    }

    #[test]
    fn returns_none_for_invalid_yaml() {
        // A bare string isn't a mapping, so from_str::<BTreeMap> fails.
        assert!(parse_frontmatter("just a string", 1).is_none());
    }

    #[test]
    fn handles_multiline_values() {
        let yaml = "type: note\ndescription: |\n  This is a\n  multi-line value\n";
        let fm = parse_frontmatter(yaml, 5).expect("should parse");
        assert_eq!(fm.offset, 5);
        assert_eq!(field_as_string(&fm, "type"), Some("note".to_owned()));
        // Multi-line strings are still strings in serde_yaml
        let desc = field_as_string(&fm, "description").unwrap();
        assert!(desc.contains("multi-line"));
    }

    #[test]
    fn field_as_string_handles_types() {
        let yaml = "count: 42\nenabled: true\nnull_val: null\n";
        let fm = parse_frontmatter(yaml, 1).unwrap();
        assert_eq!(field_as_string(&fm, "count"), Some("42".to_owned()));
        assert_eq!(field_as_string(&fm, "enabled"), Some("true".to_owned()));
        assert_eq!(field_as_string(&fm, "null_val"), Some("null".to_owned()));
        assert_eq!(field_as_string(&fm, "missing"), None);
    }
}
