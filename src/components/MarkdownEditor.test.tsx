import "@testing-library/jest-dom";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import MarkdownEditor from "./MarkdownEditor";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

vi.mock("@uiw/react-codemirror", () => ({
  default: vi.fn(() => <div data-testid="codemirror" />),
  EditorView: { theme: vi.fn(() => ({})), lineWrapping: {} },
  EditorSelection: { cursor: vi.fn() },
}));

vi.mock("@codemirror/lang-markdown", () => ({
  markdown: vi.fn(() => ({})),
}));

vi.mock("@lezer/markdown", () => ({
  GFM: {},
  Strikethrough: {},
  TaskList: {},
}));

vi.mock("@replit/codemirror-vim", () => ({
  vim: vi.fn(() => ({})),
}));

vi.mock("../extensions/markdownDecorations", () => ({
  markdownDecorations: {},
}));

vi.mock("../extensions/tagAutocomplete", () => ({
  tagAutocomplete: vi.fn(() => ({})),
}));

vi.mock("../extensions/wikilinkDecorations", () => ({
  setWikilinkConfig: { of: vi.fn() },
  wikilinkConfigField: {},
  wikilinkViewPlugin: {},
}));

vi.mock("../extensions/imageDecorations", () => ({
  imageConfigField: {},
  setImageConfig: { of: vi.fn() },
  imageDecorations: [],
}));

const DEFAULT_PROPS = {
  content: "",
  onChange: vi.fn(),
  vimMode: false,
  filePath: "/vault/test.md",
  onRename: vi.fn(),
  vaultPath: "/vault",
  onWikilinkOpen: vi.fn(),
  onWikilinkCreate: vi.fn(),
};

function switchToReadingMode() {
  fireEvent.click(screen.getByTitle("Switch to reading mode"));
}

describe("MarkdownEditor reading mode — wikilinks", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue([]);
  });

  it("renders [[note]] as a clickable wikilink span", () => {
    render(<MarkdownEditor {...DEFAULT_PROPS} content="See [[other note]]" />);
    switchToReadingMode();

    const link = screen.getByText("other note");
    expect(link).toHaveClass("onyx-wikilink");
    expect(link).toHaveAttribute("data-target", "other note");
  });

  it("renders [[note|alias]] using alias text", () => {
    render(
      <MarkdownEditor
        {...DEFAULT_PROPS}
        content="See [[meeting notes|Notes]]"
      />,
    );
    switchToReadingMode();

    const link = screen.getByText("Notes");
    expect(link).toHaveClass("onyx-wikilink");
    expect(link).toHaveAttribute("data-target", "meeting notes");
  });

  it("opens the resolved file when clicking an existing wikilink", async () => {
    const onWikilinkOpen = vi.fn();
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === "resolve_wikilink")
        return Promise.resolve("/vault/other note.md");
      return Promise.resolve([]);
    });

    render(
      <MarkdownEditor
        {...DEFAULT_PROPS}
        content="[[other note]]"
        onWikilinkOpen={onWikilinkOpen}
      />,
    );
    switchToReadingMode();
    fireEvent.click(screen.getByText("other note"));

    await waitFor(() => {
      expect(onWikilinkOpen).toHaveBeenCalledWith("/vault/other note.md");
    });
  });

  it("creates the file when clicking a wikilink that does not exist", async () => {
    const onWikilinkCreate = vi.fn();
    vi.mocked(invoke).mockImplementation((cmd) => {
      if (cmd === "resolve_wikilink") return Promise.resolve(null);
      return Promise.resolve([]);
    });

    render(
      <MarkdownEditor
        {...DEFAULT_PROPS}
        content="[[new note]]"
        onWikilinkCreate={onWikilinkCreate}
      />,
    );
    switchToReadingMode();
    fireEvent.click(screen.getByText("new note"));

    await waitFor(() => {
      expect(onWikilinkCreate).toHaveBeenCalledWith("new note");
    });
  });
});
