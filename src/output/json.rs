use serde_json::{json, Value};

use super::{AtOutput, SecOutput, TocDirOutput, TocFileOutput};

pub fn render_toc_file(output: &TocFileOutput) {
    let sections: Vec<Value> = output
        .sections
        .iter()
        .map(|s| {
            json!({
                "title": s.title,
                "level": s.level,
                "offset": s.offset,
                "limit": s.limit,
                "features": features_json(&s.features),
            })
        })
        .collect();

    let doc = json!({
        "path": output.path.display().to_string(),
        "sections": sections,
    });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

pub fn render_toc_dir(output: &TocDirOutput) {
    let entries: Vec<Value> = output
        .entries
        .iter()
        .map(|e| {
            let subs: Vec<Value> = e
                .sub_headings
                .iter()
                .map(|s| {
                    json!({
                        "title": s.title,
                        "level": s.level,
                        "offset": s.offset,
                        "limit": s.limit,
                    })
                })
                .collect();
            json!({
                "path": e.rel_path.display().to_string(),
                "total_lines": e.total_lines,
                "byte_size": e.byte_size,
                "first_heading": e.first_heading,
                "sub_headings": subs,
            })
        })
        .collect();

    let doc = json!({
        "total_files": output.total_files,
        "entries": entries,
    });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

pub fn render_sec(output: &SecOutput) {
    let matches: Vec<Value> = output
        .matches
        .iter()
        .map(|m| {
            json!({
                "title": m.title,
                "offset": m.offset,
                "limit": m.limit,
                "content": m.content,
            })
        })
        .collect();

    let doc = json!({
        "path": output.path.display().to_string(),
        "matches": matches,
    });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

pub fn render_at(output: &AtOutput) {
    let doc = json!({
        "path": output.path.display().to_string(),
        "breadcrumb": output.breadcrumb,
        "section": {
            "title": output.section.title,
            "level": output.section.level,
            "offset": output.section.offset,
            "limit": output.section.limit,
            "features": features_json(&output.section.features),
        },
        "content": output.content,
    });
    println!("{}", serde_json::to_string_pretty(&doc).unwrap());
}

fn features_json(f: &crate::parse::ContentFeatures) -> Value {
    json!({
        "has_table": f.has_table,
        "has_code_block": f.has_code_block,
        "code_languages": f.code_languages,
        "has_list": f.has_list,
        "list_item_count": f.list_item_count,
        "link_count": f.link_count,
        "word_count": f.word_count,
    })
}
