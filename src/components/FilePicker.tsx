import { useEffect, useMemo, useRef, useState } from "react";
import type { FileTreeEntry } from "./FileTree";

interface FlatFile {
  name: string;
  path: string;
  dir: string;
}

function flattenTree(entries: FileTreeEntry[]): FlatFile[] {
  const result: FlatFile[] = [];
  for (const entry of entries) {
    if (entry.is_directory) {
      result.push(...flattenTree(entry.children));
    } else {
      const lastSlash = entry.path.lastIndexOf("/");
      const dir = lastSlash > 0 ? entry.path.slice(0, lastSlash) : "";
      result.push({ name: entry.name, path: entry.path, dir });
    }
  }
  return result;
}

interface Props {
  files: FileTreeEntry[];
  onOpen: (path: string) => void;
  onCreate: (name: string) => void;
  onClose: () => void;
}

export default function FilePicker({
  files,
  onOpen,
  onCreate,
  onClose,
}: Props) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef<HTMLUListElement>(null);

  const allFiles = useMemo(() => flattenTree(files), [files]);

  const filteredFiles = useMemo(() => {
    if (!query) return allFiles;
    const lower = query.toLowerCase();
    return allFiles.filter((file) => file.name.toLowerCase().includes(lower));
  }, [allFiles, query]);

  const showCreate = filteredFiles.length === 0 && query.trim().length > 0;

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    const item = listRef.current?.children[selectedIndex] as
      | HTMLElement
      | undefined;
    item?.scrollIntoView?.({ block: "nearest" });
  }, [selectedIndex]);

  function handleSelect(index: number) {
    if (showCreate) {
      onClose();
      onCreate(query.trim());
      return;
    }
    const file = filteredFiles[index];
    if (!file) return;
    onClose();
    onOpen(file.path);
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (!showCreate) {
        setSelectedIndex((i) => Math.min(i + 1, filteredFiles.length - 1));
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (!showCreate) {
        setSelectedIndex((i) => Math.max(i - 1, 0));
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      handleSelect(selectedIndex);
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-32"
      onMouseDown={onClose}
    >
      <div
        className="w-full max-w-lg rounded-lg bg-surface shadow-xl ring-1 ring-surface-hover"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Open file…"
          autoFocus
          className="w-full border-b border-surface-hover bg-transparent px-4 py-3 text-sm text-text-primary outline-none placeholder:text-text-secondary"
          spellCheck={false}
        />
        <ul ref={listRef} className="max-h-80 overflow-y-auto py-1">
          {showCreate ? (
            <li
              onMouseDown={(e) => {
                e.preventDefault();
                handleSelect(0);
              }}
              className="flex cursor-pointer items-center justify-between bg-surface-hover px-4 py-2 text-sm text-text-primary"
            >
              <span>{query.trim()}</span>
              <span className="ml-4 shrink-0 text-xs text-text-secondary opacity-70">
                Enter to create
              </span>
            </li>
          ) : (
            filteredFiles.map((file, index) => {
              const isSelected = index === selectedIndex;
              return (
                <li
                  key={file.path}
                  onMouseDown={(e) => {
                    e.preventDefault();
                    handleSelect(index);
                  }}
                  onMouseEnter={() => setSelectedIndex(index)}
                  className={`flex cursor-pointer items-center justify-between px-4 py-2 text-sm ${
                    isSelected
                      ? "bg-surface-hover text-text-primary"
                      : "text-text-secondary"
                  }`}
                >
                  <span className="truncate">
                    {file.name.replace(/\.md$/i, "")}
                  </span>
                  {file.dir && (
                    <span className="ml-4 shrink-0 max-w-[40%] truncate text-xs text-text-secondary opacity-70 text-right">
                      {file.dir.split("/").pop()}
                    </span>
                  )}
                </li>
              );
            })
          )}
          {!showCreate && filteredFiles.length === 0 && (
            <li className="px-4 py-3 text-sm text-text-secondary">
              No files found
            </li>
          )}
        </ul>
      </div>
    </div>
  );
}
