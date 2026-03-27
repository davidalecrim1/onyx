import "@testing-library/jest-dom";
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import FileTree, { type FileTreeEntry } from "./FileTree";

function makeFile(name: string, path: string, depth = 0): FileTreeEntry {
  return {
    name,
    path,
    is_directory: false,
    depth,
    children: [],
    modified_secs: 0,
    created_secs: 0,
  };
}

function makeDir(
  name: string,
  path: string,
  children: FileTreeEntry[],
  depth = 0,
): FileTreeEntry {
  return {
    name,
    path,
    is_directory: true,
    depth,
    children,
    modified_secs: 0,
    created_secs: 0,
  };
}

const DEFAULT_PROPS = {
  entries: [],
  activeFilePath: null,
  vaultPath: "/vault",
  onFileClick: vi.fn(),
};

describe("FileTree — .md extension stripping", () => {
  it("strips .md from file names in the sidebar", () => {
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={[makeFile("notes.md", "/vault/notes.md")]}
      />,
    );
    expect(screen.getByText("notes")).toBeInTheDocument();
    expect(screen.queryByText("notes.md")).not.toBeInTheDocument();
  });

  it("preserves non-.md extensions", () => {
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={[
          makeFile("image.png", "/vault/image.png"),
          makeFile("report.pdf", "/vault/report.pdf"),
        ]}
      />,
    );
    expect(screen.getByText("image.png")).toBeInTheDocument();
    expect(screen.getByText("report.pdf")).toBeInTheDocument();
  });

  it("does not strip .md from directory names", () => {
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={[makeDir("archive", "/vault/archive", [])]}
      />,
    );
    expect(screen.getByText("archive")).toBeInTheDocument();
  });

  it("strips .md from files inside nested directories", () => {
    const nested = makeDir("docs", "/vault/docs", [
      makeFile("guide.md", "/vault/docs/guide.md", 1),
    ]);
    render(<FileTree {...DEFAULT_PROPS} entries={[nested]} />);
    expect(screen.getByText("guide")).toBeInTheDocument();
    expect(screen.queryByText("guide.md")).not.toBeInTheDocument();
  });

  it("handles files with .MD uppercase extension", () => {
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={[makeFile("README.MD", "/vault/README.MD")]}
      />,
    );
    expect(screen.getByText("README")).toBeInTheDocument();
  });
});
