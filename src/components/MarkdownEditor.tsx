import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import CodeMirror, {
  EditorView,
  EditorSelection,
  type ReactCodeMirrorRef,
} from "@uiw/react-codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { GFM, Strikethrough, TaskList } from "@lezer/markdown";
import { vim } from "@replit/codemirror-vim";
import { invoke } from "@tauri-apps/api/core";
import { marked } from "marked";
import DOMPurify from "dompurify";
import { markdownDecorations } from "../extensions/markdownDecorations";
import { tagAutocomplete } from "../extensions/tagAutocomplete";
import {
  setWikilinkConfig,
  wikilinkConfigField,
  wikilinkViewPlugin,
} from "../extensions/wikilinkDecorations";
import {
  imageConfigField,
  setImageConfig,
  imageDecorations,
} from "../extensions/imageDecorations";
import { parentScrollIntoView } from "../extensions/parentScrollIntoView";

const onyxTheme = EditorView.theme(
  {
    "&": { backgroundColor: "#282c33", color: "#dce0e5", height: "auto", fontFamily: '"Zed Sans Extended", system-ui, sans-serif' },
    "&.cm-focused": { outline: "none" },
    ".cm-scroller": { overflow: "visible" },
    ".cm-content": { caretColor: "#74ade8", lineHeight: "1.8" },
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

function PencilIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
      <path d="m15 5 4 4" />
    </svg>
  );
}

function EyeIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
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
  const [isEditing, setIsEditing] = useState(true);

  // Load the current tag list whenever a vault becomes available.
  useEffect(() => {
    if (!vaultPath) return;
    invoke<string[]>("get_tags", { vaultPath })
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

  useEffect(() => {
    const view = editorRef.current?.view;
    if (!view || !vaultPath || !filePath) return;
    view.dispatch({
      effects: setImageConfig.of({ vaultPath, filePath }),
    });
  }, [vaultPath, filePath]);

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

  // Refocus editor when switching back to editing mode.
  useEffect(() => {
    if (isEditing) {
      editorRef.current?.view?.focus();
    }
  }, [isEditing]);

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
        invoke("update_file_tags", {
          vaultPath,
          filePath,
          content: value,
        }).catch(() => {});
        invoke<string[]>("get_tags", { vaultPath })
          .then(setTags)
          .catch(() => {});
      }, 800);
    },
    [onChange, filePath, vaultPath],
  );

  const extensions = useMemo(
    () => [
      ...(vimMode ? [vim()] : []),
      markdown({ extensions: [TaskList, GFM, Strikethrough] }),
      EditorView.lineWrapping,
      onyxTheme,
      markdownDecorations,
      tagAutocomplete(tags),
      wikilinkConfigField,
      wikilinkViewPlugin,
      imageConfigField,
      ...imageDecorations,
      parentScrollIntoView,
    ],
    [vimMode, tags],
  );

  const commitRename = useCallback(() => {
    const sanitized = sanitizeFileName(titleValue);
    if (!sanitized || sanitized === fileStem) return;
    onRename(sanitized);
  }, [titleValue, fileStem, onRename]);

  const renderedHtml = useMemo(() => {
    // Consecutive tag-only lines collapse into one <p> in standard markdown.
    // Insert a blank line between them so each renders as its own paragraph.
    const TAG_LINE_RE = /^(#[a-zA-Z][a-zA-Z0-9_-]*\s*)+$/;
    const preprocessed = content
      .split("\n")
      .reduce<string[]>((acc, line, index, lines) => {
        acc.push(line);
        const nextLine = lines[index + 1];
        if (
          nextLine !== undefined &&
          TAG_LINE_RE.test(line.trim()) &&
          TAG_LINE_RE.test(nextLine.trim())
        ) {
          acc.push("");
        }
        return acc;
      }, [])
      .join("\n");

    const raw = marked.parse(preprocessed, { async: false }) as string;
    // Wrap #tag tokens in a styled span after sanitization-safe HTML is built.
    const withTags = raw.replace(
      /(?<=^|[\s>])#([a-zA-Z][a-zA-Z0-9_-]*)/g,
      '<span class="onyx-tag">#$1</span>',
    );
    return DOMPurify.sanitize(withTags, {
      ADD_ATTR: ["class"],
    });
  }, [content]);

  return (
    <div className="flex h-full justify-center overflow-y-auto bg-background">
      <div className="w-full max-w-[806px] px-8 py-6">
        {filePath &&
          vaultPath &&
          (() => {
            const normalizedVault = vaultPath.endsWith("/")
              ? vaultPath
              : vaultPath + "/";
            const relative = filePath.startsWith(normalizedVault)
              ? filePath.slice(normalizedVault.length)
              : filePath;
            const segments = relative.split("/");
            const fileName = segments[segments.length - 1];
            return (
              <div className="relative mb-4 text-center text-sm text-text-secondary">
                {segments.length > 1 ? (
                  segments.map((segment, index) => (
                    <span key={index}>
                      {index > 0 && <span className="mx-1 opacity-50">/</span>}
                      {index === segments.length - 1 ? (
                        <span className="text-text-primary">{segment}</span>
                      ) : (
                        segment
                      )}
                    </span>
                  ))
                ) : (
                  <span className="text-text-primary">{fileName}</span>
                )}
                <button
                  onClick={() => setIsEditing((prev) => !prev)}
                  className="absolute right-0 top-1/2 -translate-y-1/2 text-text-secondary hover:text-text-primary transition-colors"
                  title={
                    isEditing
                      ? "Switch to reading mode"
                      : "Switch to editing mode"
                  }
                >
                  {isEditing ? <EyeIcon /> : <PencilIcon />}
                </button>
              </div>
            );
          })()}
        {filePath &&
          (isEditing ? (
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
          ) : (
            <h1 className="onyx-inline-title">{titleValue}</h1>
          ))}
        {isEditing ? (
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
            style={{ fontSize: "16px" }}
          />
        ) : (
          <div
            className="onyx-reading-view"
            dangerouslySetInnerHTML={{ __html: renderedHtml }}
          />
        )}
      </div>
    </div>
  );
}
