import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import CodeMirror, {
  EditorView,
  EditorSelection,
  type ReactCodeMirrorRef,
} from "@uiw/react-codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { vim } from "@replit/codemirror-vim";
import { invoke } from "@tauri-apps/api/core";
import { markdownDecorations } from "../extensions/markdownDecorations";
import { tagAutocomplete } from "../extensions/tagAutocomplete";
import {
  setWikilinkConfig,
  wikilinkConfigField,
  wikilinkViewPlugin,
} from "../extensions/wikilinkDecorations";

const onyxTheme = EditorView.theme(
  {
    "&": { backgroundColor: "#282c33", color: "#dce0e5", height: "auto" },
    "&.cm-focused": { outline: "none" },
    ".cm-scroller": { overflow: "visible" },
    ".cm-content": { caretColor: "#74ade8" },
    ".cm-cursor": { borderLeftColor: "#74ade8" },
    ".cm-selectionBackground, ::selection": { backgroundColor: "#454a56" },
    ".cm-activeLine": { backgroundColor: "transparent" },
    ".cm-line": { color: "#dce0e5" },
  },
  { dark: true },
);

function sanitizeFileName(raw: string): string {
  return raw
    .replace(/[\\/:*?"<>|]/g, "")
    .replace(/^[.\s]+|[.\s]+$/g, "")
    .trim();
}

interface Props {
  content: string;
  onChange: (value: string) => void;
  vimMode: boolean;
  filePath: string | null;
  onRename: (newStem: string) => void;
  vaultPath: string | null;
  onWikilinkOpen: (path: string) => void;
  onWikilinkCreate: (linkTarget: string) => void;
}

export default function MarkdownEditor({
  content,
  onChange,
  vimMode,
  filePath,
  onRename,
  vaultPath,
  onWikilinkOpen,
  onWikilinkCreate,
}: Props) {
  const editorRef = useRef<ReactCodeMirrorRef>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [tags, setTags] = useState<string[]>([]);

  // Load the current tag list whenever a vault becomes available.
  useEffect(() => {
    if (!vaultPath) return;
    invoke<string[]>("get_tags")
      .then(setTags)
      .catch(() => {});
  }, [vaultPath]);

  useEffect(() => {
    const view = editorRef.current?.view;
    if (!view || !vaultPath) return;
    view.dispatch({
      effects: setWikilinkConfig.of({
        vaultPath,
        onOpen: onWikilinkOpen,
        onCreate: onWikilinkCreate,
      }),
    });
  }, [vaultPath, onWikilinkOpen, onWikilinkCreate]);

  const fileStem = filePath
    ? (filePath
        .split("/")
        .pop()
        ?.replace(/\.[^.]+$/, "") ?? "")
    : "";

  const [titleValue, setTitleValue] = useState(fileStem);

  useEffect(() => {
    setTitleValue(fileStem);
  }, [fileStem]);

  useEffect(() => {
    editorRef.current?.view?.focus();
  }, [filePath]);

  // Cancel any pending debounce when the component unmounts.
  useEffect(
    () => () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    },
    [],
  );

  const handleChange = useCallback(
    (value: string) => {
      onChange(value);
      if (!filePath) return;
      if (debounceRef.current) clearTimeout(debounceRef.current);
      debounceRef.current = setTimeout(() => {
        invoke("update_file_tags", { filePath, content: value }).catch(
          () => {},
        );
        invoke<string[]>("get_tags")
          .then(setTags)
          .catch(() => {});
      }, 800);
    },
    [onChange, filePath],
  );

  const extensions = useMemo(
    () => [
      ...(vimMode ? [vim()] : []),
      markdown(),
      onyxTheme,
      markdownDecorations,
      tagAutocomplete(tags),
      wikilinkConfigField,
      wikilinkViewPlugin,
    ],
    [vimMode, tags],
  );

  const commitRename = useCallback(() => {
    const sanitized = sanitizeFileName(titleValue);
    if (!sanitized || sanitized === fileStem) return;
    onRename(sanitized);
  }, [titleValue, fileStem, onRename]);

  return (
    <div className="flex h-full justify-center overflow-y-auto bg-background">
      <div className="w-full max-w-2xl px-8 py-6">
        {filePath && (
          <input
            value={titleValue}
            onChange={(e) => setTitleValue(e.target.value)}
            onBlur={commitRename}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                commitRename();
                editorRef.current?.view?.focus();
              }
              if (e.key === "Escape") {
                setTitleValue(fileStem);
              }
              if (e.key === "ArrowDown") {
                e.preventDefault();
                const view = editorRef.current?.view;
                if (view) {
                  view.dispatch({
                    selection: EditorSelection.cursor(0),
                    scrollIntoView: true,
                  });
                  view.focus();
                }
              }
            }}
            className="onyx-inline-title"
            spellCheck={false}
            aria-label="File name"
          />
        )}
        <CodeMirror
          ref={editorRef}
          value={content}
          onChange={handleChange}
          extensions={extensions}
          theme="none"
          basicSetup={{
            lineNumbers: false,
            foldGutter: false,
            highlightActiveLine: false,
            highlightSelectionMatches: true,
            syntaxHighlighting: false,
          }}
          style={{ fontSize: "18px" }}
        />
      </div>
    </div>
  );
}
