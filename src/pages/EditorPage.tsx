import { useCallback, useEffect, useReducer, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import FileTree, { type FileTreeEntry } from "../components/FileTree";
import TabBar, { type Tab } from "../components/TabBar";
import MarkdownEditor from "../components/MarkdownEditor";
import VaultSwitcher from "../components/VaultSwitcher";
import AppLayout from "../components/AppLayout";
import CommandPalette from "../components/CommandPalette";
import FilePicker from "../components/FilePicker";
import { useKeybindings } from "../hooks/useKeybindings";
import { useCommandStore } from "../stores/commandStore";
import { usePanelStore } from "../stores/panelStore";
import { useCommandPaletteStore } from "../stores/commandPaletteStore";
import { useFilePickerStore } from "../stores/filePickerStore";

interface VaultEntry {
  name: string;
  path: string;
}

interface VaultSession {
  open_tabs: string[];
  active_tab: string | null;
}

interface Props {
  vaultPath: string;
  vaultName: string;
  knownVaults: VaultEntry[];
  onClose: () => void;
  onSwitchVault: (path: string, name: string) => void;
  onOpenWelcome: () => void;
}

interface EditorState {
  tabs: Tab[];
  activeTabPath: string | null;
  fileContents: Record<string, string>;
  dirtyPaths: Set<string>;
}

type EditorAction =
  | { type: "open_file"; path: string; name: string; content: string }
  | { type: "close_tab"; path: string }
  | { type: "close_all_tabs" }
  | { type: "activate_tab"; path: string }
  | { type: "update_content"; path: string; content: string }
  | { type: "mark_saved"; path: string }
  | { type: "rename_file"; oldPath: string; newPath: string; newName: string };

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
      const dirty = new Set(state.dirtyPaths);
      dirty.delete(action.path);
      return {
        tabs: remaining,
        activeTabPath: newActive,
        fileContents: rest,
        dirtyPaths: dirty,
      };
    }
    case "activate_tab":
      return { ...state, activeTabPath: action.path };
    case "update_content": {
      const dirty = new Set(state.dirtyPaths);
      dirty.add(action.path);
      return {
        ...state,
        fileContents: { ...state.fileContents, [action.path]: action.content },
        dirtyPaths: dirty,
      };
    }
    case "mark_saved": {
      const dirty = new Set(state.dirtyPaths);
      dirty.delete(action.path);
      return { ...state, dirtyPaths: dirty };
    }
    case "close_all_tabs":
      return {
        tabs: [],
        activeTabPath: null,
        fileContents: {},
        dirtyPaths: new Set<string>(),
      };
    case "rename_file": {
      const tabs = state.tabs.map((tab) =>
        tab.path === action.oldPath
          ? { path: action.newPath, name: action.newName }
          : tab,
      );
      const activeTabPath =
        state.activeTabPath === action.oldPath
          ? action.newPath
          : state.activeTabPath;
      const { [action.oldPath]: movedContent, ...restContents } =
        state.fileContents;
      const fileContents =
        movedContent !== undefined
          ? { ...restContents, [action.newPath]: movedContent }
          : restContents;
      const dirty = new Set(state.dirtyPaths);
      if (dirty.has(action.oldPath)) {
        dirty.delete(action.oldPath);
        dirty.add(action.newPath);
      }
      return { tabs, activeTabPath, fileContents, dirtyPaths: dirty };
    }
  }
}

