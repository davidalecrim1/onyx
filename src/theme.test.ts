import { describe, it, expect, beforeEach } from "vitest";
import { DEFAULT_PALETTE, mergeTheme, applyTheme } from "./theme";

beforeEach(() => {
  document.getElementById("onyx-theme")?.remove();
});

describe("mergeTheme", () => {
  it("returns the default palette unchanged when given empty overrides", () => {
    const result = mergeTheme({});
    expect(result).toEqual(DEFAULT_PALETTE);
  });

  it("applies a partial override without mutating DEFAULT_PALETTE", () => {
    const original = { ...DEFAULT_PALETTE };
    const result = mergeTheme({ accent: "#ff0000" });
    expect(result.accent).toBe("#ff0000");
    expect(DEFAULT_PALETTE).toEqual(original);
  });

  it("leaves non-overridden keys at their default values", () => {
    const result = mergeTheme({ accent: "#ff0000" });
    expect(result.background).toBe(DEFAULT_PALETTE.background);
    expect(result.surface).toBe(DEFAULT_PALETTE.surface);
  });
});

describe("applyTheme", () => {
  it("creates a <style id='onyx-theme'> element in document.head", () => {
    applyTheme(DEFAULT_PALETTE);
    const tag = document.getElementById("onyx-theme");
    expect(tag).not.toBeNull();
    expect(tag?.tagName.toLowerCase()).toBe("style");
  });

  it("includes the expected CSS variable in the injected style", () => {
    applyTheme(DEFAULT_PALETTE);
    const tag = document.getElementById("onyx-theme") as HTMLStyleElement;
    expect(tag.textContent).toContain(
      `--onyx-background: ${DEFAULT_PALETTE.background}`,
    );
  });

  it("replaces the existing tag instead of duplicating it on repeated calls", () => {
    applyTheme(DEFAULT_PALETTE);
    applyTheme({ ...DEFAULT_PALETTE, accent: "#ff0000" });
    const tags = document.querySelectorAll("#onyx-theme");
    expect(tags.length).toBe(1);
    expect((tags[0] as HTMLStyleElement).textContent).toContain(
      "--onyx-accent: #ff0000",
    );
  });
});
