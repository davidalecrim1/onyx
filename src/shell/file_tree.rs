use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct FileTree {
    root: PathBuf,
}

impl FileTree {
    pub fn new(root: &Path) -> Self {
        FileTree { root: root.to_path_buf() }
    }

    /// Returns all .md files and directories in the vault root, sorted by path.
    pub fn entries(&self) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        self.collect_entries(&self.root, &mut entries);
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        entries
    }

    fn collect_entries(&self, dir: &Path, out: &mut Vec<FileEntry>) {
        let Ok(read_dir) = std::fs::read_dir(dir) else { return };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                if name.starts_with('.') { continue; }
                out.push(FileEntry { path: self.relative(&path), is_dir: true });
                self.collect_entries(&path, out);
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                out.push(FileEntry { path: self.relative(&path), is_dir: false });
            }
        }
    }

    fn relative(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.root).unwrap_or(path).to_path_buf()
    }

    /// Creates an empty file at `name` relative to the vault root.
    pub fn create_file(&self, name: &str) -> std::io::Result<()> {
        std::fs::write(self.root.join(name), "")
    }

    /// Deletes a file at `name` relative to the vault root.
    pub fn delete_file(&self, name: &str) -> std::io::Result<()> {
        std::fs::remove_file(self.root.join(name))
    }

    /// Renames a file within the vault root.
    pub fn rename_file(&self, from: &str, to: &str) -> std::io::Result<()> {
        std::fs::rename(self.root.join(from), self.root.join(to))
    }

    /// Moves a file to a different path within the vault root.
    pub fn move_file(&self, from: &str, to: &str) -> std::io::Result<()> {
        std::fs::rename(self.root.join(from), self.root.join(to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_file_creation() {
        let dir = tempfile::tempdir().unwrap();
        let tree = FileTree::new(dir.path());
        tree.create_file("notes.md").unwrap();
        assert!(dir.path().join("notes.md").exists());
    }

    #[test]
    fn delete_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.md"), "").unwrap();
        let tree = FileTree::new(dir.path());
        tree.delete_file("old.md").unwrap();
        assert!(!dir.path().join("old.md").exists());
    }

    #[test]
    fn rename_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "").unwrap();
        let tree = FileTree::new(dir.path());
        tree.rename_file("a.md", "b.md").unwrap();
        assert!(dir.path().join("b.md").exists());
        assert!(!dir.path().join("a.md").exists());
    }
}
