use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::error::OnyxError;

/// Maps each file path to the set of tags found in that file, enabling O(1) incremental updates on save.
pub struct TagIndex {
    file_tags: HashMap<String, HashSet<String>>,
}

impl TagIndex {
    /// Walks all `.md` files under `vault_root` and builds the initial index.
    pub fn build(vault_root: &Path) -> Result<Self, OnyxError> {
        let mut file_tags: HashMap<String, HashSet<String>> = HashMap::new();

        for entry in walkdir::WalkDir::new(vault_root)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(path)?;
            let tags = extract_tags(&content);
            if !tags.is_empty() {
                file_tags.insert(path.to_string_lossy().to_string(), tags);
            }
        }

        Ok(Self { file_tags })
    }

    /// Replaces the tag set for a single file; called after every save so no full re-scan is needed.
    pub fn update_file(&mut self, path: &str, content: &str) {
        let tags = extract_tags(content);
        if tags.is_empty() {
            self.file_tags.remove(path);
        } else {
            self.file_tags.insert(path.to_string(), tags);
        }
    }

    /// Returns a sorted, deduplicated list of every tag across all indexed files.
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: HashSet<&str> = HashSet::new();
        for tag_set in self.file_tags.values() {
            for tag in tag_set {
                tags.insert(tag.as_str());
            }
        }
        let mut result: Vec<String> = tags.into_iter().map(|t| t.to_string()).collect();
        result.sort();
        result
    }
}

/// Scans `content` for tokens matching `#[a-zA-Z][a-zA-Z0-9_-]*`.
/// The leading `#` is excluded from the returned tag strings.
pub fn extract_tags(content: &str) -> HashSet<String> {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut tags = HashSet::new();
    let mut index = 0;

    while index < len {
        if chars[index] != '#' {
            index += 1;
            continue;
        }

        // Must be preceded by whitespace or start-of-content to avoid matching mid-word #.
        let preceded_by_whitespace = index == 0 || chars[index - 1].is_whitespace();
        if !preceded_by_whitespace {
            index += 1;
            continue;
        }

        // First character after # must be a letter.
        let start = index + 1;
        if start >= len || !chars[start].is_ascii_alphabetic() {
            index += 1;
            continue;
        }

        let mut end = start + 1;
        while end < len
            && (chars[end].is_ascii_alphanumeric() || chars[end] == '_' || chars[end] == '-')
        {
            end += 1;
        }

        let tag: String = chars[start..end].iter().collect();
        tags.insert(tag);
        index = end;
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_tags_basic() {
        let tags = extract_tags("Hello #world and #foo-bar");
        assert!(tags.contains("world"));
        assert!(tags.contains("foo-bar"));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn extract_tags_ignores_headings() {
        // Markdown headings start at the beginning of a line; they are still
        // preceded by a newline (whitespace), so the rule alone would match.
        // The `#` in `# Heading` is followed by a space, not a letter, so it
        // must not be treated as a tag.
        let tags = extract_tags("# Heading\n## Another");
        assert!(tags.is_empty(), "headings should not be extracted as tags");
    }

    #[test]
    fn extract_tags_ignores_mid_word_hash() {
        let tags = extract_tags("color:#ff0000");
        assert!(tags.is_empty());
    }

    #[test]
    fn extract_tags_deduplicates() {
        let tags = extract_tags("#rust #rust #go");
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn extract_tags_at_start_of_content() {
        let tags = extract_tags("#start");
        assert!(tags.contains("start"));
    }

    #[test]
    fn tag_index_update_file() {
        let mut index = TagIndex {
            file_tags: HashMap::new(),
        };
        index.update_file("/tmp/a.md", "Hello #rust #go");
        assert!(index.all_tags().contains(&"rust".to_string()));

        index.update_file("/tmp/a.md", "Now only #python");
        let tags = index.all_tags();
        assert!(tags.contains(&"python".to_string()));
        assert!(!tags.contains(&"rust".to_string()));
    }

    #[test]
    fn tag_index_all_tags_sorted() {
        let mut index = TagIndex {
            file_tags: HashMap::new(),
        };
        index.update_file("/tmp/a.md", "#zebra #apple");
        index.update_file("/tmp/b.md", "#mango");
        let tags = index.all_tags();
        assert_eq!(tags, vec!["apple", "mango", "zebra"]);
    }
}
