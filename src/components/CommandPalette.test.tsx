import "@testing-library/jest-dom";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import CommandPalette from "./CommandPalette";
import { useCommandStore } from "../stores/commandStore";

vi.mock("../hooks/useKeybindings", () => ({
  getKeybindingLabel: (id: string) => {
    const bindings: Record<string, string> = {
      "editor.save": "Cmd+S",
    };
    return bindings[id] ?? null;
  },
}));

function seedCommands() {
  const { register } = useCommandStore.getState();
  register({ id: "editor.save", label: "Save File", execute: vi.fn() });
  register({
    id: "view.toggleSidebar",
    label: "Toggle Sidebar",
    execute: vi.fn(),
  });
  register({
    id: "file.newNote",
    label: "New Note",
    keywords: ["create", "document"],
    execute: vi.fn(),
  });
}

beforeEach(() => {
  useCommandStore.setState({ commands: new Map() });
});

describe("CommandPalette", () => {
  it("shows all registered commands when query is empty", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    expect(screen.getByText("Save File")).toBeInTheDocument();
    expect(screen.getByText("Toggle Sidebar")).toBeInTheDocument();
    expect(screen.getByText("New Note")).toBeInTheDocument();
  });

  it("filters by label substring", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "save" },
    });
    expect(screen.getByText("Save File")).toBeInTheDocument();
    expect(screen.queryByText("Toggle Sidebar")).not.toBeInTheDocument();
  });

  it("filters by keyword when label does not match", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "document" },
    });
    expect(screen.getByText("New Note")).toBeInTheDocument();
    expect(screen.queryByText("Save File")).not.toBeInTheDocument();
  });

  it("shows no commands found message when nothing matches", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "xyzzy" },
    });
    expect(screen.getByText("No commands found")).toBeInTheDocument();
  });

  it("ArrowDown navigates down and ArrowUp navigates up", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    const input = screen.getByRole("textbox");
    // Commands are sorted: "New Note", "Save File", "Toggle Sidebar"
    // Initially first item (index 0) is selected. Arrow down to index 1.
    fireEvent.keyDown(input, { key: "ArrowDown" });
    const items = screen.getAllByRole("listitem");
    expect(items[1].className).toContain("bg-surface-hover");
    fireEvent.keyDown(input, { key: "ArrowUp" });
    expect(items[0].className).toContain("bg-surface-hover");
  });

  it("ArrowUp is clamped at index 0", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    const input = screen.getByRole("textbox");
    fireEvent.keyDown(input, { key: "ArrowUp" });
    const items = screen.getAllByRole("listitem");
    expect(items[0].className).toContain("bg-surface-hover");
  });

  it("ArrowDown is clamped at last index", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    const input = screen.getByRole("textbox");
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    const items = screen.getAllByRole("listitem");
    // Three commands, last index = 2
    expect(items[2].className).toContain("bg-surface-hover");
  });

  it("Enter executes selected command and calls onClose", () => {
    const execute = vi.fn();
    useCommandStore
      .getState()
      .register({ id: "test.cmd", label: "Test Command", execute });
    const onClose = vi.fn();
    render(<CommandPalette onClose={onClose} />);
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "Enter" });
    expect(execute).toHaveBeenCalledOnce();
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("Escape calls onClose without executing", () => {
    const execute = vi.fn();
    useCommandStore
      .getState()
      .register({ id: "test.cmd", label: "Test Command", execute });
    const onClose = vi.fn();
    render(<CommandPalette onClose={onClose} />);
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "Escape" });
    expect(execute).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("clicking a row executes and calls onClose", () => {
    const execute = vi.fn();
    useCommandStore
      .getState()
      .register({ id: "test.cmd", label: "Test Command", execute });
    const onClose = vi.fn();
    render(<CommandPalette onClose={onClose} />);
    fireEvent.mouseDown(screen.getByText("Test Command"));
    expect(execute).toHaveBeenCalledOnce();
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("clicking the backdrop calls onClose", () => {
    seedCommands();
    const onClose = vi.fn();
    render(<CommandPalette onClose={onClose} />);
    // The backdrop is the outermost div
    const backdrop = screen.getByRole("textbox").closest('[class*="fixed"]');
    if (!backdrop) throw new Error("Backdrop not found");
    fireEvent.mouseDown(backdrop);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("clicking the modal does not call onClose", () => {
    seedCommands();
    const onClose = vi.fn();
    render(<CommandPalette onClose={onClose} />);
    // The modal inner div wraps the input
    const modal = screen.getByRole("textbox").closest('[class*="rounded-lg"]');
    if (!modal) throw new Error("Modal not found");
    fireEvent.mouseDown(modal);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("displays keybinding hint when available", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    expect(screen.getByText("Cmd+S")).toBeInTheDocument();
  });

  it("input is focused on open", () => {
    seedCommands();
    render(<CommandPalette onClose={vi.fn()} />);
    expect(screen.getByRole("textbox")).toHaveFocus();
  });
});