export default function EditorPage({
  vaultPath,
  vaultName,
  knownVaults,
  onClose,
  onSwitchVault,
  onOpenWelcome,
}: Props) {
  const [fileTree, setFileTree] = useState<FileTreeEntry[]>([]);
  const [treeError, setTreeError] = useState<string | null>(null);
  const [newNoteName, setNewNoteName] = useState<string | null>(null);
  const [newFolderName, setNewFolderName] = useState<string | null>(null);
  const [sessionLoaded, setSessionLoaded] = useState(false);
  const [vimMode, setVimMode] = useState(false);
  const [selectedFolderPath, setSelectedFolderPath] = useState<string | null>(
    null,
  );
  const [state, dispatch] = useReducer(editorReducer, {
    tabs: [],
    activeTabPath: null,
    fileContents: {},
    dirtyPaths: new Set<string>(),
  });

  const { register, unregister } = useCommandStore();

  const fetchFileTree = useCallback(() => {
    invoke<FileTreeEntry[]>("get_file_tree", { vaultPath })
      .then(setFileTree)
      .catch((err) => setTreeError(String(err)));
  }, [vaultPath]);

  useEffect(() => {
    fetchFileTree();
  }, [fetchFileTree]);

  // Build the tag index whenever the vault changes so autocomplete is ready immediately.
  useEffect(() => {
    invoke("build_tag_index", { vaultPath }).catch((err) =>
      console.error("Failed to build tag index:", err),
    );
  }, [vaultPath]);

  useEffect(() => {
    invoke<{ vim_mode: boolean }>("get_settings")
      .then((settings) => setVimMode(settings.vim_mode))
      .catch((err) => console.error("Failed to load settings:", err));
  }, []);

  // Reset context folder when vault changes.
  useEffect(() => {
    setSelectedFolderPath(null);
  }, [vaultPath]);

  // Restore session after the file tree is available.
  useEffect(() => {
    if (sessionLoaded || fileTree.length === 0) return;

    invoke<VaultSession>("load_vault_session_cmd", { vaultPath })
      .then(async (session) => {
        for (const tabPath of session.open_tabs) {
          try {
            const content = await invoke<string>("read_file", {
              path: tabPath,
            });
            const name = tabPath.split("/").pop() ?? tabPath;
            dispatch({ type: "open_file", path: tabPath, name, content });
          } catch {
            // File may have been deleted since last session — skip it.
          }
        }
        if (session.active_tab) {
          dispatch({ type: "activate_tab", path: session.active_tab });
        }
        setSessionLoaded(true);
      })
      .catch(() => setSessionLoaded(true));
  }, [vaultPath, fileTree, sessionLoaded]);

  // Persist session whenever tabs or active tab changes (after initial load).
  useEffect(() => {
    if (!sessionLoaded) return;
    invoke("save_vault_session_cmd", {
      vaultPath,
      openTabs: state.tabs.map((tab) => tab.path),
      activeTab: state.activeTabPath,
    }).catch((err) => console.error("Failed to save session:", err));
  }, [vaultPath, sessionLoaded, state.tabs, state.activeTabPath]);

  const handleNewNoteOpen = useCallback(() => {
    setNewNoteName("Untitled.md");
  }, []);

  const handleNewFolderOpen = useCallback(() => {
    setNewFolderName("Untitled");
  }, []);

  const newNoteInputCallbackRef = useCallback((el: HTMLInputElement | null) => {
    if (el) el.select();
  }, []);

  const newFolderInputCallbackRef = useCallback(
    (el: HTMLInputElement | null) => {
      if (el) el.select();
    },
    [],
  );

  const handleNewNoteConfirm = useCallback(
    async (rawName: string) => {
      const name = rawName.trim() || "Untitled.md";
      const finalName = name.endsWith(".md") ? name : `${name}.md`;
      setNewNoteName(null);
      try {
        const basePath = selectedFolderPath ?? vaultPath;
        const filePath = await invoke<string>("create_file", {
          vaultPath: basePath,
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
    },
    [vaultPath, selectedFolderPath, fetchFileTree],
  );

  const handleNewFolderConfirm = useCallback(
    async (rawName: string) => {
      const name = rawName.trim() || "Untitled";
      setNewFolderName(null);
      try {
        const basePath = selectedFolderPath ?? vaultPath;
        await invoke("create_folder", { vaultPath: basePath, name });
        fetchFileTree();
      } catch (err) {
        console.error("Failed to create folder:", err);
      }
    },
    [vaultPath, selectedFolderPath, fetchFileTree],
  );

  const handleFolderClick = useCallback((path: string) => {
    setSelectedFolderPath(path);
  }, []);

  const handleFileClick = useCallback(
    async (path: string) => {
      const parentDir = path.substring(0, path.lastIndexOf("/"));
      setSelectedFolderPath(parentDir || null);
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

  const handleWikilinkCreate = useCallback(
    async (linkTarget: string) => {
      const fileName = `${linkTarget}.md`;
      try {
        const filePath = await invoke<string>("create_file", {
          vaultPath,
          name: fileName,
        });
        fetchFileTree();
        await handleFileClick(filePath);
      } catch (err) {
        console.error("Failed to create wikilink target:", err);
      }
    },
    [vaultPath, fetchFileTree, handleFileClick],
  );

  const handleFileDrop = useCallback(
    async (sourcePath: string, targetDirPath: string) => {
      try {
        await invoke("move_file", { sourcePath, targetDir: targetDirPath });
        fetchFileTree();
        dispatch({ type: "close_tab", path: sourcePath });
      } catch (err) {
        console.error("Failed to move file:", err);
      }
    },
    [fetchFileTree],
  );

  const handleFileDelete = useCallback(
    async (path: string) => {
      try {
        await invoke("delete_file", { path });
        dispatch({ type: "close_tab", path });
        fetchFileTree();
      } catch (err) {
        console.error("Failed to delete file:", err);
      }
    },
    [fetchFileTree],
  );

  const handleContentChange = useCallback(
    (content: string) => {
      if (!state.activeTabPath) return;
      dispatch({ type: "update_content", path: state.activeTabPath, content });
    },
    [state.activeTabPath],
  );

  const handleRename = useCallback(
    async (newStem: string) => {
      if (!state.activeTabPath) return;
      const oldPath = state.activeTabPath;
      try {
        const newPath = await invoke<string>("rename_file", {
          oldPath,
          newStem,
        });
        const newName = newPath.split("/").pop() ?? newPath;
        dispatch({ type: "rename_file", oldPath, newPath, newName });
        fetchFileTree();
      } catch (err) {
        console.error("Failed to rename file:", err);
      }
    },
    [state.activeTabPath, fetchFileTree],
  );

  const handleTabClick = useCallback(
    (path: string) => dispatch({ type: "activate_tab", path }),
    [],
  );

  const handleTabClose = useCallback(
    (path: string) => dispatch({ type: "close_tab", path }),
    [],
  );

  useEffect(() => {
    register({
      id: "editor.save",
      label: "Save File",
      execute: () => {
        if (!state.activeTabPath) return;
        const content = state.fileContents[state.activeTabPath];
        if (content === undefined) return;
        invoke("write_file", { path: state.activeTabPath, content })
          .then(() =>
            dispatch({ type: "mark_saved", path: state.activeTabPath! }),
          )
          .catch((err) => console.error("Failed to save file:", err));
      },
    });

    register({
      id: "tab.close",
      label: "Close Tab",
      execute: () => {
        if (state.activeTabPath) {
          dispatch({ type: "close_tab", path: state.activeTabPath });
        }
      },
    });

    register({
      id: "tab.closeAll",
      label: "Close All Tabs",
      execute: () => dispatch({ type: "close_all_tabs" }),
    });

    return () => {
      unregister("editor.save");
      unregister("tab.close");
      unregister("tab.closeAll");
    };
  }, [state.activeTabPath, state.fileContents, register, unregister]);

  useEffect(() => {
    register({
      id: "view.toggleSidebar",
      label: "Toggle Sidebar",
      execute: () => usePanelStore.getState().togglePanel("fileTree"),
    });
    return () => unregister("view.toggleSidebar");
  }, [register, unregister]);

  useEffect(() => {
    register({
      id: "view.palette",
      label: "Open Command Palette",
      execute: () => useCommandPaletteStore.getState().open(),
    });
    register({
      id: "file.open",
      label: "Open File",
      keywords: ["file", "open", "find", "search"],
      execute: () => useFilePickerStore.getState().open(),
    });
    register({
      id: "file.newNote",
      label: "New Note",
      keywords: ["create", "file", "note", "document"],
      execute: handleNewNoteOpen,
    });
    register({
      id: "file.newFolder",
      label: "New Folder",
      keywords: ["create", "directory", "folder"],
      execute: handleNewFolderOpen,
    });
    return () => {
      unregister("view.palette");
      unregister("file.open");
      unregister("file.newNote");
      unregister("file.newFolder");
    };
  }, [register, unregister, handleNewNoteOpen, handleNewFolderOpen]);

  useKeybindings();

  const paletteOpen = useCommandPaletteStore((s) => s.isOpen);
  const closePalette = useCommandPaletteStore((s) => s.close);
  const filePickerOpen = useFilePickerStore((s) => s.isOpen);
  const closeFilePicker = useFilePickerStore((s) => s.close);

  const activeContent =
    state.activeTabPath !== null
      ? (state.fileContents[state.activeTabPath] ?? "")
      : null;

  const sidebar = (
    <>
      <div className="flex items-center justify-between border-b border-surface px-3 py-2 pt-[38px]">
        <span
          data-tauri-drag-region
          className="flex-1 truncate text-sm font-medium text-text-primary"
        >
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
            onClick={handleNewFolderOpen}
            className="rounded px-1 text-text-secondary transition-colors hover:text-text-primary"
            aria-label="New folder"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 14 14"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              aria-hidden="true"
            >
              <path
                d="M1 3.5C1 2.94772 1.44772 2.5 2 2.5H5.5L7 4H12C12.5523 4 13 4.44772 13 5V10.5C13 11.0523 12.5523 11.5 12 11.5H2C1.44772 11.5 1 11.0523 1 10.5V3.5Z"
                stroke="currentColor"
                strokeWidth="1.2"
                strokeLinejoin="round"
              />
            </svg>
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
                if (e.key === "Enter") handleNewNoteConfirm(newNoteName ?? "");
                if (e.key === "Escape") setNewNoteName(null);
              }}
              onBlur={() => setNewNoteName(null)}
              className="w-full rounded bg-surface px-2 py-0.5 text-sm text-text-primary outline-none ring-1 ring-accent"
              spellCheck={false}
            />
          </div>
        )}
        {newFolderName !== null && (
          <div className="px-3 py-1">
            <input
              ref={newFolderInputCallbackRef}
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter")
                  handleNewFolderConfirm(newFolderName ?? "");
                if (e.key === "Escape") setNewFolderName(null);
              }}
              onBlur={() => setNewFolderName(null)}
              className="w-full rounded bg-surface px-2 py-0.5 text-sm text-text-primary outline-none ring-1 ring-accent"
              spellCheck={false}
            />
          </div>
        )}
        {treeError ? (
          <div className="px-3 py-2">
            <p className="text-xs text-red-400">{treeError}</p>
            <button
              onClick={onClose}
              className="mt-2 text-xs text-accent hover:underline"
            >
              Return to vault picker
            </button>
          </div>
        ) : (
          <FileTree
            entries={fileTree}
            activeFilePath={state.activeTabPath}
            vaultPath={vaultPath}
            onFileClick={handleFileClick}
            onFolderClick={handleFolderClick}
            onFileDrop={handleFileDrop}
            onDelete={handleFileDelete}
          />
        )}
      </div>
      <VaultSwitcher
        currentVaultName={vaultName}
        currentVaultPath={vaultPath}
        vaults={knownVaults}
        onSwitch={onSwitchVault}
        onOpenWelcome={onOpenWelcome}
      />
    </>
  );

  const tabBar = (
    <TabBar
      tabs={state.tabs}
      activeTabPath={state.activeTabPath}
      dirtyPaths={state.dirtyPaths}
      onTabClick={handleTabClick}
      onTabClose={handleTabClose}
    />
  );

  return (
    <>
      <AppLayout sidebar={sidebar} tabBar={tabBar}>
        <div className="flex-1 overflow-hidden">
          {activeContent !== null ? (
            <MarkdownEditor
              content={activeContent}
              onChange={handleContentChange}
              vimMode={vimMode}
              filePath={state.activeTabPath}
              onRename={handleRename}
              vaultPath={vaultPath}
              onWikilinkOpen={handleFileClick}
              onWikilinkCreate={handleWikilinkCreate}
            />
          ) : (
            <div className="flex h-full items-center justify-center text-text-secondary">
              <p className="text-sm">Open a file from the sidebar</p>
            </div>
          )}
        </div>
      </AppLayout>
      {paletteOpen && <CommandPalette onClose={closePalette} />}
      {filePickerOpen && (
        <FilePicker
          files={fileTree}
          onOpen={handleFileClick}
          onCreate={handleNewNoteConfirm}
          onClose={closeFilePicker}
        />
      )}
    </>
  );
}
