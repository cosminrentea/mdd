use std::path::Path;

use regex::Regex;

use super::{Link, LinkKind};

/// Extract all links from markdown content using regex.
///
/// Two patterns:
/// 1. Standard Markdown links: [text](target)
/// 2. Obsidian wiki-links: [[target]] and [[target|alias]]
///
/// Uses regex instead of pulldown-cmark to avoid double-counting when
/// wiki-links are partially interpreted as standard links by the parser.
///
/// `file_dir` is the parent directory of the source file, used to resolve relative paths.
pub fn extract_links(content: &str, file_dir: &Path) -> Vec<Link> {
    let mut links = Vec::new();

    // Precompute line starts for byte-offset-to-line conversion
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    // Track wiki-link byte ranges to avoid double-counting them as markdown links
    let mut wiki_ranges: Vec<(usize, usize)> = Vec::new();

    // Pass 1: Wiki-links [[target]] or [[target|alias]]
    let wiki_re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    for cap in wiki_re.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let inner = cap.get(1).unwrap().as_str();

        wiki_ranges.push((full_match.start(), full_match.end()));

        // Strip alias: [[target|display text]] -> target
        let target = inner.split('|').next().unwrap_or(inner).trim().to_owned();
        if target.is_empty() {
            continue;
        }

        let line = byte_offset_to_line(&line_starts, full_match.start());
        let resolved = resolve_wiki_link(&target, file_dir);

        links.push(Link {
            kind: LinkKind::WikiLink,
            target,
            resolved_path: resolved,
            line,
        });
    }

    // Pass 2: Standard Markdown links [text](target)
    let md_re = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").unwrap();
    for cap in md_re.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let target = cap.get(2).unwrap().as_str().to_owned();

        if target.is_empty() || target.starts_with('#') {
            continue;
        }

        // Skip if this match overlaps with a wiki-link range
        let start = full_match.start();
        if wiki_ranges.iter().any(|(ws, we)| start >= *ws && start < *we) {
            continue;
        }

        let line = byte_offset_to_line(&line_starts, start);
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

/// Resolve a standard markdown link target to an absolute path.
/// Returns None for external URLs, anchors, or unresolvable paths.
fn resolve_link_path(target: &str, file_dir: &Path) -> Option<std::path::PathBuf> {
    // Skip external URLs and anchors
    if target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with('#')
        || target.starts_with("mailto:")
    {
        return None;
    }

    // Strip fragment anchors from path
    let path_part = target.split('#').next().unwrap_or(target);
    if path_part.is_empty() {
        return None;
    }

    let resolved = file_dir.join(path_part);
    Some(resolved)
}

/// Resolve an Obsidian wiki-link target.
/// Wiki-links are filename-based: [[note-name]] resolves to note-name.md
/// in the same directory (simplified -- full Obsidian resolves vault-wide).
fn resolve_wiki_link(target: &str, file_dir: &Path) -> Option<std::path::PathBuf> {
    // Strip section references: [[note#section]] -> note
    let file_part = target.split('#').next().unwrap_or(target);
    if file_part.is_empty() {
        return None;
    }

    // Add .md extension if not present
    let filename = if file_part.ends_with(".md") {
        file_part.to_owned()
    } else {
        format!("{}.md", file_part)
    };

    let resolved = file_dir.join(filename);
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
        let links = extract_links(content, Path::new("/vault/notes"));
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].kind, LinkKind::Markdown);
        assert_eq!(links[0].target, "./other.md");
        assert_eq!(links[0].resolved_path, Some(PathBuf::from("/vault/notes/./other.md")));
        assert_eq!(links[0].line, 3);
    }

    #[test]
    fn extracts_wiki_links() {
        let content = "# Title\n\nSee [[My Note]] and [[Other|alias]].\n";
        let links = extract_links(content, Path::new("/vault"));
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].kind, LinkKind::WikiLink);
        assert_eq!(links[0].target, "My Note");
        assert_eq!(links[0].resolved_path, Some(PathBuf::from("/vault/My Note.md")));
        assert_eq!(links[1].target, "Other");
    }

    #[test]
    fn skips_external_urls() {
        let content = "Visit [site](https://example.com) and [mail](mailto:a@b.com).\n";
        let links = extract_links(content, Path::new("/vault"));
        assert_eq!(links.len(), 2);
        assert!(links[0].resolved_path.is_none());
        assert!(links[1].resolved_path.is_none());
    }

    #[test]
    fn handles_anchors() {
        let content = "[jump](#section) and [file](./doc.md#heading).\n";
        let links = extract_links(content, Path::new("/vault"));
        // Pure anchors (#section) are skipped entirely per spec
        assert_eq!(links.len(), 1);
        // File with anchor: resolves to file path (without fragment)
        assert_eq!(links[0].resolved_path, Some(PathBuf::from("/vault/./doc.md")));
    }
}
