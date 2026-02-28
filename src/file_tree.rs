use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::OnyxError;

/// A single entry (file or directory) in the vault's file tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileTreeEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
    pub depth: usize,
    pub children: Vec<FileTreeEntry>,
}

const ACCEPTED_EXTENSIONS: &[&str] = &[
    // Documents
    ".md", ".canvas", ".pdf", // Images
    ".avif", ".bmp", ".gif", ".jpeg", ".jpg", ".png", ".svg", ".webp", // Audio
    ".flac", ".m4a", ".mp3", ".ogg", ".wav", ".3gp", // Video
    ".mkv", ".mov", ".mp4", ".ogv", ".webm",
];

/// Whether a filename has a recognized extension for the file tree.
fn is_accepted_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    ACCEPTED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Recursively scans `root` for recognized file types, excluding dot-directories, sorted dirs-first.
pub fn scan_file_tree(root: &Path) -> Result<Vec<FileTreeEntry>, OnyxError> {
    scan_recursive(root, 0)
}

fn scan_recursive(directory: &Path, depth: usize) -> Result<Vec<FileTreeEntry>, OnyxError> {
    let mut entries = Vec::new();

    let mut dir_entries: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .collect();

    dir_entries.sort_by_key(|entry| entry.file_name());

    for entry in dir_entries {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        let file_type = entry.file_type()?;

        if name.starts_with('.') {
            continue;
        }

        if file_type.is_dir() {
            let children = scan_recursive(&path, depth + 1)?;
            if !children.is_empty() {
                entries.push(FileTreeEntry {
                    name,
                    path,
                    is_directory: true,
                    depth,
                    children,
                });
            }
        } else if is_accepted_file(&name) {
            entries.push(FileTreeEntry {
                name,
                path,
                is_directory: false,
                depth,
                children: Vec::new(),
            });
        }
    }

    // Sort: directories first, then alphabetically within each group
    entries.sort_by(|a, b| {
        b.is_directory
            .cmp(&a.is_directory)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

/// Flattens a nested file tree into a depth-ordered list for rendering.
#[cfg(test)]
pub fn flatten_tree(entries: &[FileTreeEntry]) -> Vec<&FileTreeEntry> {
    let mut result = Vec::new();
    for entry in entries {
        result.push(entry);
        if entry.is_directory {
            result.extend(flatten_tree(&entry.children));
        }
    }
    result
}

/// Flattens the tree but skips children of directories in `collapsed`.
pub fn flatten_tree_filtered<'a>(
    entries: &'a [FileTreeEntry],
    collapsed: &HashSet<PathBuf>,
) -> Vec<&'a FileTreeEntry> {
    let mut result = Vec::new();
    for entry in entries {
        result.push(entry);
        if entry.is_directory && !collapsed.contains(&entry.path) {
            result.extend(flatten_tree_filtered(&entry.children, collapsed));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_vault() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        std::fs::create_dir_all(root.join(".onyx")).unwrap();
        std::fs::create_dir_all(root.join("notes")).unwrap();
        std::fs::create_dir_all(root.join("media")).unwrap();
        std::fs::write(root.join("readme.md"), "# Readme").unwrap();
        std::fs::write(root.join("notes/todo.md"), "# Todo").unwrap();
        std::fs::write(root.join("notes/ideas.md"), "# Ideas").unwrap();
        std::fs::write(root.join("media/photo.png"), "png data").unwrap();
        std::fs::write(root.join("media/doc.pdf"), "pdf data").unwrap();
        std::fs::write(root.join("ignored.txt"), "not recognized").unwrap();

        temp
    }

    #[test]
    fn scan_finds_recognized_files() {
        let temp = setup_vault();
        let entries = scan_file_tree(temp.path()).unwrap();
        let flat = flatten_tree(&entries);
        let names: Vec<&str> = flat.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"readme.md"));
        assert!(names.contains(&"todo.md"));
        assert!(names.contains(&"ideas.md"));
        assert!(names.contains(&"photo.png"));
        assert!(names.contains(&"doc.pdf"));
    }

    #[test]
    fn scan_ignores_unrecognized_files() {
        let temp = setup_vault();
        let entries = scan_file_tree(temp.path()).unwrap();
        let flat = flatten_tree(&entries);
        let names: Vec<&str> = flat.iter().map(|e| e.name.as_str()).collect();

        assert!(!names.contains(&"ignored.txt"));
    }

    #[test]
    fn scan_excludes_dot_directories() {
        let temp = setup_vault();
        let entries = scan_file_tree(temp.path()).unwrap();
        let flat = flatten_tree(&entries);
        let names: Vec<&str> = flat.iter().map(|e| e.name.as_str()).collect();

        assert!(!names.contains(&".onyx"));
    }

    #[test]
    fn flatten_filtered_skips_collapsed_children() {
        let temp = setup_vault();
        let entries = scan_file_tree(temp.path()).unwrap();
        let notes_path = temp.path().join("notes");
        let mut collapsed = HashSet::new();
        collapsed.insert(notes_path);

        let flat = flatten_tree_filtered(&entries, &collapsed);
        let names: Vec<&str> = flat.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"notes"));
        assert!(!names.contains(&"todo.md"));
        assert!(!names.contains(&"ideas.md"));
        assert!(names.contains(&"readme.md"));
    }

    #[test]
    fn flatten_filtered_empty_collapsed_matches_flatten() {
        let temp = setup_vault();
        let entries = scan_file_tree(temp.path()).unwrap();
        let collapsed = HashSet::new();

        let flat = flatten_tree(&entries);
        let filtered = flatten_tree_filtered(&entries, &collapsed);

        let names: Vec<&str> = flat.iter().map(|e| e.name.as_str()).collect();
        let filtered_names: Vec<&str> = filtered.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, filtered_names);
    }

    #[test]
    fn scan_sorts_dirs_before_files() {
        let temp = setup_vault();
        let entries = scan_file_tree(temp.path()).unwrap();

        let last_dir_idx = entries.iter().rposition(|e| e.is_directory).unwrap();
        let first_file_idx = entries.iter().position(|e| !e.is_directory).unwrap();
        assert!(last_dir_idx < first_file_idx);
    }
}
