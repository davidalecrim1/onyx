import { create } from "zustand";
import { persist } from "zustand/middleware";

interface PanelConfig {
  isOpen: boolean;
  width: number;
  height: number;
  minSize: number;
  maxSize: number;
}

interface PanelState {
  panels: Record<string, PanelConfig>;
  togglePanel: (id: string) => void;
  setOpen: (id: string, open: boolean) => void;
  resize: (id: string, size: number) => void;
}

const DEFAULT_PANELS: Record<string, PanelConfig> = {
  fileTree: {
    isOpen: true,
    width: 260,
    height: 0,
    minSize: 160,
    maxSize: 480,
  },
  outline: {
    isOpen: false,
    width: 240,
    height: 0,
    minSize: 160,
    maxSize: 400,
  },
};

export const usePanelStore = create<PanelState>()(
  persist(
    (set) => ({
      panels: DEFAULT_PANELS,
      togglePanel: (id) =>
        set((state) => {
          const panel = state.panels[id];
          if (!panel) return state;
          return {
            panels: {
              ...state.panels,
              [id]: { ...panel, isOpen: !panel.isOpen },
            },
          };
        }),
      setOpen: (id, open) =>
        set((state) => {
          const panel = state.panels[id];
          if (!panel) return state;
          return {
            panels: { ...state.panels, [id]: { ...panel, isOpen: open } },
          };
        }),
      resize: (id, size) =>
        set((state) => {
          const panel = state.panels[id];
          if (!panel) return state;
          const clamped = Math.max(
            panel.minSize,
            Math.min(panel.maxSize, size),
          );
          return {
            panels: {
              ...state.panels,
              [id]: { ...panel, width: clamped, height: clamped },
            },
          };
        }),
    }),
    {
      name: "onyx-panels",
      partialize: (state) => ({ panels: state.panels }),
    },
  ),
);
