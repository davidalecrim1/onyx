import "@testing-library/jest-dom";
import { describe, it, expect, vi } from "vitest";
import { render } from "@testing-library/react";
import AppLayout from "./AppLayout";

vi.mock("./PanelToggleButton", () => ({
  default: () => <button />,
}));

vi.mock("./ResizablePanel", () => ({
  default: ({
    children,
    panelId,
  }: {
    children: React.ReactNode;
    panelId: string;
  }) => <div data-testid={`panel-${panelId}`}>{children}</div>,
}));

vi.mock("../stores/panelStore", () => ({
  usePanelStore: vi.fn((selector) =>
    selector({ panels: { fileTree: { isOpen: true } } }),
  ),
}));

vi.mock("../hooks/useKeybindings", () => ({
  getKeybindingLabel: vi.fn(() => null),
}));

function renderLayout() {
  return render(
    <AppLayout
      sidebar={<div>sidebar</div>}
      tabBar={<div>tabs</div>}
      outlinePanel={<div>outline</div>}
    >
      <div data-testid="content">content</div>
    </AppLayout>,
  );
}

// These tests pin the CSS classes that keep the editor scroll region bounded.
// If any of these fail, trackpad scrolling in the editor will break.
describe("AppLayout — scroll height chain", () => {
  it("root has h-full so the height chain starts from a definite value", () => {
    const { container } = renderLayout();
    const root = container.firstElementChild as HTMLElement;
    expect(root.className).toContain("h-full");
  });

  it("content column has overflow-hidden to prevent vertical overflow", () => {
    const { container } = renderLayout();
    const root = container.firstElementChild as HTMLElement;
    const column = root.children[1] as HTMLElement;
    expect(column.className).toContain("overflow-hidden");
  });

  it("content row (below tab bar) has flex-1 and min-h-0 to bound height for children", () => {
    const { container } = renderLayout();
    const root = container.firstElementChild as HTMLElement;
    const column = root.children[1] as HTMLElement;
    // last child of the column is the content row (tab bar is first)
    const contentRow = column.lastElementChild as HTMLElement;
    expect(contentRow.className).toContain("flex-1");
    expect(contentRow.className).toContain("min-h-0");
    expect(contentRow.className).toContain("overflow-hidden");
  });

  it("children wrapper inside content row has min-h-0 to propagate bounded height", () => {
    const { container } = renderLayout();
    const root = container.firstElementChild as HTMLElement;
    const column = root.children[1] as HTMLElement;
    const contentRow = column.lastElementChild as HTMLElement;
    // first child of the row is the children wrapper (second is outline panel)
    const childrenWrapper = contentRow.firstElementChild as HTMLElement;
    expect(childrenWrapper.className).toContain("flex-1");
    expect(childrenWrapper.className).toContain("min-h-0");
    expect(childrenWrapper.className).toContain("overflow-hidden");
  });
});
