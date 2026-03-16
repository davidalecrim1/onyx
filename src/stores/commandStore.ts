import { create } from "zustand";

interface CommandEntry {
  id: string;
  label: string;
  keywords?: string[];
  execute: () => void;
}

interface CommandState {
  commands: Map<string, CommandEntry>;
  register: (entry: CommandEntry) => void;
  unregister: (id: string) => void;
  execute: (id: string) => boolean;
}

/// Central command registry. Components register actions on mount; keybindings and (future) command palette dispatch through here.
export const useCommandStore = create<CommandState>()((set, get) => ({
  commands: new Map(),

  register: (entry) =>
    set((state) => {
      const next = new Map(state.commands);
      next.set(entry.id, entry);
      return { commands: next };
    }),

  unregister: (id) =>
    set((state) => {
      const next = new Map(state.commands);
      next.delete(id);
      return { commands: next };
    }),

  execute: (id) => {
    const entry = get().commands.get(id);
    if (!entry) return false;
    entry.execute();
    return true;
  },
}));
