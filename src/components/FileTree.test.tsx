import "@testing-library/jest-dom";
import { createRef } from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import FileTree, { type FileTreeEntry, type FileTreeHandle } from "./FileTree";

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

beforeEach(() => {
  vi.clearAllMocks();
});

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

describe("FileTree — vim keyboard navigation", () => {
  const twoFiles = [
    makeFile("alpha.md", "/vault/alpha.md"),
    makeFile("beta.md", "/vault/beta.md"),
  ];

  function getTree() {
    return screen.getByRole("tree");
  }

  it("focus() method focuses the container element", () => {
    const ref = createRef<FileTreeHandle>();
    render(<FileTree ref={ref} {...DEFAULT_PROPS} entries={twoFiles} />);
    ref.current?.focus();
    expect(document.activeElement).toBe(getTree());
  });

  it("ArrowDown moves focus to the first item when nothing is selected", () => {
    render(<FileTree {...DEFAULT_PROPS} entries={twoFiles} />);
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    expect(tree.querySelector("[data-focused='true']")).toHaveTextContent(
      "alpha",
    );
  });

  it("ArrowDown moves focused path to the next item", () => {
    render(<FileTree {...DEFAULT_PROPS} entries={twoFiles} />);
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    expect(tree.querySelector("[data-focused='true']")).toHaveTextContent(
      "beta",
    );
  });

  it("ArrowUp moves focused path to the previous item", () => {
    render(<FileTree {...DEFAULT_PROPS} entries={twoFiles} />);
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "ArrowUp" });
    expect(tree.querySelector("[data-focused='true']")).toHaveTextContent(
      "alpha",
    );
  });

  it("ArrowDown does not go past the last item", () => {
    render(<FileTree {...DEFAULT_PROPS} entries={twoFiles} />);
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    expect(tree.querySelector("[data-focused='true']")).toHaveTextContent(
      "beta",
    );
  });

  it("ArrowUp does not go before the first item", () => {
    render(<FileTree {...DEFAULT_PROPS} entries={twoFiles} />);
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "ArrowUp" });
    fireEvent.keyDown(tree, { key: "ArrowUp" });
    expect(tree.querySelector("[data-focused='true']")).toHaveTextContent(
      "alpha",
    );
  });

  it("Space calls onFileClick with the focused path", () => {
    const onFileClick = vi.fn();
    render(
      <FileTree
        {...DEFAULT_PROPS}
        onFileClick={onFileClick}
        entries={twoFiles}
      />,
    );
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: " " });
    expect(onFileClick).toHaveBeenCalledWith("/vault/alpha.md");
  });

  it("Enter calls onFileClick with the focused path", () => {
    const onFileClick = vi.fn();
    render(
      <FileTree
        {...DEFAULT_PROPS}
        onFileClick={onFileClick}
        entries={twoFiles}
      />,
    );
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "Enter" });
    expect(onFileClick).toHaveBeenCalledWith("/vault/alpha.md");
  });

  it("a key calls onCreateFile with the vault root when nothing is focused", () => {
    const onCreateFile = vi.fn();
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={twoFiles}
        onCreateFile={onCreateFile}
      />,
    );
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "a" });
    expect(onCreateFile).toHaveBeenCalledWith("/vault");
  });

  it("a key calls onCreateFile with parent dir of focused file", () => {
    const onCreateFile = vi.fn();
    const entries = [
      makeDir("docs", "/vault/docs", [
        makeFile("guide.md", "/vault/docs/guide.md", 1),
      ]),
    ];
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={entries}
        onCreateFile={onCreateFile}
      />,
    );
    const tree = getTree();
    tree.focus();
    // ArrowDown focuses "docs" dir, ArrowDown again focuses "guide.md"
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "a" });
    expect(onCreateFile).toHaveBeenCalledWith("/vault/docs");
  });

  it("A key calls onCreateFolder", () => {
    const onCreateFolder = vi.fn();
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={twoFiles}
        onCreateFolder={onCreateFolder}
      />,
    );
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "A", shiftKey: true });
    expect(onCreateFolder).toHaveBeenCalledWith("/vault");
  });

  it("d key calls onDelete with the focused path", () => {
    const onDelete = vi.fn();
    render(
      <FileTree {...DEFAULT_PROPS} entries={twoFiles} onDelete={onDelete} />,
    );
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "d" });
    expect(onDelete).toHaveBeenCalledWith("/vault/alpha.md", false);
  });

  it("r key shows inline rename input with the file stem", () => {
    render(<FileTree {...DEFAULT_PROPS} entries={twoFiles} />);
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "r" });
    const input = screen.getByRole("textbox");
    expect(input).toBeInTheDocument();
    expect((input as HTMLInputElement).value).toBe("alpha");
  });

  it("confirming rename calls onRenameFile with path and new stem", () => {
    const onRenameFile = vi.fn();
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={twoFiles}
        onRenameFile={onRenameFile}
      />,
    );
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "r" });
    const input = screen.getByRole("textbox") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "renamed" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onRenameFile).toHaveBeenCalledWith("/vault/alpha.md", "renamed");
  });

  it("Escape on rename input cancels without calling onRenameFile", () => {
    const onRenameFile = vi.fn();
    render(
      <FileTree
        {...DEFAULT_PROPS}
        entries={twoFiles}
        onRenameFile={onRenameFile}
      />,
    );
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    fireEvent.keyDown(tree, { key: "r" });
    const input = screen.getByRole("textbox");
    fireEvent.keyDown(input, { key: "Escape" });
    expect(onRenameFile).not.toHaveBeenCalled();
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("Escape on tree container clears focused path", () => {
    render(<FileTree {...DEFAULT_PROPS} entries={twoFiles} />);
    const tree = getTree();
    tree.focus();
    fireEvent.keyDown(tree, { key: "ArrowDown" });
    expect(tree.querySelector("[data-focused='true']")).toBeInTheDocument();
    fireEvent.keyDown(tree, { key: "Escape" });
    expect(tree.querySelector("[data-focused='true']")).not.toBeInTheDocument();
  });
});
