// YAML frontmatter parsing.
//
// Strategy: use serde_norway to parse the YAML text into a BTreeMap for queries.

use std::collections::BTreeMap;

use super::Frontmatter;

/// Parse raw YAML text (the content between `---` delimiters, not including them)
/// into a Frontmatter struct.
///
/// Returns None if the YAML is empty or cannot be parsed as a mapping.
///
/// We return Option here (not Result) because malformed frontmatter in a single
/// file shouldn't abort parsing of the entire document. The caller decides
/// whether to warn (default) or fail (--strict mode).
pub fn parse_frontmatter(raw_yaml: &str) -> Option<Frontmatter> {
    // Trim the raw YAML -- some files have trailing whitespace after the last field.
    let trimmed = raw_yaml.trim();
    if trimmed.is_empty() {
        return None;
    }

    // serde_norway::from_str can parse into various types. We want a BTreeMap
    // because it preserves key ordering (alphabetical) for deterministic output.
    let fields: BTreeMap<String, serde_norway::Value> = serde_norway::from_str(trimmed).ok()?;

    Some(Frontmatter { fields })
}

/// Extract the value of a specific field from frontmatter as a string.
///
/// Handles common YAML value types: strings, numbers, bools.
/// Returns None if the field doesn't exist or is a complex type (sequence/mapping).
pub fn field_as_string(fm: &Frontmatter, key: &str) -> Option<String> {
    let value = fm.fields.get(key)?;
    match value {
        serde_norway::Value::String(s) => Some(s.clone()),
        serde_norway::Value::Number(n) => Some(n.to_string()),
        serde_norway::Value::Bool(b) => Some(b.to_string()),
        serde_norway::Value::Null => Some("null".to_owned()),
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
        let fm = parse_frontmatter(yaml).expect("should parse");

        assert_eq!(field_as_string(&fm, "type"), Some("reference".to_owned()));
        assert_eq!(field_as_string(&fm, "topic"), Some("testing".to_owned()));
        assert_eq!(field_as_string(&fm, "weight"), Some("0.8".to_owned()));
    }

    #[test]
    fn returns_none_for_empty() {
        assert!(parse_frontmatter("").is_none());
        assert!(parse_frontmatter("   \n  ").is_none());
    }

    #[test]
    fn returns_none_for_invalid_yaml() {
        // A bare string isn't a mapping, so from_str::<BTreeMap> fails.
        assert!(parse_frontmatter("just a string").is_none());
    }

    #[test]
    fn handles_multiline_values() {
        let yaml = "type: note\ndescription: |\n  This is a\n  multi-line value\n";
        let fm = parse_frontmatter(yaml).expect("should parse");
        assert_eq!(field_as_string(&fm, "type"), Some("note".to_owned()));
        // Multi-line strings are still strings in serde_norway
        let desc = field_as_string(&fm, "description").unwrap();
        assert!(desc.contains("multi-line"));
    }

    #[test]
    fn field_as_string_handles_types() {
        let yaml = "count: 42\nenabled: true\nnull_val: null\n";
        let fm = parse_frontmatter(yaml).unwrap();
        assert_eq!(field_as_string(&fm, "count"), Some("42".to_owned()));
        assert_eq!(field_as_string(&fm, "enabled"), Some("true".to_owned()));
        assert_eq!(field_as_string(&fm, "null_val"), Some("null".to_owned()));
        assert_eq!(field_as_string(&fm, "missing"), None);
    }

    /// (case name, raw frontmatter text, expected fields as JSON; None = not a mapping / invalid).
    /// Non-finite floats become JSON null, so they are pinned in `pins_non_finite_floats`.
    #[rustfmt::skip]
    const PARSE_CASES: &[(&str, &str, Option<&str>)] = &[
        ("plain_strings", "title: hello\nname: world", Some(r#"{"name":"world","title":"hello"}"#)),
        ("ints", "a: -5\nb: 18446744073709551615\nc: 0x1F\nd: 0o17\ne: 42\nf: -9223372036854775808", Some(r#"{"a":-5,"b":18446744073709551615,"c":31,"d":15,"e":42,"f":-9223372036854775808}"#)),
        ("int_above_u64", "a: 36893488147419103232", None),
        ("int_above_u64_neg", "a: -36893488147419103232", None),
        ("floats", "a: 1.5\nb: -0.25\nc: 1.0\nd: 3.14159", Some(r#"{"a":1.5,"b":-0.25,"c":1.0,"d":3.14159}"#)),
        ("yaml11_leftovers", "a: 012\nb: 1_000\nc: Yes\nd: NO\ne: +1\nf: .5\ng: 1e3", Some(r#"{"a":"012","b":"1_000","c":"Yes","d":"NO","e":1,"f":0.5,"g":1000.0}"#)),
        ("yaml11_extra", "a: 0b101\nb: 1:30\nc: 0x\nd: 1.\ne: 0.\nf: -0\ng: +.5\nh: 1E3\ni: 1e+3", Some(r#"{"a":5,"b":"1:30","c":"0x","d":1.0,"e":0.0,"f":0,"g":0.5,"h":1000.0,"i":1000.0}"#)),
        ("bools", "a: true\nb: false\nc: True\nd: FALSE\ne: TRUE", Some(r#"{"a":true,"b":false,"c":true,"d":false,"e":true}"#)),
        ("yes_on_no", "a: yes\nb: on\nc: no\nd: off\ne: y\nf: n", Some(r#"{"a":"yes","b":"on","c":"no","d":"off","e":"y","f":"n"}"#)),
        ("nulls", "a: ~\nb:\nc: null\nd: Null\ne: NULL", Some(r#"{"a":null,"b":null,"c":null,"d":null,"e":null}"#)),
        ("sequences", "a: [1, two, true, ~]\nb:\n  - x\n  - y\nc: []", Some(r#"{"a":[1,"two",true,null],"b":["x","y"],"c":[]}"#)),
        ("nested_maps", "a:\n  b:\n    c: 1\n  d: [x]\ne: {f: 1, g: two}", Some(r#"{"a":{"b":{"c":1},"d":["x"]},"e":{"f":1,"g":"two"}}"#)),
        ("block_literal", "a: |\n  l1\n  l2\nb: end", Some(r#"{"a":"l1\nl2\n","b":"end"}"#)),
        ("block_folded", "a: >\n  f1\n  f2\n\n  f3\nb: end", Some(r#"{"a":"f1 f2\nf3\n","b":"end"}"#)),
        ("block_strip_keep", "a: |-\n  x\nb: |+\n  y\n\nc: >-\n  p\n  q\n", Some(r#"{"a":"x","b":"y\n\n","c":"p q"}"#)),
        ("quoted", "a: \"x y\"\nb: 'it''s'\nc: \"tab\there\"\nd: \"123\"\ne: \"\"\nf: ''", Some(r#"{"a":"x y","b":"it's","c":"tab\there","d":"123","e":"","f":""}"#)),
        ("unicode", "a: héllo\nb: 日本語\nc: \"\\u00e9\"\nd: \"\\U0001F600\"", Some(r#"{"a":"héllo","b":"日本語","c":"é","d":"😀"}"#)),
        ("anchors_aliases", "a: &x [1, 2]\nb: *x\nc: &s hi\nd: *s", Some(r#"{"a":[1,2],"b":[1,2],"c":"hi","d":"hi"}"#)),
        ("merge_key", "base: &b\n  k: 1\nd:\n  <<: *b\n  z: 2", Some(r#"{"base":{"k":1},"d":{"<<":{"k":1},"z":2}}"#)),
        ("tag_custom", "a: !custom x", Some(r#"{"a":{"!custom":"x"}}"#)),
        ("tag_str", "a: !!str 123", Some(r#"{"a":"123"}"#)),
        ("tag_binary", "a: !!binary aGk=", Some(r#"{"a":"aGk="}"#)),
        ("tag_int", "a: !!int \"5\"", Some(r#"{"a":5}"#)),
        ("tag_seq", "a: !custom [1, 2]", Some(r#"{"a":{"!custom":[1,2]}}"#)),
        ("tag_top_level_map", "!custom\na: 1", Some(r#"{"a":1}"#)),
        ("dates", "d: 2026-11-01\nt: 2026-11-01T10:00:00Z", Some(r#"{"d":"2026-11-01","t":"2026-11-01T10:00:00Z"}"#)),
        ("duplicate_keys", "a: 1\na: 2", Some(r#"{"a":2}"#)),
        ("invalid_unclosed", "key: [unclosed", None),
        ("invalid_scalar", "just a string", None),
        ("top_level_list", "- a\n- b", None),
        ("comment_only", "# only a comment", Some("{}")),
        ("non_string_key_int", "1: a", Some(r#"{"1":"a"}"#)),
        ("non_string_key_bool", "true: a", Some(r#"{"true":"a"}"#)),
        ("non_string_key_mixed", "1: a\nb: c", Some(r#"{"1":"a","b":"c"}"#)),
        ("complex_key", "? [x]\n: y", None),
        ("empty_key", "\"\": v", Some(r#"{"":"v"}"#)),
        ("null_key_word", "null: v", Some(r#"{"null":"v"}"#)),
        ("null_key_tilde", "~: v", Some(r#"{"~":"v"}"#)),
        ("bare_colon_key", ": v", None),
        ("multiline_double_quoted", "a: \"line1\\nline2\"", Some(r#"{"a":"line1\nline2"}"#)),
        ("multiline_plain", "a: first\n  second\n  third", Some(r#"{"a":"first second third"}"#)),
        ("comments", "a: 1 # trailing\n# full line\nb: 2", Some(r#"{"a":1,"b":2}"#)),
        ("directive", "%YAML 1.2\n---\na: 1", Some(r#"{"a":1}"#)),
        ("end_marker", "a: 1\n...\n", Some(r#"{"a":1}"#)),
        ("multi_document", "a: 1\n---\nb: 2", None),
        ("leading_doc_start", "---\na: 1", Some(r#"{"a":1}"#)),
        ("crlf", "a: 1\r\nb: |\r\n  x\r\n  y\r\n", Some(r#"{"a":1,"b":"x\ny"}"#)),
        ("tab_indent", "a:\n\tb: 1", None),
        ("tab_after_colon", "a:\tb", Some(r#"{"a":"b"}"#)),
        ("flow_seq_nested", "a: [[1, 2], [3]]\nb: [{x: 1}, {y: 2}]", Some(r#"{"a":[[1,2],[3]],"b":[{"x":1},{"y":2}]}"#)),
        ("seq_of_maps", "a:\n  - x: 1\n    y: 2\n  - z: 3", Some(r#"{"a":[{"x":1,"y":2},{"z":3}]}"#)),
        ("key_with_spaces", "my key: v\nother-key: w", Some(r#"{"my key":"v","other-key":"w"}"#)),
        ("value_with_colon", "a: b: c", None),
        ("long_float_exp", "a: 1.5e300\nb: 1e400", Some(r#"{"a":1.5e+300,"b":"1e400"}"#)),
        ("octal_like_string", "a: 0o9\nb: 0x1G", Some(r#"{"a":"0o9","b":"0x1G"}"#)),
    ];

    /// (case name, key, expected `field_as_string`).
    const SCALAR_CASES: &[(&str, &str, Option<&str>)] = &[
        ("plain_strings", "name", Some("world")),
        ("plain_strings", "title", Some("hello")),
        ("ints", "a", Some("-5")),
        ("ints", "b", Some("18446744073709551615")),
        ("ints", "c", Some("31")),
        ("ints", "d", Some("15")),
        ("ints", "e", Some("42")),
        ("ints", "f", Some("-9223372036854775808")),
        ("floats", "a", Some("1.5")),
        ("floats", "b", Some("-0.25")),
        ("floats", "c", Some("1.0")),
        ("floats", "d", Some("3.14159")),
        ("yaml11_leftovers", "a", Some("012")),
        ("yaml11_leftovers", "b", Some("1_000")),
        ("yaml11_leftovers", "c", Some("Yes")),
        ("yaml11_leftovers", "d", Some("NO")),
        ("yaml11_leftovers", "e", Some("1")),
        ("yaml11_leftovers", "f", Some("0.5")),
        ("yaml11_leftovers", "g", Some("1000.0")),
        ("yaml11_extra", "a", Some("5")),
        ("yaml11_extra", "b", Some("1:30")),
        ("yaml11_extra", "c", Some("0x")),
        ("yaml11_extra", "d", Some("1.0")),
        ("yaml11_extra", "e", Some("0.0")),
        ("yaml11_extra", "f", Some("0")),
        ("yaml11_extra", "g", Some("0.5")),
        ("yaml11_extra", "h", Some("1000.0")),
        ("yaml11_extra", "i", Some("1000.0")),
        ("bools", "a", Some("true")),
        ("bools", "b", Some("false")),
        ("bools", "c", Some("true")),
        ("bools", "d", Some("false")),
        ("bools", "e", Some("true")),
        ("yes_on_no", "a", Some("yes")),
        ("yes_on_no", "b", Some("on")),
        ("yes_on_no", "c", Some("no")),
        ("yes_on_no", "d", Some("off")),
        ("yes_on_no", "e", Some("y")),
        ("yes_on_no", "f", Some("n")),
        ("nulls", "a", Some("null")),
        ("nulls", "b", Some("null")),
        ("nulls", "c", Some("null")),
        ("nulls", "d", Some("null")),
        ("nulls", "e", Some("null")),
        ("sequences", "a", None),
        ("sequences", "b", None),
        ("sequences", "c", None),
        ("nested_maps", "a", None),
        ("nested_maps", "e", None),
        ("block_literal", "a", Some("l1\nl2\n")),
        ("block_literal", "b", Some("end")),
        ("block_folded", "a", Some("f1 f2\nf3\n")),
        ("block_folded", "b", Some("end")),
        ("block_strip_keep", "a", Some("x")),
        ("block_strip_keep", "b", Some("y\n\n")),
        ("block_strip_keep", "c", Some("p q")),
        ("quoted", "a", Some("x y")),
        ("quoted", "b", Some("it's")),
        ("quoted", "c", Some("tab\there")),
        ("quoted", "d", Some("123")),
        ("quoted", "e", Some("")),
        ("quoted", "f", Some("")),
        ("unicode", "a", Some("héllo")),
        ("unicode", "b", Some("日本語")),
        ("unicode", "c", Some("é")),
        ("unicode", "d", Some("😀")),
        ("anchors_aliases", "a", None),
        ("anchors_aliases", "b", None),
        ("anchors_aliases", "c", Some("hi")),
        ("anchors_aliases", "d", Some("hi")),
        ("merge_key", "base", None),
        ("merge_key", "d", None),
        ("tag_custom", "a", None),
        ("tag_str", "a", Some("123")),
        ("tag_binary", "a", Some("aGk=")),
        ("tag_int", "a", Some("5")),
        ("tag_seq", "a", None),
        ("tag_top_level_map", "a", Some("1")),
        ("dates", "d", Some("2026-11-01")),
        ("dates", "t", Some("2026-11-01T10:00:00Z")),
        ("duplicate_keys", "a", Some("2")),
        ("non_string_key_int", "1", Some("a")),
        ("non_string_key_bool", "true", Some("a")),
        ("non_string_key_mixed", "1", Some("a")),
        ("non_string_key_mixed", "b", Some("c")),
        ("empty_key", "", Some("v")),
        ("null_key_word", "null", Some("v")),
        ("null_key_tilde", "~", Some("v")),
        ("multiline_double_quoted", "a", Some("line1\nline2")),
        ("multiline_plain", "a", Some("first second third")),
        ("comments", "a", Some("1")),
        ("comments", "b", Some("2")),
        ("directive", "a", Some("1")),
        ("end_marker", "a", Some("1")),
        ("leading_doc_start", "a", Some("1")),
        ("crlf", "a", Some("1")),
        ("crlf", "b", Some("x\ny")),
        ("tab_after_colon", "a", Some("b")),
        ("flow_seq_nested", "a", None),
        ("flow_seq_nested", "b", None),
        ("seq_of_maps", "a", None),
        ("key_with_spaces", "my key", Some("v")),
        ("key_with_spaces", "other-key", Some("w")),
        ("long_float_exp", "a", Some("1.5e300")),
        ("long_float_exp", "b", Some("1e400")),
        ("octal_like_string", "a", Some("0o9")),
        ("octal_like_string", "b", Some("0x1G")),
    ];

    #[test]
    fn pins_parsed_values() {
        for (name, raw, expected) in PARSE_CASES {
            let actual = parse_frontmatter(raw).map(|fm| serde_json::to_value(&fm.fields).unwrap());
            let expected =
                expected.map(|json| serde_json::from_str::<serde_json::Value>(json).unwrap());
            assert_eq!(actual, expected, "case {name}");
        }
    }

    #[test]
    fn pins_field_as_string() {
        for (name, key, expected) in SCALAR_CASES {
            let (_, raw, _) = PARSE_CASES.iter().find(|c| c.0 == *name).unwrap();
            let fm = parse_frontmatter(raw).unwrap();
            assert_eq!(
                field_as_string(&fm, key).as_deref(),
                *expected,
                "case {name} key {key:?}"
            );
        }
    }

    #[test]
    fn pins_non_finite_floats() {
        for raw in ["a: .inf", "a: .Inf", "a: .INF", "a: +.inf"] {
            let fm = parse_frontmatter(raw).unwrap();
            let n = fm.fields["a"].as_f64().unwrap_or_else(|| panic!("{raw}"));
            assert!(n.is_infinite() && n.is_sign_positive(), "{raw}");
            assert_eq!(field_as_string(&fm, "a").as_deref(), Some(".inf"), "{raw}");
        }
        for raw in ["a: -.inf", "a: -.Inf"] {
            let fm = parse_frontmatter(raw).unwrap();
            let n = fm.fields["a"].as_f64().unwrap_or_else(|| panic!("{raw}"));
            assert!(n.is_infinite() && n.is_sign_negative(), "{raw}");
            assert_eq!(field_as_string(&fm, "a").as_deref(), Some("-.inf"), "{raw}");
        }
        for raw in ["a: .nan", "a: .NaN", "a: .NAN"] {
            let fm = parse_frontmatter(raw).unwrap();
            let n = fm.fields["a"].as_f64().unwrap_or_else(|| panic!("{raw}"));
            assert!(n.is_nan(), "{raw}");
            assert_eq!(field_as_string(&fm, "a").as_deref(), Some(".nan"), "{raw}");
        }
        for raw in ["a: inf", "a: nan", "a: .iNf"] {
            let fm = parse_frontmatter(raw).unwrap();
            assert!(fm.fields["a"].as_f64().is_none(), "{raw}");
        }
    }

    #[test]
    fn pathological_inputs_return_promptly() {
        use std::fmt::Write;

        let mut deep_block = String::new();
        for level in 0..200 {
            deep_block.push_str(&"  ".repeat(level));
            deep_block.push_str("k:\n");
        }
        deep_block.push_str(&"  ".repeat(200));
        deep_block.push_str("v: 1\n");
        let deep_flow = format!("a: {}1{}", "[".repeat(200), "]".repeat(200));
        let mut bomb = String::from("a0: &a0 [x, x]\n");
        for i in 1..20 {
            writeln!(bomb, "a{i}: &a{i} [*a{0}, *a{0}]", i - 1).unwrap();
        }

        for raw in [deep_block, deep_flow, bomb] {
            let start = std::time::Instant::now();
            let _ = parse_frontmatter(&raw);
            assert!(start.elapsed() < std::time::Duration::from_secs(10));
        }
    }

    #[test]
    fn pins_printed_number_text() {
        let raw = "a: 1e3\nb: .5\nc: -0.0\nd: .inf\ne: -.inf\nf: .nan\ng: 18446744073709551615\nh: -9223372036854775808";
        let fm = parse_frontmatter(raw).unwrap();
        let printed = |key: &str| field_as_string(&fm, key).unwrap();
        assert_eq!(printed("a"), "1000.0");
        assert_eq!(printed("b"), "0.5");
        assert_eq!(printed("c"), "-0.0");
        assert_eq!(printed("d"), ".inf");
        assert_eq!(printed("e"), "-.inf");
        assert_eq!(printed("f"), ".nan");
        assert_eq!(printed("g"), u64::MAX.to_string());
        assert_eq!(printed("h"), i64::MIN.to_string());
        let neg_zero = fm.fields["c"].as_f64().unwrap();
        assert!(neg_zero == 0.0 && neg_zero.is_sign_negative());
    }

    #[test]
    fn fixture_fields_come_out_in_alphabetical_key_order() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/service-doc.md");
        let md = crate::parse::markdown::parse_file(std::path::Path::new(path)).unwrap();
        let fm = md.entries[0].frontmatter.as_ref().unwrap();
        let keys: Vec<&str> = fm.fields.keys().map(String::as_str).collect();
        assert_eq!(keys, ["last_updated", "topic", "type"]);
        assert_eq!(
            field_as_string(fm, "last_updated").as_deref(),
            Some("2026-04-18")
        );
    }
}
