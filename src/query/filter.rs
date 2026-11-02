use crate::parse::frontmatter::field_as_string;
use crate::parse::{DocEntry, Section};

/// The filters shared by `find` and `agg`, built once per run.
///
/// Every provided predicate must match (AND logic). `topic` and `heading` are
/// lowercased once at construction (Unicode rules); `type` and the code
/// language stay raw and are compared with `eq_ignore_ascii_case`.
pub struct Filter {
    doc_type: Option<String>,
    topic: Option<String>,
    heading: Option<String>,
    has_table: bool,
    has_code: Option<String>,
}

impl Filter {
    pub fn new(
        type_filter: Option<&str>,
        topic: Option<&str>,
        heading: Option<&str>,
        has_table: bool,
        has_code: Option<&str>,
    ) -> Self {
        Self {
            doc_type: type_filter.map(str::to_owned),
            topic: topic.map(str::to_lowercase),
            heading: heading.map(str::to_lowercase),
            has_table,
            has_code: has_code.map(str::to_owned),
        }
    }

    /// Check a document entry's frontmatter against the type and topic filters.
    /// With neither filter set every entry matches, even one without frontmatter.
    pub fn matches_entry(&self, entry: &DocEntry) -> bool {
        if self.doc_type.is_none() && self.topic.is_none() {
            return true;
        }

        let Some(fm) = &entry.frontmatter else {
            return false;
        };

        if let Some(t) = &self.doc_type {
            match field_as_string(fm, "type") {
                Some(val) => {
                    if !val.eq_ignore_ascii_case(t) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        if let Some(t) = &self.topic {
            match field_as_string(fm, "topic") {
                Some(val) => {
                    if !val.to_lowercase().contains(t.as_str()) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    /// Check a section against the heading, table and code-language filters.
    pub fn matches_section(&self, section: &Section) -> bool {
        if let Some(h) = &self.heading
            && !section.title.to_lowercase().contains(h.as_str())
        {
            return false;
        }

        if self.has_table && !section.features.has_table {
            return false;
        }

        if let Some(lang) = &self.has_code {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::markdown::parse_content;

    const PLAIN: &str = "# Title\n\nbody\n";
    const FM: &str = "---\ntype: reference\ntopic: unit testing\n---\n\n# Title\n\nbody\n";
    const FM_UNICODE: &str = "---\ntype: étude\ntopic: étude\n---\n\n# Title\n\nbody\n";
    const FM_UPPER: &str = "---\ntype: Reference\ntopic: Unit Testing\n---\n\n# Title\n\nbody\n";
    const FM_UNICODE_UPPER: &str = "---\ntype: ÉTUDE\ntopic: ÉTUDE\n---\n\n# Title\n\nbody\n";
    const FM_NO_TYPE: &str = "---\ntopic: unit testing\n---\n\n# Title\n\nbody\n";
    const FM_NO_TOPIC: &str = "---\ntype: reference\n---\n\n# Title\n\nbody\n";
    const HEADING: &str = "# Project Setup Guide\n\nbody\n";
    const HEADING_UNICODE: &str = "# École Primaire\n\nbody\n";
    const TABLE: &str = "# Title\n\n| A | B |\n|---|---|\n| 1 | 2 |\n";
    const CODE_RUST: &str = "# Title\n\n```rust\nfn main() {}\n```\n";
    const CODE_TWO: &str = "# Title\n\n```rust\nfn main() {}\n```\n\n```python\npass\n```\n";
    const CODE_NO_LANG: &str = "# Title\n\n```\nplain\n```\n";

    /// One filter-semantics case: a document, the filters, and whether the
    /// first entry and the first section are expected to match.
    struct Case {
        name: &'static str,
        doc: &'static str,
        type_filter: Option<&'static str>,
        topic: Option<&'static str>,
        heading: Option<&'static str>,
        has_table: bool,
        has_code: Option<&'static str>,
        entry: bool,
        section: bool,
    }

    const ANY: Case = Case {
        name: "",
        doc: PLAIN,
        type_filter: None,
        topic: None,
        heading: None,
        has_table: false,
        has_code: None,
        entry: true,
        section: true,
    };

    /// The only place the tests touch the matching API: (entry matches, first
    /// section matches) for the first entry of the parsed document.
    fn evaluate(c: &Case) -> (bool, bool) {
        let md = parse_content(c.doc);
        let entry = &md.entries[0];
        let filter = Filter::new(c.type_filter, c.topic, c.heading, c.has_table, c.has_code);
        (
            filter.matches_entry(entry),
            filter.matches_section(&entry.sections[0]),
        )
    }

    fn check(cases: &[Case]) {
        for c in cases {
            assert_eq!(evaluate(c), (c.entry, c.section), "case: {}", c.name);
        }
    }

    #[test]
    fn no_filters_match_everything() {
        check(&[
            Case {
                name: "no filters, no frontmatter",
                ..ANY
            },
            Case {
                name: "no filters, frontmatter",
                doc: FM,
                ..ANY
            },
        ]);
    }

    #[test]
    fn type_filter() {
        check(&[
            Case {
                name: "type on entry without frontmatter",
                type_filter: Some("reference"),
                entry: false,
                ..ANY
            },
            Case {
                name: "type, same case",
                doc: FM,
                type_filter: Some("reference"),
                ..ANY
            },
            Case {
                name: "type, filter capitalised",
                doc: FM,
                type_filter: Some("Reference"),
                ..ANY
            },
            Case {
                name: "type, different value",
                doc: FM,
                type_filter: Some("guide"),
                entry: false,
                ..ANY
            },
            Case {
                name: "type, substring is not a match",
                doc: FM,
                type_filter: Some("refer"),
                entry: false,
                ..ANY
            },
            Case {
                name: "type, value capitalised, filter lowercase",
                doc: FM_UPPER,
                type_filter: Some("reference"),
                ..ANY
            },
            Case {
                name: "type, non-ASCII uppercase value, lowercase filter is ASCII-only",
                doc: FM_UNICODE_UPPER,
                type_filter: Some("étude"),
                entry: false,
                ..ANY
            },
            Case {
                name: "type, non-ASCII uppercase value, equal filter",
                doc: FM_UNICODE_UPPER,
                type_filter: Some("ÉTUDE"),
                ..ANY
            },
            Case {
                name: "type, field missing",
                doc: FM_NO_TYPE,
                type_filter: Some("reference"),
                entry: false,
                ..ANY
            },
            Case {
                name: "type, non-ASCII equal",
                doc: FM_UNICODE,
                type_filter: Some("étude"),
                ..ANY
            },
            Case {
                name: "type, non-ASCII uppercase filter is ASCII-only",
                doc: FM_UNICODE,
                type_filter: Some("ÉTUDE"),
                entry: false,
                ..ANY
            },
            Case {
                name: "type, non-ASCII initial uppercase is ASCII-only",
                doc: FM_UNICODE,
                type_filter: Some("Étude"),
                entry: false,
                ..ANY
            },
        ]);
    }

    #[test]
    fn topic_filter() {
        check(&[
            Case {
                name: "topic on entry without frontmatter",
                topic: Some("testing"),
                entry: false,
                ..ANY
            },
            Case {
                name: "topic substring, filter capitalised",
                doc: FM,
                topic: Some("Test"),
                ..ANY
            },
            Case {
                name: "topic substring, filter uppercase",
                doc: FM,
                topic: Some("UNIT"),
                ..ANY
            },
            Case {
                name: "topic, value capitalised, filter lowercase",
                doc: FM_UPPER,
                topic: Some("testing"),
                ..ANY
            },
            Case {
                name: "topic, non-ASCII uppercase value, lowercase filter folds",
                doc: FM_UNICODE_UPPER,
                topic: Some("étude"),
                ..ANY
            },
            Case {
                name: "topic, no substring",
                doc: FM,
                topic: Some("integration"),
                entry: false,
                ..ANY
            },
            Case {
                name: "topic, field missing",
                doc: FM_NO_TOPIC,
                topic: Some("testing"),
                entry: false,
                ..ANY
            },
            Case {
                name: "topic, non-ASCII uppercase filter folds",
                doc: FM_UNICODE,
                topic: Some("ÉTUDE"),
                ..ANY
            },
            Case {
                name: "topic, non-ASCII lowercase filter",
                doc: FM_UNICODE,
                topic: Some("étude"),
                ..ANY
            },
        ]);
    }

    #[test]
    fn type_and_topic_combine_with_and() {
        check(&[
            Case {
                name: "both match",
                doc: FM,
                type_filter: Some("reference"),
                topic: Some("testing"),
                ..ANY
            },
            Case {
                name: "type matches, topic does not",
                doc: FM,
                type_filter: Some("reference"),
                topic: Some("integration"),
                entry: false,
                ..ANY
            },
            Case {
                name: "topic matches, type does not",
                doc: FM,
                type_filter: Some("guide"),
                topic: Some("testing"),
                entry: false,
                ..ANY
            },
        ]);
    }

    #[test]
    fn heading_filter() {
        check(&[
            Case {
                name: "heading substring",
                doc: HEADING,
                heading: Some("Setup"),
                ..ANY
            },
            Case {
                name: "heading, filter uppercase",
                doc: HEADING,
                heading: Some("SETUP"),
                ..ANY
            },
            Case {
                name: "heading, filter lowercase",
                doc: HEADING,
                heading: Some("project setup"),
                ..ANY
            },
            Case {
                name: "heading, no match",
                doc: HEADING,
                heading: Some("teardown"),
                section: false,
                ..ANY
            },
            Case {
                name: "heading, non-ASCII uppercase filter folds",
                doc: HEADING_UNICODE,
                heading: Some("ÉCOLE"),
                ..ANY
            },
            Case {
                name: "heading, non-ASCII lowercase filter folds",
                doc: HEADING_UNICODE,
                heading: Some("école"),
                ..ANY
            },
        ]);
    }

    #[test]
    fn table_filter() {
        check(&[
            Case {
                name: "has_table with a table",
                doc: TABLE,
                has_table: true,
                ..ANY
            },
            Case {
                name: "has_table without a table",
                doc: PLAIN,
                has_table: true,
                section: false,
                ..ANY
            },
            Case {
                name: "table present, filter off",
                doc: TABLE,
                ..ANY
            },
        ]);
    }

    #[test]
    fn code_filter() {
        check(&[
            Case {
                name: "any language, with a code block",
                doc: CODE_RUST,
                has_code: Some(""),
                ..ANY
            },
            Case {
                name: "any language, without a code block",
                doc: PLAIN,
                has_code: Some(""),
                section: false,
                ..ANY
            },
            Case {
                name: "language, filter uppercase",
                doc: CODE_RUST,
                has_code: Some("RUST"),
                ..ANY
            },
            Case {
                name: "language, filter lowercase",
                doc: CODE_RUST,
                has_code: Some("rust"),
                ..ANY
            },
            Case {
                name: "language, other language",
                doc: CODE_RUST,
                has_code: Some("python"),
                section: false,
                ..ANY
            },
            Case {
                name: "language, second of two blocks",
                doc: CODE_TWO,
                has_code: Some("Python"),
                ..ANY
            },
            Case {
                name: "language, no code block",
                doc: PLAIN,
                has_code: Some("rust"),
                section: false,
                ..ANY
            },
            Case {
                name: "block without a language, any language",
                doc: CODE_NO_LANG,
                has_code: Some(""),
                ..ANY
            },
            Case {
                name: "block without a language, named language",
                doc: CODE_NO_LANG,
                has_code: Some("rust"),
                section: false,
                ..ANY
            },
        ]);
    }

    #[test]
    fn section_filters_combine_with_and() {
        check(&[
            Case {
                name: "heading, table and language all match",
                doc: "# Project Setup\n\n| A |\n|---|\n| 1 |\n\n```rust\nx\n```\n",
                heading: Some("setup"),
                has_table: true,
                has_code: Some("rust"),
                ..ANY
            },
            Case {
                name: "heading and table match, language does not",
                doc: "# Project Setup\n\n| A |\n|---|\n| 1 |\n\n```rust\nx\n```\n",
                heading: Some("setup"),
                has_table: true,
                has_code: Some("python"),
                section: false,
                ..ANY
            },
        ]);
    }
}
