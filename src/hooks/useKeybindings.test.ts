import { describe, it, expect } from "vitest";
import { resolveKey } from "./useKeybindings";

function fakeEvent(code: string, key: string) {
  return { code, key };
}

describe("resolveKey", () => {
  it("returns the letter for a standard KeyX code", () => {
    expect(resolveKey(fakeEvent("KeyB", "b"))).toBe("b");
  });

  it("returns lowercase for a KeyX code regardless of shift", () => {
    expect(resolveKey(fakeEvent("KeyB", "B"))).toBe("b");
  });

  it("returns the digit for a DigitN code", () => {
    expect(resolveKey(fakeEvent("Digit3", "3"))).toBe("3");
  });

  it("falls back to event.key for non-Key/Digit codes (e.g. Enter)", () => {
    expect(resolveKey(fakeEvent("Enter", "Enter"))).toBe("enter");
  });

  it("ignores macOS Option-remapped characters for KeyX codes", () => {
    // Option+B on macOS produces "∫" in event.key — resolveKey must return "b".
    expect(resolveKey(fakeEvent("KeyB", "∫"))).toBe("b");
  });

  it("ignores macOS Option-remapped characters for other letter keys", () => {
    // Option+S → "ß"
    expect(resolveKey(fakeEvent("KeyS", "ß"))).toBe("s");
  });
});
