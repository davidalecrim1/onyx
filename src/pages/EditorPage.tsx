import { useCallback, useEffect, useReducer, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FileTree, { type FileTreeEntry } from "../components/FileTree";
import TabBar, { type Tab } from "../components/TabBar";
import MarkdownEditor from "../components/MarkdownEditor";

interface Props {
  vaultPath: string;
  vaultName: string;
  onClose: () => void;
}

interface EditorState {
  tabs: Tab[];
  activeTabPath: string | null;
  fileContents: Record<string, string>;
}

type EditorAction =
  | { type: "open_file"; path: string; name: string; content: string }
  | { type: "close_tab"; path: string }
  | { type: "activate_tab"; path: string }
  | { type: "update_content"; path: string; content: string };

function editorReducer(state: EditorState, action: EditorAction): EditorState {
  switch (action.type) {
    case "open_file": {
      if (state.tabs.some((tab) => tab.path === action.path)) {
        return { ...state, activeTabPath: action.path };
      }
      return {
        ...state,
        tabs: [...state.tabs, { path: action.path, name: action.name }],
        activeTabPath: action.path,
        fileContents: { ...state.fileContents, [action.path]: action.content },
      };
    }
    case "close_tab": {
      const remaining = state.tabs.filter((tab) => tab.path !== action.path);
      const newActive =
        state.activeTabPath === action.path
          ? (remaining[remaining.length - 1]?.path ?? null)
          : state.activeTabPath;
      const { [action.path]: _removed, ...rest } = state.fileContents;
      return { tabs: remaining, activeTabPath: newActive, fileContents: rest };
    }
    case "activate_tab":
      return { ...state, activeTabPath: action.path };
    case "update_content":
      return {
        ...state,
        fileContents: { ...state.fileContents, [action.path]: action.content },
      };
  }
}

export default function EditorPage({ vaultPath, vaultName, onClose }: Props) {
  const [fileTree, setFileTree] = useState<FileTreeEntry[]>([]);
  const [treeError, setTreeError] = useState<string | null>(null);
  const [newNoteName, setNewNoteName] = useState<string | null>(null);
  const [state, dispatch] = useReducer(editorReducer, {
    tabs: [],
    activeTabPath: null,
    fileContents: {},
  });

  // Kept as a ref so the keyboard handler never needs to re-register when content changes.
  const saveRef = useRef<() => Promise<void>>(async () => {});

  const fetchFileTree = useCallback(() => {
    invoke<FileTreeEntry[]>("get_file_tree", { vaultPath })
      .then(setFileTree)
      .catch((err) => setTreeError(String(err)));
  }, [vaultPath]);

  useEffect(() => {
    fetchFileTree();
  }, [fetchFileTree]);

  const handleNewNoteOpen = useCallback(() => {
    setNewNoteName("Untitled.md");
  }, []);

  const newNoteInputCallbackRef = useCallback((el: HTMLInputElement | null) => {
    if (el) el.select();
  }, []);

  const handleNewNoteConfirm = useCallback(async () => {
    if (newNoteName === null) return;
    const name = newNoteName.trim() || "Untitled.md";
    const finalName = name.endsWith(".md") ? name : `${name}.md`;
    setNewNoteName(null);
    try {
      const filePath = await invoke<string>("create_file", {
        vaultPath,
        name: finalName,
      });
      fetchFileTree();
      dispatch({
        type: "open_file",
        path: filePath,
        name: finalName,
        content: "",
      });
    } catch (err) {
      console.error("Failed to create file:", err);
    }
  }, [newNoteName, vaultPath, fetchFileTree]);

  const handleFileClick = useCallback(
    async (path: string) => {
      if (state.tabs.some((tab) => tab.path === path)) {
        dispatch({ type: "activate_tab", path });
        return;
      }
      try {
        const content = await invoke<string>("read_file", { path });
        const name = path.split("/").pop() ?? path;
        dispatch({ type: "open_file", path, name, content });
      } catch (err) {
        console.error("Failed to read file:", err);
      }
    },
    [state.tabs],
  );

  const handleContentChange = useCallback(
    (content: string) => {
      if (!state.activeTabPath) return;
      dispatch({ type: "update_content", path: state.activeTabPath, content });
    },
    [state.activeTabPath],
  );

  const handleTabClick = useCallback(
    (path: string) => dispatch({ type: "activate_tab", path }),
    [],
  );

  const handleTabClose = useCallback(
    (path: string) => dispatch({ type: "close_tab", path }),
    [],
  );

  saveRef.current = async () => {
    if (!state.activeTabPath) return;
    const content = state.fileContents[state.activeTabPath];
    if (content === undefined) return;
    try {
      await invoke("write_file", { path: state.activeTabPath, content });
    } catch (err) {
      console.error("Failed to save file:", err);
    }
  };

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        saveRef.current();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const activeContent =
    state.activeTabPath !== null
      ? (state.fileContents[state.activeTabPath] ?? "")
      : null;

  return (
    <div className="flex h-full bg-background text-text-primary">
      <aside className="flex w-56 shrink-0 flex-col border-r border-surface">
        <div className="flex items-center justify-between border-b border-surface px-3 py-2">
          <span className="truncate text-sm font-medium text-text-primary">
            {vaultName}
          </span>
          <div className="ml-2 flex shrink-0 items-center gap-1">
            <button
              onClick={handleNewNoteOpen}
              className="rounded px-1 text-text-secondary transition-colors hover:text-text-primary"
              aria-label="New note"
            >
              +
            </button>
            <button
              onClick={onClose}
              className="rounded px-1 text-text-secondary transition-colors hover:text-text-primary"
              aria-label="Close vault"
            >
              ×
            </button>
          </div>
        </div>
        <div className="flex-1 overflow-y-auto py-1">
          {newNoteName !== null && (
            <div className="px-3 py-1">
              <input
                ref={newNoteInputCallbackRef}
                value={newNoteName}
                onChange={(e) => setNewNoteName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleNewNoteConfirm();
                  if (e.key === "Escape") setNewNoteName(null);
                }}
                onBlur={() => setNewNoteName(null)}
                className="w-full rounded bg-surface px-2 py-0.5 text-sm text-text-primary outline-none ring-1 ring-accent"
                spellCheck={false}
              />
            </div>
          )}
          {treeError ? (
            <p className="px-3 py-2 text-xs text-red-400">{treeError}</p>
          ) : (
            <FileTree
              entries={fileTree}
              activeFilePath={state.activeTabPath}
              onFileClick={handleFileClick}
            />
          )}
        </div>
      </aside>

      <div className="flex flex-1 flex-col overflow-hidden">
        <TabBar
          tabs={state.tabs}
          activeTabPath={state.activeTabPath}
          dirtyPaths={state.dirtyPaths}
          onTabClick={handleTabClick}
          onTabClose={handleTabClose}
        />
        <div className="flex-1 overflow-hidden">
          {activeContent !== null ? (
            <MarkdownEditor
              content={activeContent}
              onChange={handleContentChange}
            />
          ) : (
            <div className="flex h-full items-center justify-center text-text-secondary">
              <p className="text-sm">Open a file from the sidebar</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
