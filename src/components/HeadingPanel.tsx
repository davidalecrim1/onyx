import {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from "react";

interface Heading {
  level: number;
  text: string;
  position: number;
}

function parseHeadings(content: string): Heading[] {
  const headings: Heading[] = [];
  const lines = content.split("\n");
  let pos = 0;
  for (const line of lines) {
    const match = /^(#{1,6})\s+(.+)/.exec(line);
    if (match) {
      headings.push({
        level: match[1].length,
        text: match[2].trim(),
        position: pos,
      });
    }
    pos += line.length + 1;
  }
  return headings;
}

export interface HeadingPanelHandle {
  /// Focuses the panel and resets selection to the first heading.
  focus: () => void;
}

interface Props {
  /// Raw markdown content of the active tab, or null when no tab is open.
  content: string | null;
  /// Called when the user activates a heading — the argument is the heading's char offset.
  onJump: (position: number) => void;
}

const HeadingPanel = forwardRef<HeadingPanelHandle, Props>(
  function HeadingPanel({ content, onJump }, ref) {
    const containerRef = useRef<HTMLDivElement>(null);
    const [selectedIndex, setSelectedIndex] = useState(0);

    const headings = useMemo(
      () => (content !== null ? parseHeadings(content) : []),
      [content],
    );

    // Reset selection when the heading list changes (e.g. tab switch).
    useEffect(() => {
      setSelectedIndex(0);
    }, [headings]);

    useImperativeHandle(ref, () => ({
      focus() {
        setSelectedIndex(0);
        containerRef.current?.focus();
      },
    }));

    const handleKeyDown = useCallback(
      (event: React.KeyboardEvent<HTMLDivElement>) => {
        if (headings.length === 0) return;
        if (event.key === "ArrowDown") {
          event.preventDefault();
          setSelectedIndex((prev) => Math.min(prev + 1, headings.length - 1));
        } else if (event.key === "ArrowUp") {
          event.preventDefault();
          setSelectedIndex((prev) => Math.max(prev - 1, 0));
        } else if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onJump(headings[selectedIndex].position);
        }
      },
      [headings, selectedIndex, onJump],
    );

    if (content === null) return null;

    return (
      <div
        ref={containerRef}
        tabIndex={-1}
        onKeyDown={handleKeyDown}
        className="flex h-full flex-col overflow-hidden outline-none"
      >
        <div className="shrink-0 border-b border-surface px-3 py-2 text-xs font-medium uppercase tracking-wider text-text-secondary">
          Outline
        </div>
        <div className="flex-1 overflow-y-auto py-1">
          {headings.length === 0 ? (
            <p className="px-3 py-2 text-xs text-text-secondary">No headings</p>
          ) : (
            headings.map((heading, index) => (
              <button
                key={index}
                onClick={() => {
                  setSelectedIndex(index);
                  onJump(heading.position);
                }}
                style={{ paddingLeft: `${(heading.level - 1) * 12 + 12}px` }}
                className={`w-full truncate py-0.5 pr-3 text-left text-sm transition-colors ${
                  index === selectedIndex
                    ? "bg-surface text-accent"
                    : "text-text-secondary hover:text-text-primary"
                }`}
              >
                {heading.text}
              </button>
            ))
          )}
        </div>
      </div>
    );
  },
);

export default HeadingPanel;
