import "@testing-library/jest-dom";
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import FilePicker from "./FilePicker";
import type { FileTreeEntry } from "./FileTree";

function makeFile(name: string, path: string): FileTreeEntry {
  return {
    name,
    path,
    is_directory: false,
    depth: 0,
    children: [],
    modified_secs: 0,
    created_secs: 0,
  };
}

const DEFAULT_PROPS = {
  files: [],
  onOpen: vi.fn(),
  onCreate: vi.fn(),
  onClose: vi.fn(),
};

describe("FilePicker — .md extension stripping", () => {
  it("strips .md from file names in the list", () => {
    render(
      <FilePicker
        {...DEFAULT_PROPS}
        files={[makeFile("notes.md", "/vault/notes.md")]}
      />,
    );
    expect(screen.getByText("notes")).toBeInTheDocument();
    expect(screen.queryByText("notes.md")).not.toBeInTheDocument();
  });

  it("preserves non-.md extensions", () => {
    render(
      <FilePicker
        {...DEFAULT_PROPS}
        files={[
          makeFile("diagram.png", "/vault/diagram.png"),
          makeFile("spec.pdf", "/vault/spec.pdf"),
        ]}
      />,
    );
    expect(screen.getByText("diagram.png")).toBeInTheDocument();
    expect(screen.getByText("spec.pdf")).toBeInTheDocument();
  });

  it("handles .MD uppercase extension", () => {
    render(
      <FilePicker
        {...DEFAULT_PROPS}
        files={[makeFile("README.MD", "/vault/README.MD")]}
      />,
    );
    expect(screen.getByText("README")).toBeInTheDocument();
  });

  it("strips .md from files in subdirectories", () => {
    const dirEntry: FileTreeEntry = {
      name: "docs",
      path: "/vault/docs",
      is_directory: true,
      depth: 0,
      children: [makeFile("guide.md", "/vault/docs/guide.md")],
      modified_secs: 0,
      created_secs: 0,
    };
    render(<FilePicker {...DEFAULT_PROPS} files={[dirEntry]} />);
    expect(screen.getByText("guide")).toBeInTheDocument();
    expect(screen.queryByText("guide.md")).not.toBeInTheDocument();
  });
});
