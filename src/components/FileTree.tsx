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
  vaultPath: string;
  onFileClick: (path: string) => void;
  onFolderClick?: (path: string) => void;
  onFileDrop?: (sourcePath: string, targetDirPath: string) => void;
}

// Shared drag state lifted outside React so all nodes can read it without prop drilling.
// This is module-level, not component state, so it never triggers re-renders.
interface DragState {
  sourcePath: string;
  ghost: HTMLDivElement;
}

let activeDrag: DragState | null = null;

function createGhost(label: string): HTMLDivElement {
  const ghost = document.createElement("div");
  ghost.textContent = label;
  ghost.style.cssText = `
    position: fixed;
    pointer-events: none;
    z-index: 9999;
    background: var(--color-surface, #2a2a2a);
    color: var(--color-text-primary, #e0e0e0);
    border: 1px solid var(--color-accent, #74ade8);
    border-radius: 4px;
    padding: 2px 8px;
    font-size: 12px;
    white-space: nowrap;
    opacity: 0.9;
    transform: translate(-50%, -120%);
  `;
  document.body.appendChild(ghost);
  return ghost;
}

export default function FileTree({
  entries,
  activeFilePath,
  vaultPath,
  onFileClick,
  onFolderClick,
  onFileDrop,
}: Props) {
  return (
    // data-drop-path on the container catches drops onto empty space below all entries,
    // which moves the file to the vault root.
    <div className="select-none text-sm" data-drop-path={vaultPath}>
      {entries.map((entry) => (
        <FileTreeNode
          key={entry.path}
          entry={entry}
          activeFilePath={activeFilePath}
          onFileClick={onFileClick}
          onFolderClick={onFolderClick}
          onFileDrop={onFileDrop}
        />
      ))}
    </div>
  );
}

interface NodeProps {
  entry: FileTreeEntry;
  activeFilePath: string | null;
  onFileClick: (path: string) => void;
  onFolderClick?: (path: string) => void;
  onFileDrop?: (sourcePath: string, targetDirPath: string) => void;
}

function FileTreeNode({
  entry,
  activeFilePath,
  onFileClick,
  onFolderClick,
  onFileDrop,
}: NodeProps) {
  const [collapsed, setCollapsed] = useState(false);
  const [isDragTarget, setIsDragTarget] = useState(false);
  const indent = entry.depth * 12;

  if (entry.is_directory) {
    return (
      <div data-drop-path={entry.path}>
        <div
          role="button"
          tabIndex={0}
          onClick={() => {
            setCollapsed((prev) => !prev);
            onFolderClick?.(entry.path);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              setCollapsed((prev) => !prev);
              onFolderClick?.(entry.path);
            }
          }}
          onPointerEnter={() => {
            if (activeDrag) setIsDragTarget(true);
          }}
          onPointerLeave={() => setIsDragTarget(false)}
          className={`flex w-full cursor-pointer items-center gap-1.5 rounded px-2 py-1 text-left text-text-secondary transition-colors hover:bg-surface-hover hover:text-text-primary ${
            isDragTarget ? "bg-surface-hover ring-1 ring-accent" : ""
          }`}
          style={{ paddingLeft: `${8 + indent}px` }}
        >
          <span className="text-xs shrink-0">{collapsed ? "▶" : "▼"}</span>
          <span className="truncate">{entry.name}</span>
        </div>
        {!collapsed && (
          <div className="relative">
            <div
              className="absolute top-0 bottom-0 w-px"
              style={{ left: `${12 + indent}px`, backgroundColor: "#454a56" }}
            />
            {entry.children.map((child) => (
              <FileTreeNode
                key={child.path}
                entry={child}
                activeFilePath={activeFilePath}
                onFileClick={onFileClick}
                onFolderClick={onFolderClick}
                onFileDrop={onFileDrop}
              />
            ))}
          </div>
        )}
      </div>
    );
  }

  const isActive = entry.path === activeFilePath;

  function handlePointerDown(e: React.PointerEvent<HTMLButtonElement>) {
    // Only initiate drag on primary button (left click).
    if (e.button !== 0) return;

    const startX = e.clientX;
    const startY = e.clientY;
    let dragStarted = false;

    const target = e.currentTarget;
    target.setPointerCapture(e.pointerId);

    function onPointerMove(moveEvent: PointerEvent) {
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;

      if (!dragStarted && Math.hypot(dx, dy) > 6) {
        dragStarted = true;
        activeDrag = {
          sourcePath: entry.path,
          ghost: createGhost(entry.name),
        };
      }

      if (dragStarted && activeDrag) {
        activeDrag.ghost.style.left = `${moveEvent.clientX}px`;
        activeDrag.ghost.style.top = `${moveEvent.clientY}px`;
      }
    }

    function onPointerUp(upEvent: PointerEvent) {
      target.removeEventListener("pointermove", onPointerMove);
      target.removeEventListener("pointerup", onPointerUp);
      target.releasePointerCapture(upEvent.pointerId);

      if (!dragStarted || !activeDrag) {
        activeDrag = null;
        return;
      }

      activeDrag.ghost.remove();
      const sourcePath = activeDrag.sourcePath;
      activeDrag = null;

      // Hit-test the element under the pointer to find the drop target.
      const el = document.elementFromPoint(upEvent.clientX, upEvent.clientY);
      const dropTarget = el?.closest("[data-drop-path]");
      const targetPath = dropTarget?.getAttribute("data-drop-path");

      if (targetPath && targetPath !== sourcePath && onFileDrop) {
        onFileDrop(sourcePath, targetPath);
      }
    }

    target.addEventListener("pointermove", onPointerMove);
    target.addEventListener("pointerup", onPointerUp);
  }

  return (
    <button
      onPointerDown={handlePointerDown}
      onClick={() => onFileClick(entry.path)}
      className={`flex w-full items-center rounded px-2 py-1 text-left transition-colors ${
        isActive
          ? "bg-surface-active text-text-primary"
          : "text-text-secondary hover:bg-surface-hover hover:text-text-primary"
      }`}
      style={{ paddingLeft: `${20 + indent}px` }}
    >
      <span className="truncate">{entry.name}</span>
    </button>
  );
}
