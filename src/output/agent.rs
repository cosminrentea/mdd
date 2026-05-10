use super::{fmt_size, format_signature, AtOutput, SecOutput, TocDirOutput, TocFileOutput};

pub fn render_toc_file(output: &TocFileOutput) {
    let path_str = output.path.display();
    for section in &output.sections {
        let indent = "  ".repeat((section.level - output.min_level) as usize);
        let sig = format_signature(&section.features);
        let sig_suffix = if sig.is_empty() {
            String::new()
        } else {
            format!(" {}", sig)
        };
        println!(
            "{}[{}:{},{}] {} {}{}",
            indent,
            path_str,
            section.offset,
            section.limit,
            "#".repeat(section.level as usize),
            section.title,
            sig_suffix,
        );
    }
}

pub fn render_toc_dir(output: &TocDirOutput) {
    for entry in &output.entries {
        println!(
            "[{}] ({} lines, {}) {}",
            entry.rel_path.display(),
            entry.total_lines,
            fmt_size(entry.byte_size),
            entry.first_heading,
        );

        for s in &entry.sub_headings {
            let indent = "  ".repeat((s.level - entry.min_level) as usize);
            println!("  {}{}", indent, s.title);
        }
    }

    eprintln!("\n{} files", output.total_files);
}

pub fn render_sec(output: &SecOutput) {
    let mut first = true;
    for m in &output.matches {
        if !first {
            println!("\n---\n");
        }
        first = false;
        println!("[{}:{},{}]", output.path.display(), m.offset, m.limit,);
        print!("{}", m.content);
    }
}

pub fn render_at(output: &AtOutput) {
    println!(
        "[{}:{},{}] {} {}",
        output.path.display(),
        output.section.offset,
        output.section.limit,
        "#".repeat(output.section.level as usize),
        output.breadcrumb.last().unwrap_or(&String::new()),
    );
    print!("{}", output.content);
}
