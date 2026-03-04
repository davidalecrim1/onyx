import { useEffect } from "react";
import { usePanelStore } from "../stores/panelStore";

interface Keybinding {
  key: string;
  meta?: boolean;
  shift?: boolean;
  alt?: boolean;
  action: () => void;
}

function matchesBinding(event: KeyboardEvent, binding: Keybinding): boolean {
  if (event.key.toLowerCase() !== binding.key.toLowerCase()) return false;
  if ((binding.meta ?? false) !== (event.metaKey || event.ctrlKey))
    return false;
  if ((binding.shift ?? false) !== event.shiftKey) return false;
  if ((binding.alt ?? false) !== event.altKey) return false;
  return true;
}

/// Registers all global keyboard shortcuts. Add new bindings to the array inside.
export function useKeybindings(saveRef: React.RefObject<() => void>) {
  useEffect(() => {
    const bindings: Keybinding[] = [
      {
        key: "s",
        meta: true,
        action: () => saveRef.current?.(),
      },
      {
        key: "b",
        meta: true,
        action: () => usePanelStore.getState().togglePanel("fileTree"),
      },
    ];

    function handler(event: KeyboardEvent) {
      for (const binding of bindings) {
        if (!matchesBinding(event, binding)) continue;
        event.preventDefault();
        binding.action();
        return;
      }
    }

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [saveRef]);
}
