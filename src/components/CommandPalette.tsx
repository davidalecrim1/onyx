import { useEffect, useMemo, useRef, useState } from "react";
import { useCommandStore } from "../stores/commandStore";
import { getKeybindingLabel } from "../hooks/useKeybindings";

interface Props {
  onClose: () => void;
}

export default function CommandPalette({ onClose }: Props) {
  const commands = useCommandStore((s) => s.commands);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef<HTMLUListElement>(null);

  const sortedCommands = useMemo(
    () =>
      Array.from(commands.values()).sort((a, b) =>
        a.label.localeCompare(b.label),
      ),
    [commands],
  );

  const filteredCommands = useMemo(() => {
    if (!query) return sortedCommands;
    const lower = query.toLowerCase();
    return sortedCommands.filter(
      (cmd) =>
        cmd.label.toLowerCase().includes(lower) ||
        cmd.keywords?.some((kw) => kw.toLowerCase().includes(lower)),
    );
  }, [sortedCommands, query]);

  // Reset selection when query changes.
  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  // Scroll selected item into view.
  useEffect(() => {
    const item = listRef.current?.children[selectedIndex] as
      | HTMLElement
      | undefined;
    item?.scrollIntoView?.({ block: "nearest" });
  }, [selectedIndex]);

  function executeCommand(index: number) {
    const cmd = filteredCommands[index];
    if (!cmd) return;
    onClose();
    cmd.execute();
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((i) => Math.min(i + 1, filteredCommands.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      executeCommand(selectedIndex);
    } else if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-32"
      onMouseDown={onClose}
    >
      <div
        className="w-full max-w-lg rounded-lg bg-surface shadow-xl ring-1 ring-surface-hover"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type a command…"
          autoFocus
          className="w-full border-b border-surface-hover bg-transparent px-4 py-3 text-sm text-text-primary outline-none placeholder:text-text-secondary"
          spellCheck={false}
        />
        <ul ref={listRef} className="max-h-80 overflow-y-auto py-1">
          {filteredCommands.map((cmd, index) => {
            const hint = getKeybindingLabel(cmd.id);
            const isSelected = index === selectedIndex;
            return (
              <li
                key={cmd.id}
                onMouseDown={(e) => {
                  e.preventDefault();
                  executeCommand(index);
                }}
                onMouseEnter={() => setSelectedIndex(index)}
                className={`flex cursor-pointer items-center justify-between px-4 py-2 text-sm ${
                  isSelected
                    ? "bg-surface-hover text-text-primary"
                    : "text-text-secondary"
                }`}
              >
                <span>{cmd.label}</span>
                {hint && (
                  <span className="ml-4 shrink-0 text-xs text-text-secondary opacity-70">
                    {hint}
                  </span>
                )}
              </li>
            );
          })}
          {filteredCommands.length === 0 && (
            <li className="px-4 py-3 text-sm text-text-secondary">
              No commands found
            </li>
          )}
        </ul>
      </div>
    </div>
  );
}
