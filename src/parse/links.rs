use std::path::Path;

use regex::Regex;

use super::{Link, LinkKind};

/// Extract all standard markdown links from content using regex.
///
/// Pattern: `[text](target)` -- skips pure anchors (#section) and empty targets.
/// `file_dir` is the parent directory of the source file, used to resolve relative paths.
pub fn extract_links(content: &str, file_dir: &Path) -> Vec<Link> {
    let mut links = Vec::new();

    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    let md_re = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    for cap in md_re.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let target = cap.get(2).unwrap().as_str().to_owned();

        if target.is_empty() || target.starts_with('#') {
            continue;
        }

        let line = byte_offset_to_line(&line_starts, full_match.start());
        let resolved = resolve_link_path(&target, file_dir);

        links.push(Link {
            kind: LinkKind::Markdown,
            target,
            resolved_path: resolved,
            line,
        });
    }

    links
}

fn resolve_link_path(target: &str, file_dir: &Path) -> Option<std::path::PathBuf> {
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

fn byte_offset_to_line(line_starts: &[usize], byte_offset: usize) -> usize {
    match line_starts.binary_search(&byte_offset) {
        Ok(idx) => idx + 1,
        Err(idx) => idx,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn extracts_markdown_links() {
        let content = "# Title\n\nSee [other](./other.md) for details.\n";
        let links = extract_links(content, Path::new("/docs/notes"));
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].kind, LinkKind::Markdown);
        assert_eq!(links[0].target, "./other.md");
        assert_eq!(
            links[0].resolved_path,
            Some(PathBuf::from("/docs/notes/./other.md"))
        );
        assert_eq!(links[0].line, 3);
    }

    #[test]
    fn skips_external_urls() {
        let content = "Visit [site](https://example.com) and [mail](mailto:a@b.com).\n";
        let links = extract_links(content, Path::new("/docs"));
        assert_eq!(links.len(), 2);
        assert!(links[0].resolved_path.is_none());
        assert!(links[1].resolved_path.is_none());
    }

    #[test]
    fn handles_anchors() {
        let content = "[jump](#section) and [file](./doc.md#heading).\n";
        let links = extract_links(content, Path::new("/docs"));
        assert_eq!(links.len(), 1);
        assert_eq!(
            links[0].resolved_path,
            Some(PathBuf::from("/docs/./doc.md"))
        );
    }

    #[test]
    fn line_numbers_accurate() {
        let content = "# Title\n\nParagraph.\n\nSee [a](./a.md) here.\n\nAnother [b](./b.md).\n";
        let links = extract_links(content, Path::new("/docs"));
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].line, 5);
        assert_eq!(links[0].target, "./a.md");
        assert_eq!(links[1].line, 7);
        assert_eq!(links[1].target, "./b.md");
    }

    #[test]
    fn multiple_links_same_line() {
        let content = "[a](./a.md) and [b](./b.md)\n";
        let links = extract_links(content, Path::new("/docs"));
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].line, 1);
        assert_eq!(links[1].line, 1);
    }
}
