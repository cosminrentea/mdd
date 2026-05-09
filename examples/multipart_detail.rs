use pulldown_cmark::{Event, MetadataBlockKind, Options, Parser, Tag, TagEnd};

fn main() {
    let content = std::fs::read_to_string("tests/fixtures/lrn-two-entries.md").unwrap();
    
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    
    let parser = Parser::new_ext(&content, opts);
    let offset_iter = parser.into_offset_iter();
    
    println!("All events with byte offsets:");
    for (event, range) in offset_iter {
        match &event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                println!("  [{}..{}] START MetadataBlock", range.start, range.end);
            }
            Event::Text(t) => {
                let preview: String = t.chars().take(50).collect();
                println!("  [{}..{}] Text: {:?}", range.start, range.end, preview);
            }
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                println!("  [{}..{}] END MetadataBlock", range.start, range.end);
            }
            Event::Start(Tag::Heading { level, .. }) => {
                println!("  [{}..{}] START Heading(H{})", range.start, range.end, *level as u8);
            }
            Event::End(TagEnd::Heading(level)) => {
                println!("  [{}..{}] END Heading(H{})", range.start, range.end, *level as u8);
            }
            Event::Start(Tag::Paragraph) => {
                println!("  [{}..{}] START Paragraph", range.start, range.end);
            }
            _ => {}
        }
    }
}
