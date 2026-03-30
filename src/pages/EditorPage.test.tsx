import { describe, it, expect } from "vitest";
import { sortFileTree } from "../utils/fileSort";
import type { FileTreeEntry } from "../components/FileTree";

function makeFile(
  name: string,
  modified_secs = 0,
  created_secs = 0,
): FileTreeEntry {
  return {
    name,
    path: `/vault/${name}`,
    is_directory: false,
    depth: 0,
    children: [],
    modified_secs,
    created_secs,
  };
}

function makeDir(name: string, children: FileTreeEntry[] = []): FileTreeEntry {
  return {
    name,
    path: `/vault/${name}`,
    is_directory: true,
    depth: 0,
    children,
    modified_secs: 0,
    created_secs: 0,
  };
}

describe("sortFileTree", () => {
  it("sorts by name ascending, case-insensitive", () => {
    const result = sortFileTree(
      [makeFile("Zebra.md"), makeFile("apple.md"), makeFile("mango.md")],
      "name-asc",
    );
    expect(result.map((e) => e.name)).toEqual(["apple.md", "mango.md", "Zebra.md"]);
  });

  it("sorts by name descending", () => {
    const result = sortFileTree(
      [makeFile("apple.md"), makeFile("zebra.md"), makeFile("mango.md")],
      "name-desc",
    );
    expect(result.map((e) => e.name)).toEqual(["zebra.md", "mango.md", "apple.md"]);
  });

  it("sorts by modified time newest first", () => {
    const result = sortFileTree(
      [makeFile("old.md", 100), makeFile("new.md", 300), makeFile("mid.md", 200)],
      "modified-desc",
    );
    expect(result.map((e) => e.name)).toEqual(["new.md", "mid.md", "old.md"]);
  });

  it("sorts by modified time oldest first", () => {
    const result = sortFileTree(
      [makeFile("new.md", 300), makeFile("old.md", 100), makeFile("mid.md", 200)],
      "modified-asc",
    );
    expect(result.map((e) => e.name)).toEqual(["old.md", "mid.md", "new.md"]);
  });

  it("sorts by created time newest first", () => {
    const result = sortFileTree(
      [makeFile("a.md", 0, 50), makeFile("b.md", 0, 200), makeFile("c.md", 0, 100)],
      "created-desc",
    );
    expect(result.map((e) => e.name)).toEqual(["b.md", "c.md", "a.md"]);
  });

  it("sorts by created time oldest first", () => {
    const result = sortFileTree(
      [makeFile("b.md", 0, 200), makeFile("a.md", 0, 50), makeFile("c.md", 0, 100)],
      "created-asc",
    );
    expect(result.map((e) => e.name)).toEqual(["a.md", "c.md", "b.md"]);
  });

  it("always places directories before files", () => {
    const result = sortFileTree(
      [makeFile("zebra.md"), makeDir("alpha"), makeFile("apple.md")],
      "name-asc",
    );
    expect(result[0].name).toBe("alpha");
    expect(result[0].is_directory).toBe(true);
  });

  it("recursively sorts children inside directories", () => {
    const dir = makeDir("notes", [makeFile("zebra.md"), makeFile("apple.md")]);
    const result = sortFileTree([dir], "name-asc");
    expect(result[0].children.map((c) => c.name)).toEqual(["apple.md", "zebra.md"]);
  });
});
