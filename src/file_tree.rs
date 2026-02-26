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

/// Recursively scans `root` for `.md` files, excluding `.onyx/`, sorted dirs-first.
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
        } else if name.ends_with(".md") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_vault() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        std::fs::create_dir_all(root.join(".onyx")).unwrap();
        std::fs::create_dir_all(root.join("notes")).unwrap();
        std::fs::write(root.join("readme.md"), "# Readme").unwrap();
        std::fs::write(root.join("notes/todo.md"), "# Todo").unwrap();
        std::fs::write(root.join("notes/ideas.md"), "# Ideas").unwrap();
        std::fs::write(root.join("ignored.txt"), "not markdown").unwrap();

        temp
    }

    #[test]
    fn scan_finds_md_files() {
        let temp = setup_vault();
        let entries = scan_file_tree(temp.path()).unwrap();
        let flat = flatten_tree(&entries);
        let names: Vec<&str> = flat.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"readme.md"));
        assert!(names.contains(&"todo.md"));
        assert!(names.contains(&"ideas.md"));
    }

    #[test]
    fn scan_ignores_non_md() {
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
    fn scan_sorts_dirs_before_files() {
        let temp = setup_vault();
        let entries = scan_file_tree(temp.path()).unwrap();

        assert!(entries[0].is_directory);
        assert!(!entries[1].is_directory);
    }
}
