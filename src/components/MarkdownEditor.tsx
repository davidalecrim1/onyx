import { useCallback, useEffect, useRef, useState } from "react";
import CodeMirror, {
  EditorView,
  type ReactCodeMirrorRef,
} from "@uiw/react-codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { vim } from "@replit/codemirror-vim";
import { markdownDecorations } from "../extensions/markdownDecorations";

const onyxTheme = EditorView.theme(
  {
    "&": { backgroundColor: "#282c33", color: "#dce0e5" },
    "&.cm-focused": { outline: "none" },
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
}

export default function MarkdownEditor({
  content,
  onChange,
  vimMode,
  filePath,
  onRename,
}: Props) {
  const editorRef = useRef<ReactCodeMirrorRef>(null);

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
  }, [content]);

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
              }
              if (e.key === "Escape") {
                setTitleValue(fileStem);
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
          onChange={onChange}
          extensions={[
            ...(vimMode ? [vim()] : []),
            markdown(),
            onyxTheme,
            markdownDecorations,
          ]}
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
