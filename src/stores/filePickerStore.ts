import { create } from "zustand";

interface FilePickerState {
  isOpen: boolean;
  open: () => void;
  close: () => void;
}

export const useFilePickerStore = create<FilePickerState>()((set) => ({
  isOpen: false,
  open: () => set({ isOpen: true }),
  close: () => set({ isOpen: false }),
}));
