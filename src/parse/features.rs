// ContentFeatures detection from pulldown-cmark events.
//
// During the event stream, we accumulate a ContentFeatures struct per section.
// This is a "builder" pattern: create a mutable FeatureAccumulator, feed it
// events, then finalize into an immutable ContentFeatures.

use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};

use super::ContentFeatures;

/// Mutable accumulator that collects feature signals from parser events.
///
/// Rust concept: separating the mutable builder from the immutable result type.
/// The builder lives only during parsing; once a section ends, we call `.finish()`
/// to produce the final ContentFeatures value.
#[derive(Debug, Default, Clone)]
pub struct FeatureAccumulator {
    pub has_table: bool,
    pub has_code_block: bool,
    pub code_languages: Vec<String>,
    pub has_list: bool,
    pub list_item_count: u16,
    pub link_count: u16,
    pub word_count: u32,
    // Internal tracking state
    in_list: bool,
}

impl FeatureAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a single pulldown-cmark event, updating feature counts.
    ///
    /// Call this for every event that falls within the current section's range.
    pub fn process_event(&mut self, event: &Event) {
        match event {
            // Tables
            Event::Start(Tag::Table(_)) => {
                self.has_table = true;
            }

            // Code blocks -- extract language from fenced blocks
            Event::Start(Tag::CodeBlock(kind)) => {
                self.has_code_block = true;
                if let CodeBlockKind::Fenced(lang) = kind {
                    let lang_str = lang.as_ref().trim();
                    if !lang_str.is_empty() && !self.code_languages.contains(&lang_str.to_owned()) {
                        self.code_languages.push(lang_str.to_owned());
                    }
                }
            }

            // Lists
            Event::Start(Tag::List(_)) => {
                self.has_list = true;
                self.in_list = true;
            }
            Event::End(TagEnd::List(_)) => {
                self.in_list = false;
            }
            Event::Start(Tag::Item) if self.in_list => {
                self.list_item_count = self.list_item_count.saturating_add(1);
            }

            // Links (all kinds)
            Event::Start(Tag::Link { .. }) => {
                self.link_count = self.link_count.saturating_add(1);
            }

            // Text -- count words
            Event::Text(text) => {
                let count = text.split_whitespace().count() as u32;
                self.word_count = self.word_count.saturating_add(count);
            }

            // Code text inside inline code also contributes to word count
            Event::Code(text) => {
                let count = text.split_whitespace().count() as u32;
                self.word_count = self.word_count.saturating_add(count);
            }

            _ => {}
        }
    }

    /// Convert the accumulated state into an immutable ContentFeatures.
    pub fn finish(self) -> ContentFeatures {
        ContentFeatures {
            has_table: self.has_table,
            has_code_block: self.has_code_block,
            code_languages: self.code_languages,
            has_list: self.has_list,
            list_item_count: self.list_item_count,
            link_count: self.link_count,
            word_count: self.word_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use pulldown_cmark::{Options, Parser};

    use super::*;

    fn features_from(md: &str) -> ContentFeatures {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TASKLISTS);

        let parser = Parser::new_ext(md, opts);
        let mut acc = FeatureAccumulator::new();
        for event in parser {
            acc.process_event(&event);
        }
        acc.finish()
    }

    #[test]
    fn detects_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let f = features_from(md);
        assert!(f.has_table);
        assert!(!f.has_code_block);
    }

    #[test]
    fn detects_fenced_code_with_language() {
        let md = "```rust\nfn main() {}\n```\n";
        let f = features_from(md);
        assert!(f.has_code_block);
        assert_eq!(f.code_languages, vec!["rust"]);
    }

    #[test]
    fn detects_list_items() {
        let md = "- one\n- two\n- three\n";
        let f = features_from(md);
        assert!(f.has_list);
        assert_eq!(f.list_item_count, 3);
    }

    #[test]
    fn counts_words() {
        let md = "Hello world, this is a test.\n\nSecond paragraph here.\n";
        let f = features_from(md);
        // "Hello world, this is a test." = 6 words, "Second paragraph here." = 3 words
        assert_eq!(f.word_count, 9);
    }

    #[test]
    fn counts_links() {
        let md = "See [foo](http://foo.com) and [bar](http://bar.com).\n";
        let f = features_from(md);
        assert_eq!(f.link_count, 2);
    }

    #[test]
    fn multiple_code_languages() {
        let md = "```rust\ncode\n```\n\n```python\ncode\n```\n\n```rust\nmore\n```\n";
        let f = features_from(md);
        assert_eq!(f.code_languages, vec!["rust", "python"]);
    }

    #[test]
    fn empty_content() {
        let f = features_from("");
        assert!(!f.has_table);
        assert!(!f.has_code_block);
        assert!(!f.has_list);
        assert_eq!(f.word_count, 0);
    }

    #[test]
    fn list_items_not_counted_outside_list() {
        let md = "- one\n- two\n\nNot a list item.\n\n- three\n";
        let f = features_from(md);
        assert!(f.has_list);
        assert_eq!(f.list_item_count, 3);
    }

    #[test]
    fn consecutive_lists_counted_separately() {
        let md = "- a\n- b\n\nParagraph.\n\n- c\n- d\n- e\n";
        let f = features_from(md);
        assert!(f.has_list);
        assert_eq!(f.list_item_count, 5);
    }
}
