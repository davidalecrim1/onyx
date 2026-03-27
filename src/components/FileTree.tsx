import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";

export interface FileTreeEntry {
  name: string;
  path: string;
  is_directory: boolean;
  depth: number;
  children: FileTreeEntry[];
  modified_secs: number;
  created_secs: number;
}

/// Imperative handle exposed to parents so they can programmatically focus the tree.
export interface FileTreeHandle {
  focus(): void;
}

interface ContextMenuState {
  x: number;
  y: number;
  entry: FileTreeEntry;
}

interface Props {
  entries: FileTreeEntry[];
  activeFilePath: string | null;
  vaultPath: string;
  onFileClick: (path: string) => void;
  onFolderClick?: (path: string) => void;
  onFileDrop?: (sourcePath: string, targetDirPath: string) => void;
  onDelete?: (path: string, isDirectory: boolean) => void;
  onCreateFile?: (contextDir: string) => void;
  onCreateFolder?: (contextDir: string) => void;
  onRenameFile?: (path: string, newStem: string) => void;
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
    background: var(--onyx-surface);
    color: var(--onyx-text-primary);
    border: 1px solid var(--onyx-accent);
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

/// Returns the flat ordered list of entries that are visible (respecting collapsed dirs).
function flattenVisible(
  entries: FileTreeEntry[],
  collapsed: Map<string, boolean>,
): FileTreeEntry[] {
  const result: FileTreeEntry[] = [];
  for (const entry of entries) {
    result.push(entry);
    if (
      entry.is_directory &&
      !collapsed.get(entry.path) &&
      entry.children.length > 0
    ) {
      result.push(...flattenVisible(entry.children, collapsed));
    }
  }
  return result;
}

/// Returns the directory that should be used as context for create operations.
/// For files, this is the parent directory. For directories, it is the directory itself.
function contextDirOf(
  focusedPath: string | null,
  entries: FileTreeEntry[],
  vaultPath: string,
  collapsed: Map<string, boolean>,
): string {
  if (!focusedPath) return vaultPath;
  const flat = flattenVisible(entries, collapsed);
  const entry = flat.find((e) => e.path === focusedPath);
  if (!entry) return vaultPath;
  if (entry.is_directory) return entry.path;
  return focusedPath.substring(0, focusedPath.lastIndexOf("/")) || vaultPath;
}

/// Extracts the stem (filename without extension) from a full path.
function stemOf(path: string): string {
  const name = path.split("/").pop() ?? path;
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.substring(0, dot) : name;
}

const FileTree = forwardRef<FileTreeHandle, Props>(function FileTree(
  {
    entries,
    activeFilePath,
    vaultPath,
    onFileClick,
    onFolderClick,
    onFileDrop,
    onDelete,
    onCreateFile,
    onCreateFolder,
    onRenameFile,
  },
  ref,
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [collapsed, setCollapsed] = useState<Map<string, boolean>>(new Map());
  const [focusedPath, setFocusedPath] = useState<string | null>(null);
  const [renamingPath, setRenamingPath] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  useImperativeHandle(ref, () => ({
    focus() {
      containerRef.current?.focus();
    },
  }));

  useEffect(() => {
    if (!contextMenu) return;

    function onKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") setContextMenu(null);
    }

    // Use a timeout so this listener doesn't catch the right-click that opened the menu.
    const timer = setTimeout(() => {
      document.addEventListener("click", () => setContextMenu(null), {
        once: true,
      });
    }, 0);

    document.addEventListener("keydown", onKeyDown);
    return () => {
      clearTimeout(timer);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [contextMenu]);

  function toggleCollapsed(path: string) {
    setCollapsed((prev) => {
      const next = new Map(prev);
      next.set(path, !prev.get(path));
      return next;
    });
  }

  function handleContainerKeyDown(e: React.KeyboardEvent<HTMLDivElement>) {
    // While a rename input is open, let it handle its own keys.
    if (renamingPath !== null) return;

    const flat = flattenVisible(entries, collapsed);
    if (flat.length === 0) return;

    const currentIndex = focusedPath
      ? flat.findIndex((entry) => entry.path === focusedPath)
      : -1;

    switch (e.key) {
      case "ArrowDown": {
        e.preventDefault();
        e.stopPropagation();
        const nextIndex =
          currentIndex === -1 ? 0 : Math.min(currentIndex + 1, flat.length - 1);
        const nextPath = flat[nextIndex]?.path ?? null;
        setFocusedPath(nextPath);
        break;
      }
      case "ArrowUp": {
        e.preventDefault();
        e.stopPropagation();
        if (currentIndex <= 0) {
          setFocusedPath(flat[0]?.path ?? null);
          break;
        }
        setFocusedPath(flat[currentIndex - 1]?.path ?? null);
        break;
      }
      case " ":
      case "Enter": {
        e.preventDefault();
        e.stopPropagation();
        if (!focusedPath) break;
        const focused = flat.find((entry) => entry.path === focusedPath);
        if (!focused) break;
        if (focused.is_directory) {
          toggleCollapsed(focused.path);
          onFolderClick?.(focused.path);
        } else {
          onFileClick(focused.path);
        }
        break;
      }
      case "r": {
        e.preventDefault();
        e.stopPropagation();
        if (!focusedPath) break;
        setRenamingPath(focusedPath);
        setRenameValue(stemOf(focusedPath));
        break;
      }
      case "a": {
        if (e.shiftKey) break;
        e.preventDefault();
        e.stopPropagation();
        onCreateFile?.(
          contextDirOf(focusedPath, entries, vaultPath, collapsed),
        );
        break;
      }
      case "A": {
        e.preventDefault();
        e.stopPropagation();
        onCreateFolder?.(
          contextDirOf(focusedPath, entries, vaultPath, collapsed),
        );
        break;
      }
      case "d": {
        e.preventDefault();
        e.stopPropagation();
        if (!focusedPath) break;
        const focused = flat.find((entry) => entry.path === focusedPath);
        onDelete?.(focusedPath, focused?.is_directory ?? false);
        break;
      }
      case "Escape": {
        e.preventDefault();
        e.stopPropagation();
        setFocusedPath(null);
        containerRef.current?.blur();
        break;
      }
    }
  }

  // Scroll the newly focused item into view without jarring jumps.
  useEffect(() => {
    if (!focusedPath) return;
    const el = containerRef.current?.querySelector(
      `[data-tree-path="${CSS.escape(focusedPath)}"]`,
    );
    if (el && typeof (el as HTMLElement).scrollIntoView === "function") {
      (el as HTMLElement).scrollIntoView({ block: "nearest" });
    }
  }, [focusedPath]);

  return (
    <>
      {/* data-drop-path on the container catches drops onto empty space below all entries,
          which moves the file to the vault root. */}
      <div
        ref={containerRef}
        role="tree"
        tabIndex={0}
        className="select-none text-sm outline-none focus:ring-1 focus:ring-accent/30"
        data-drop-path={vaultPath}
        onKeyDown={handleContainerKeyDown}
        onBlur={() => {
          // Only clear focused path if focus leaves the tree entirely.
          // relatedTarget is the element receiving focus; if it's inside the
          // container we don't want to clear.
          setFocusedPath((prev) => {
            const related = document.activeElement;
            if (containerRef.current?.contains(related)) return prev;
            return null;
          });
        }}
      >
        {entries.map((entry) => (
          <FileTreeNode
            key={entry.path}
            entry={entry}
            activeFilePath={activeFilePath}
            focusedPath={focusedPath}
            renamingPath={renamingPath}
            renameValue={renameValue}
            collapsed={collapsed}
            onFileClick={onFileClick}
            onFolderClick={onFolderClick}
            onFileDrop={onFileDrop}
            onContextMenu={(x, y, entry) => setContextMenu({ x, y, entry })}
            onToggleCollapsed={toggleCollapsed}
            onRenameChange={setRenameValue}
            onRenameConfirm={() => {
              if (renamingPath && onRenameFile) {
                onRenameFile(renamingPath, renameValue);
              }
              setRenamingPath(null);
            }}
            onRenameCancel={() => setRenamingPath(null)}
          />
        ))}
      </div>
      {contextMenu && (
        <div
          className="fixed z-50 min-w-[140px] rounded border border-surface bg-surface-hover py-1 shadow-lg"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button
            className="flex w-full px-3 py-1.5 text-sm text-red-400 hover:bg-surface-active"
            onClick={() => {
              onDelete?.(
                contextMenu.entry.path,
                contextMenu.entry.is_directory,
              );
              setContextMenu(null);
            }}
          >
            Delete
          </button>
        </div>
      )}
    </>
  );
});

export default FileTree;

interface NodeProps {
  entry: FileTreeEntry;
  activeFilePath: string | null;
  focusedPath: string | null;
  renamingPath: string | null;
  renameValue: string;
  collapsed: Map<string, boolean>;
  onFileClick: (path: string) => void;
  onFolderClick?: (path: string) => void;
  onFileDrop?: (sourcePath: string, targetDirPath: string) => void;
  onContextMenu: (x: number, y: number, entry: FileTreeEntry) => void;
  onToggleCollapsed: (path: string) => void;
  onRenameChange: (value: string) => void;
  onRenameConfirm: () => void;
  onRenameCancel: () => void;
}

function FileTreeNode({
  entry,
  activeFilePath,
  focusedPath,
  renamingPath,
  renameValue,
  collapsed,
  onFileClick,
  onFolderClick,
  onFileDrop,
  onContextMenu,
  onToggleCollapsed,
  onRenameChange,
  onRenameConfirm,
  onRenameCancel,
}: NodeProps) {
  const [isDragTarget, setIsDragTarget] = useState(false);
  const indent = entry.depth * 12;
  const isCollapsed = collapsed.get(entry.path) ?? false;
  const isFocused = entry.path === focusedPath;
  const isRenaming = entry.path === renamingPath;

  if (entry.is_directory) {
    return (
      <div data-drop-path={entry.path}>
        <div
          role="treeitem"
          tabIndex={-1}
          data-tree-path={entry.path}
          data-focused={isFocused ? "true" : undefined}
          onClick={() => {
            onToggleCollapsed(entry.path);
            onFolderClick?.(entry.path);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              onToggleCollapsed(entry.path);
              onFolderClick?.(entry.path);
            }
          }}
          onContextMenu={(e) => {
            e.preventDefault();
            onContextMenu(e.clientX, e.clientY, entry);
          }}
          onPointerEnter={() => {
            if (activeDrag) setIsDragTarget(true);
          }}
          onPointerLeave={() => setIsDragTarget(false)}
          className={`flex w-full cursor-pointer items-center gap-1.5 rounded px-2 py-1 text-left text-text-secondary transition-colors hover:bg-surface-hover hover:text-text-primary ${
            isDragTarget ? "bg-surface-hover ring-1 ring-accent" : ""
          } ${isFocused ? "ring-1 ring-accent/60" : ""}`}
          style={{ paddingLeft: `${8 + indent}px` }}
        >
          <span className="text-xs shrink-0">{isCollapsed ? "▶" : "▼"}</span>
          {isRenaming ? (
            <input
              autoFocus
              value={renameValue}
              onChange={(e) => onRenameChange(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.stopPropagation();
                  onRenameConfirm();
                }
                if (e.key === "Escape") {
                  e.stopPropagation();
                  onRenameCancel();
                }
              }}
              onBlur={onRenameCancel}
              className="w-full rounded bg-surface px-1 text-sm text-text-primary outline-none ring-1 ring-accent"
              spellCheck={false}
            />
          ) : (
            <span className="truncate">{entry.name}</span>
          )}
        </div>
        {!isCollapsed && (
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
                focusedPath={focusedPath}
                renamingPath={renamingPath}
                renameValue={renameValue}
                collapsed={collapsed}
                onFileClick={onFileClick}
                onFolderClick={onFolderClick}
                onFileDrop={onFileDrop}
                onContextMenu={onContextMenu}
                onToggleCollapsed={onToggleCollapsed}
                onRenameChange={onRenameChange}
                onRenameConfirm={onRenameConfirm}
                onRenameCancel={onRenameCancel}
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
      role="treeitem"
      tabIndex={-1}
      data-tree-path={entry.path}
      data-focused={isFocused ? "true" : undefined}
      onPointerDown={handlePointerDown}
      onClick={() => onFileClick(entry.path)}
      onContextMenu={(e) => {
        e.preventDefault();
        onContextMenu(e.clientX, e.clientY, entry);
      }}
      className={`flex w-full items-center rounded px-2 py-1 text-left transition-colors ${
        isActive
          ? "bg-surface-active text-text-primary"
          : "text-text-secondary hover:bg-surface-hover hover:text-text-primary"
      } ${isFocused ? "ring-1 ring-accent/60" : ""}`}
      style={{ paddingLeft: `${20 + indent}px` }}
    >
      {isRenaming ? (
        <input
          autoFocus
          value={renameValue}
          onChange={(e) => onRenameChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.stopPropagation();
              onRenameConfirm();
            }
            if (e.key === "Escape") {
              e.stopPropagation();
              onRenameCancel();
            }
          }}
          onBlur={onRenameCancel}
          className="w-full rounded bg-surface px-1 text-sm text-text-primary outline-none ring-1 ring-accent"
          spellCheck={false}
        />
      ) : (
        <span className="truncate">{entry.name.replace(/\.md$/i, "")}</span>
      )}
    </button>
  );
}
