use std::path::{Path, PathBuf};

use super::Link;

/// Build the `Link` for one link destination reported by the parser.
///
/// Returns `None` for an empty destination or a pure anchor (`#section`).
/// `file_dir` is the parent directory of the source file, used to resolve relative paths.
pub fn make_link(target: &str, line: usize, file_dir: &Path) -> Option<Link> {
    if target.is_empty() || target.starts_with('#') {
        return None;
    }

    Some(Link {
        target: target.to_owned(),
        resolved_path: resolve_link_path(target, file_dir),
        line,
    })
}

fn resolve_link_path(target: &str, file_dir: &Path) -> Option<PathBuf> {
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with('#')
        || target.starts_with("mailto:")
    {
        return None;
    }

    let path_part = target.split('#').next().unwrap_or(target);
    if path_part.is_empty() {
        return None;
    }

    let resolved = file_dir.join(path_part);
    Some(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::markdown::{parse_content, parse_file};

    /// `(target, line)` of every link `parse_content` finds.
    fn found(md: &str) -> Vec<(String, usize)> {
        parse_content(md)
            .links
            .into_iter()
            .map(|l| (l.target, l.line))
            .collect()
    }

    fn pair(target: &str, line: usize) -> (String, usize) {
        (target.to_owned(), line)
    }

    #[test]
    fn extracts_markdown_links() {
        let links = parse_content("# Title\n\nSee [other](./other.md) for details.\n").links;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].target, "./other.md");
        assert_eq!(links[0].resolved_path, Some(PathBuf::from("./other.md")));
        assert_eq!(links[0].line, 3);
    }

    #[test]
    fn resolves_against_the_files_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("note.md");
        std::fs::write(&path, "[a](./a.md) and [b](sub/b.md#h)\n").unwrap();
        let links = parse_file(&path).unwrap().links;
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].resolved_path, Some(tmp.path().join("./a.md")));
        assert_eq!(links[1].resolved_path, Some(tmp.path().join("sub/b.md")));
    }

    #[test]
    fn external_urls_are_reported_without_a_path() {
        let links =
            parse_content("Visit [site](https://example.com) and [mail](mailto:a@b.com).\n").links;
        assert_eq!(links.len(), 2);
        assert!(links[0].resolved_path.is_none());
        assert!(links[1].resolved_path.is_none());
    }

    #[test]
    fn skips_pure_anchors_and_keeps_the_file_part() {
        let links = parse_content("[jump](#section) and [file](./doc.md#heading).\n").links;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].resolved_path, Some(PathBuf::from("./doc.md")));
    }

    #[test]
    fn skips_empty_targets() {
        assert_eq!(found("[empty]() and [angle](<>)\n"), vec![]);
    }

    #[test]
    fn line_numbers_accurate() {
        let md = "# Title\n\nParagraph.\n\nSee [a](./a.md) here.\n\nAnother [b](./b.md).\n";
        assert_eq!(found(md), [pair("./a.md", 5), pair("./b.md", 7)]);
    }

    #[test]
    fn multiple_links_same_line() {
        assert_eq!(
            found("[a](./a.md) and [b](./b.md)\n"),
            [pair("./a.md", 1), pair("./b.md", 1)]
        );
    }

    #[test]
    fn title_is_not_part_of_the_target() {
        assert_eq!(
            found("[t](exists.md \"A title\") [s](x.md 'single') [p](y.md (paren))\n"),
            [pair("exists.md", 1), pair("x.md", 1), pair("y.md", 1)]
        );
    }

    #[test]
    fn angle_brackets_are_not_part_of_the_target() {
        assert_eq!(found("[a](<exists.md>)\n"), [pair("exists.md", 1)]);
    }

    #[test]
    fn parentheses_in_the_target_are_kept() {
        assert_eq!(found("[p](x_(1).md)\n"), [pair("x_(1).md", 1)]);
    }

    #[test]
    fn reference_links_use_their_definition() {
        let md = "A [ref][r1], [coll][] and [short].\n\n[r1]: one.md\n[coll]: two.md\n[short]: three.md \"T\"\n";
        assert_eq!(
            found(md),
            [pair("one.md", 1), pair("two.md", 1), pair("three.md", 1)]
        );
    }

    #[test]
    fn reference_without_definition_is_not_a_link() {
        assert_eq!(found("[ref][nope] and [coll][] and [short]\n"), vec![]);
    }

    #[test]
    fn images_are_not_links() {
        assert_eq!(
            found("![alt](img.png) and ![ref][i]\n\n[i]: img.png\n"),
            vec![]
        );
    }

    #[test]
    fn autolinks_are_not_reported() {
        assert_eq!(
            found("<https://example.com> and <mail@example.com>\n"),
            vec![]
        );
    }

    #[test]
    fn code_span_and_fenced_block_are_not_scanned() {
        let md = "A `[span](span.md)` here.\n\n```md\n[fenced](fenced.md)\n```\n\n    [indented](indented.md)\n\n[real](real.md)\n";
        assert_eq!(found(md), [pair("real.md", 9)]);
    }

    #[test]
    fn frontmatter_is_not_scanned() {
        let md = "---\nlink: \"[fm](fm.md)\"\n---\n# T\n\n[body](body.md)\n";
        assert_eq!(found(md), [pair("body.md", 6)]);
    }

    #[test]
    fn links_in_headings_tables_quotes_and_nested_lists_are_kept() {
        let md =
            "# H [h](h.md)\n\n| c |\n|---|\n| [t](t.md) |\n\n> [q](q.md)\n\n- a\n  - [n](n.md)\n";
        assert_eq!(
            found(md),
            [
                pair("h.md", 1),
                pair("t.md", 5),
                pair("q.md", 7),
                pair("n.md", 10)
            ]
        );
    }

    #[test]
    fn link_before_the_first_frontmatter_block_and_in_later_entries_is_kept() {
        let md = "# Pre [pre](pre.md)\n\n---\ntype: a\n---\n# One\n\n[one](one.md)\n\n---\ntype: b\n---\n# Two\n\n[two](two.md)\n";
        assert_eq!(
            found(md),
            [pair("pre.md", 1), pair("one.md", 8), pair("two.md", 15)]
        );
    }

    #[test]
    fn link_text_spanning_lines_reports_the_line_it_starts_on() {
        assert_eq!(found("[multi\nline](exists.md)\n"), [pair("exists.md", 1)]);
    }
}
