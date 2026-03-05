import { useEffect } from "react";
import { useCommandStore } from "../stores/commandStore";
import macBindings from "../keybindings.mac.json";
import linuxBindings from "../keybindings.linux.json";

const isMac = navigator.platform.startsWith("Mac");
const keybindings = isMac ? macBindings : linuxBindings;

interface ParsedBinding {
  command: string;
  key: string;
  cmd: boolean;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
}

function parseBinding(raw: string): Omit<ParsedBinding, "command"> {
  const parts = raw.split("+");
  const key = parts[parts.length - 1].toLowerCase();
  return {
    key,
    cmd: parts.includes("Cmd"),
    ctrl: parts.includes("Ctrl"),
    shift: parts.includes("Shift"),
    alt: parts.includes("Alt"),
  };
}

function matchesBinding(event: KeyboardEvent, binding: ParsedBinding): boolean {
  if (event.key.toLowerCase() !== binding.key) return false;
  if (binding.cmd !== event.metaKey) return false;
  if (binding.ctrl !== event.ctrlKey) return false;
  if (binding.shift !== event.shiftKey) return false;
  if (binding.alt !== event.altKey) return false;
  return true;
}

/// Listens for global keydown events and dispatches matching commands through the command store.
export function useKeybindings() {
  useEffect(() => {
    const parsed: ParsedBinding[] = keybindings.map((entry) => ({
      command: entry.command,
      ...parseBinding(entry.key),
    }));

    function handler(event: KeyboardEvent) {
      for (const binding of parsed) {
        if (!matchesBinding(event, binding)) continue;
        event.preventDefault();
        useCommandStore.getState().execute(binding.command);
        return;
      }
    }

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
}

/// Looks up the key combo for a command ID and returns the label as written in the JSON, or null if unbound.
export function getKeybindingLabel(commandId: string): string | null {
  const entry = keybindings.find((binding) => binding.command === commandId);
  return entry?.key ?? null;
}
