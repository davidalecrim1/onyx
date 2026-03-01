import { useEffect, useRef } from "react";
import CodeMirror, { EditorView, type ReactCodeMirrorRef } from "@uiw/react-codemirror";
import { markdown } from "@codemirror/lang-markdown";
import { vim } from "@replit/codemirror-vim";

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

interface Props {
  content: string;
  onChange: (value: string) => void;
}

export default function MarkdownEditor({ content, onChange }: Props) {
  const editorRef = useRef<ReactCodeMirrorRef>(null);

  useEffect(() => {
    editorRef.current?.view?.focus();
  }, [content]);

  return (
    <div className="flex h-full justify-center overflow-y-auto bg-background">
      <div className="w-full max-w-2xl px-8 py-6">
        <CodeMirror
          ref={editorRef}
          value={content}
          onChange={onChange}
          extensions={[vim(), markdown(), onyxTheme]}
          theme="none"
          basicSetup={{
            lineNumbers: false,
            foldGutter: false,
            highlightActiveLine: false,
            highlightSelectionMatches: true,
          }}
          style={{ fontSize: "18px" }}
        />
      </div>
    </div>
  );
}
