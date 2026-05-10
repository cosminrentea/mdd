use owo_colors::OwoColorize;

use super::{fmt_size, AtOutput, SecOutput, TocDirOutput, TocFileOutput};

pub fn render_toc_file(output: &TocFileOutput) {
    for section in &output.sections {
        let indent = "  ".repeat((section.level - output.min_level) as usize);
        println!(
            "{}{} {} {}",
            indent,
            "-".dimmed(),
            section.title.bold(),
            format!("(offset={}, limit={})", section.offset, section.limit).dimmed(),
        );
    }
}

pub fn render_toc_dir(output: &TocDirOutput) {
    for entry in &output.entries {
        println!(
            "{} {} -- {}",
            entry.rel_path.display().to_string().bold(),
            format!(
                "({} lines, {})",
                entry.total_lines,
                fmt_size(entry.byte_size)
            )
            .dimmed(),
            entry.first_heading.green(),
        );

        for s in &entry.sub_headings {
            let indent = "  ".repeat((s.level - entry.min_level) as usize);
            println!("  {}{}", indent, s.title);
        }
    }

    eprintln!("\n{}", format!("{} files", output.total_files).dimmed());
}

pub fn render_sec(output: &SecOutput) {
    let mut first = true;
    for m in &output.matches {
        if !first {
            println!("\n{}\n", "---".dimmed());
        }
        first = false;
        print!("{}", m.content);
    }
}

pub fn render_at(output: &AtOutput) {
    eprintln!("[{}]", output.breadcrumb.join(" > ").blue());
    print!("{}", output.content);
}
