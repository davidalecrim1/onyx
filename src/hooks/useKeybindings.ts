import { useEffect, useRef } from "react";
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

interface ParsedChordBinding {
  command: string;
  first: ParsedBinding;
  second: ParsedBinding;
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

/// Resolves the logical key name from a KeyboardEvent, ignoring OS-level character remapping.
/// On macOS, Option combinations produce special characters (e.g. Option+B → "∫") in event.key.
/// Using event.code ("KeyB" → "b") gives the physical key regardless of modifier state.
export function resolveKey(event: Pick<KeyboardEvent, "code" | "key">): string {
  if (event.code.startsWith("Key")) return event.code.slice(3).toLowerCase();
  if (event.code.startsWith("Digit")) return event.code.slice(5).toLowerCase();
  return event.key.toLowerCase();
}

function matchesBinding(
  event: KeyboardEvent,
  binding: Omit<ParsedBinding, "command">,
): boolean {
  if (resolveKey(event) !== binding.key) return false;
  if (binding.cmd !== event.metaKey) return false;
  if (binding.ctrl !== event.ctrlKey) return false;
  if (binding.shift !== event.shiftKey) return false;
  if (binding.alt !== event.altKey) return false;
  return true;
}

type RawBinding = { command: string; key: string };

function isChord(raw: string): boolean {
  return raw.includes(" ");
}

const singleBindings: ParsedBinding[] = (keybindings as RawBinding[])
  .filter((entry) => !isChord(entry.key))
  .map((entry) => ({ command: entry.command, ...parseBinding(entry.key) }));

const chordBindings: ParsedChordBinding[] = (keybindings as RawBinding[])
  .filter((entry) => isChord(entry.key))
  .map((entry) => {
    const [first, second] = entry.key.split(" ");
    return {
      command: entry.command,
      first: { command: "", ...parseBinding(first) },
      second: { command: "", ...parseBinding(second) },
    };
  });

/// Listens for global keydown events and dispatches matching commands through the command store.
/// Supports both single bindings and two-key chord sequences (e.g., "Cmd+R W").
export function useKeybindings() {
  const pendingChordRef = useRef<ParsedChordBinding[] | null>(null);

  useEffect(() => {
    function handler(event: KeyboardEvent) {
      // If we're waiting for the second key of a chord, check for matches.
      if (pendingChordRef.current !== null) {
        const matched = pendingChordRef.current.find((chord) =>
          matchesBinding(event, chord.second),
        );
        pendingChordRef.current = null;
        if (matched) {
          event.preventDefault();
          useCommandStore.getState().execute(matched.command);
        }
        return;
      }

      // Check if this key starts any chord sequence.
      const chordStarters = chordBindings.filter((chord) =>
        matchesBinding(event, chord.first),
      );
      if (chordStarters.length > 0) {
        event.preventDefault();
        pendingChordRef.current = chordStarters;
        return;
      }

      // Fall through to single bindings.
      for (const binding of singleBindings) {
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
  const entry = (keybindings as RawBinding[]).find(
    (binding) => binding.command === commandId,
  );
  return entry?.key ?? null;
}
