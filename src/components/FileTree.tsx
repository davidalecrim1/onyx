import { useState } from "react";

export interface FileTreeEntry {
  name: string;
  path: string;
  is_directory: boolean;
  depth: number;
  children: FileTreeEntry[];
}

interface Props {
  entries: FileTreeEntry[];
  activeFilePath: string | null;
  onFileClick: (path: string) => void;
}

export default function FileTree({
  entries,
  activeFilePath,
  onFileClick,
}: Props) {
  return (
    <div className="select-none text-sm">
      {entries.map((entry) => (
        <FileTreeNode
          key={entry.path}
          entry={entry}
          activeFilePath={activeFilePath}
          onFileClick={onFileClick}
        />
      ))}
    </div>
  );
}

interface NodeProps {
  entry: FileTreeEntry;
  activeFilePath: string | null;
  onFileClick: (path: string) => void;
}

function FileTreeNode({ entry, activeFilePath, onFileClick }: NodeProps) {
  const [collapsed, setCollapsed] = useState(false);
  const indent = entry.depth * 12;

  if (entry.is_directory) {
    return (
      <div>
        <button
          onClick={() => setCollapsed((prev) => !prev)}
          className="flex w-full items-center gap-1.5 rounded px-2 py-0.5 text-left text-text-secondary transition-colors hover:bg-surface-hover hover:text-text-primary"
          style={{ paddingLeft: `${8 + indent}px` }}
        >
          <span className="text-xs">{collapsed ? "▶" : "▼"}</span>
          {entry.name}
        </button>
        {!collapsed && (
          <div>
            {entry.children.map((child) => (
              <FileTreeNode
                key={child.path}
                entry={child}
                activeFilePath={activeFilePath}
                onFileClick={onFileClick}
              />
            ))}
          </div>
        )}
      </div>
    );
  }

  const isActive = entry.path === activeFilePath;
  return (
    <button
      onClick={() => onFileClick(entry.path)}
      className={`flex w-full items-center rounded px-2 py-0.5 text-left transition-colors ${
        isActive
          ? "bg-surface-active text-text-primary"
          : "text-text-secondary hover:bg-surface-hover hover:text-text-primary"
      }`}
      style={{ paddingLeft: `${20 + indent}px` }}
    >
      {entry.name}
    </button>
  );
}
