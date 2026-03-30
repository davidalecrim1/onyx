import type { FileTreeEntry } from "../components/FileTree";

export type FileSortOrder =
  | "name-asc"
  | "name-desc"
  | "modified-desc"
  | "modified-asc"
  | "created-desc"
  | "created-asc";

export function sortFileTree(
  entries: FileTreeEntry[],
  order: FileSortOrder,
): FileTreeEntry[] {
  return entries
    .map((entry) => {
      if (!entry.is_directory) return entry;
      return { ...entry, children: sortFileTree(entry.children, order) };
    })
    .sort((a, b) => {
      // Directories always sort before files.
      if (a.is_directory !== b.is_directory) {
        return a.is_directory ? -1 : 1;
      }
      // Within directories, preserve existing order.
      if (a.is_directory) return 0;

      switch (order) {
        case "name-asc":
          return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
        case "name-desc":
          return b.name.toLowerCase().localeCompare(a.name.toLowerCase());
        case "modified-desc":
          return b.modified_secs - a.modified_secs;
        case "modified-asc":
          return a.modified_secs - b.modified_secs;
        case "created-desc":
          return b.created_secs - a.created_secs;
        case "created-asc":
          return a.created_secs - b.created_secs;
      }
    });
}
