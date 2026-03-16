import { describe, it, expect, beforeEach } from "vitest";
import { useCommandPaletteStore } from "./commandPaletteStore";

beforeEach(() => {
  useCommandPaletteStore.setState({ isOpen: false });
});

describe("commandPaletteStore", () => {
  it("starts closed", () => {
    expect(useCommandPaletteStore.getState().isOpen).toBe(false);
  });

  it("open() sets isOpen to true", () => {
    useCommandPaletteStore.getState().open();
    expect(useCommandPaletteStore.getState().isOpen).toBe(true);
  });

  it("close() sets isOpen to false", () => {
    useCommandPaletteStore.getState().open();
    useCommandPaletteStore.getState().close();
    expect(useCommandPaletteStore.getState().isOpen).toBe(false);
  });
});
