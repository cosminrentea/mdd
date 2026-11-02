use std::collections::BTreeSet;

use crate::cli::FmArgs;
use crate::error::{MddError, Result};
use crate::parse::frontmatter::field_as_string;
use crate::parse::markdown::parse_file;
use crate::query::walk_md_files;

pub fn run(args: &FmArgs) -> Result<()> {
    let FmArgs { path, list_values } = args;
    if !path.exists() {
        return Err(MddError::FileNotFound { path: path.clone() });
    }

    if let Some(field) = list_values {
        return list_field_values(path, field);
    }

    show_frontmatter(path)
}

fn list_field_values(path: &std::path::Path, field: &str) -> Result<()> {
    let mut values = BTreeSet::new();

    if path.is_file() {
        collect_field_values(path, field, &mut values)?;
    } else {
        let files = walk_md_files(path);

        for file_path in &files {
            let _ = collect_field_values(file_path, field, &mut values);
        }
    }

    if values.is_empty() {
        eprintln!("No values found for field '{field}'");
    } else {
        for val in &values {
            println!("{val}");
        }
        eprintln!("\n{} unique values", values.len());
    }

    Ok(())
}

fn collect_field_values(
    path: &std::path::Path,
    field: &str,
    values: &mut BTreeSet<String>,
) -> Result<()> {
    let md = parse_file(path)?;
    for entry in &md.entries {
        if let Some(ref fm) = entry.frontmatter
            && let Some(val) = field_as_string(fm, field)
        {
            values.insert(val);
        }
    }
    Ok(())
}

fn show_frontmatter(path: &std::path::Path) -> Result<()> {
    if path.is_file() {
        let md = parse_file(path)?;
        for entry in &md.entries {
            if let Some(ref fm) = entry.frontmatter {
                for (key, val) in &fm.fields {
                    println!("{}: {}", key, format_yaml_value(val));
                }
            }
        }
    } else {
        let files = walk_md_files(path);

        let mut count = 0;
        let mut with_fm = 0;
        for file_path in &files {
            count += 1;
            let Ok(md) = parse_file(file_path) else {
                continue;
            };
            let has_fm = md.entries.iter().any(|e| e.frontmatter.is_some());
            if has_fm {
                with_fm += 1;
            }
        }
        println!(
            "{}/{} files have frontmatter ({:.0}%)",
            with_fm,
            count,
            if count > 0 {
                f64::from(with_fm) / f64::from(count) * 100.0
            } else {
                0.0
            }
        );
    }
    Ok(())
}

fn format_yaml_value(val: &serde_norway::Value) -> String {
    match val {
        serde_norway::Value::String(s) => s.clone(),
        serde_norway::Value::Number(n) => n.to_string(),
        serde_norway::Value::Bool(b) => b.to_string(),
        serde_norway::Value::Null => "null".to_owned(),
        serde_norway::Value::Sequence(seq) => {
            let items: Vec<String> = seq.iter().map(format_yaml_value).collect();
            format!("[{}]", items.join(", "))
        }
        serde_norway::Value::Mapping(_) | serde_norway::Value::Tagged(_) => {
            serde_norway::to_string(val)
                .unwrap_or_default()
                .trim()
                .to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_format_yaml_value() {
        let raw = concat!(
            "map:\n",
            "  yes_s: \"yes\"\n",
            "  one: \"1\"\n",
            "  empty: \"\"\n",
            "  colon: \"a: b\"\n",
            "  lead: \" x\"\n",
            "  trail: \"x \"\n",
            "  real_bool: true\n",
            "  nul: ~\n",
            "  n: 5\n",
            "  f: 1.5\n",
            "  nested:\n",
            "    k: v\n",
            "multi: \"l1\\nl2\\n\"\n",
            "seq_of_maps:\n",
            "  - x: 1\n",
            "    y: \"yes\"\n",
            "  - z: [1, 2]\n",
            "seq_scalars: [a, \"b c\", \"1\", yes, \"\"]\n",
            "tag: !custom x\n",
            "tag_map: !custom\n",
            "  k: v\n",
            "tag_seq: !custom [1, 2]\n",
        );
        let fm = crate::parse::frontmatter::parse_frontmatter(raw).unwrap();
        let shown = |key: &str| format_yaml_value(&fm.fields[key]);

        assert_eq!(
            shown("map"),
            "yes_s: yes\none: '1'\nempty: ''\ncolon: 'a: b'\nlead: ' x'\ntrail: 'x '\nreal_bool: true\nnul: null\nn: 5\nf: 1.5\nnested:\n  k: v"
        );
        assert_eq!(shown("multi"), "l1\nl2\n");
        assert_eq!(shown("seq_of_maps"), "[x: 1\ny: yes, z:\n- 1\n- 2]");
        assert_eq!(shown("seq_scalars"), "[a, b c, 1, yes, ]");
        assert_eq!(shown("tag"), "!custom x");
        assert_eq!(shown("tag_map"), "!custom\nk: v");
        assert_eq!(shown("tag_seq"), "!custom\n- 1\n- 2");
    }

    #[test]
    fn pins_format_yaml_value_numbers() {
        let raw = "a: 1e3\nb: .5\nc: -0.0\nd: .inf\ne: -.inf\nf: .nan\ng: 18446744073709551615\nh: -9223372036854775808\ni: [1e3, .inf, .nan]";
        let fm = crate::parse::frontmatter::parse_frontmatter(raw).unwrap();
        let shown = |key: &str| format_yaml_value(&fm.fields[key]);
        assert_eq!(shown("a"), "1000.0");
        assert_eq!(shown("b"), "0.5");
        assert_eq!(shown("c"), "-0.0");
        assert_eq!(shown("d"), ".inf");
        assert_eq!(shown("e"), "-.inf");
        assert_eq!(shown("f"), ".nan");
        assert_eq!(shown("g"), u64::MAX.to_string());
        assert_eq!(shown("h"), i64::MIN.to_string());
        assert_eq!(shown("i"), "[1000.0, .inf, .nan]");
    }
}
