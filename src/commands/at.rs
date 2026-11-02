use super::Outcome;
use crate::cli::{AtArgs, OutputFormat};
use crate::error::{MddError, Result};
use crate::output::{self, AtOutput};
use crate::parse::Section;
use crate::parse::markdown::parse_file;

pub fn run(args: &AtArgs, format: OutputFormat) -> Result<Outcome> {
    let AtArgs { path, line, level } = args;
    let (line, level) = (*line, *level);
    if !path.exists() {
        return Err(MddError::FileNotFound { path: path.clone() });
    }

    let md = parse_file(path)?;

    if line < 1 || line > md.total_lines {
        return Err(MddError::LineOutOfRange {
            line,
            total_lines: md.total_lines,
        });
    }

    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    let containing: Vec<&Section> = all_sections
        .iter()
        .filter(|s| {
            let end = s.offset + s.limit - 1;
            s.offset <= line && line <= end
        })
        .copied()
        .collect();

    let Some(&last) = containing.last() else {
        return Ok(Outcome::NothingFound(format!(
            "line {line} is before any heading"
        )));
    };

    let mut target = last;

    for _ in 0..level {
        let parent = containing.iter().rev().find(|s| s.level < target.level);
        match parent {
            Some(p) => target = p,
            None => break,
        }
    }

    let ancestry: Vec<&Section> = containing
        .iter()
        .filter(|s| s.level < target.level || std::ptr::eq(**s, target))
        .copied()
        .collect();
    let breadcrumb: Vec<String> = ancestry.iter().map(|s| s.title.clone()).collect();

    let lines_vec: Vec<&str> = md.source.lines().collect();

    let section_content = target.slice(&lines_vec);

    output::render_at(
        format,
        &AtOutput {
            path,
            breadcrumb,
            section: target,
            content: section_content,
        },
    );

    Ok(Outcome::Found)
}
