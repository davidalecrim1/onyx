import "@testing-library/jest-dom";
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import TabBar from "./TabBar";

const TABS = [
  { path: "/a.md", name: "Alpha" },
  { path: "/b.md", name: "Beta" },
  { path: "/c.md", name: "Gamma" },
];

function renderTabBar(overrides: Partial<Parameters<typeof TabBar>[0]> = {}) {
  const props = {
    tabs: TABS,
    activeTabPath: "/a.md",
    dirtyPaths: new Set<string>(),
    onTabClick: vi.fn(),
    onTabClose: vi.fn(),
    ...overrides,
  };
  return { ...render(<TabBar {...props} />), props };
}

describe("TabBar layout — overflow shrink guard", () => {
  it("root container has flex-1 so it fills available width", () => {
    const { container } = renderTabBar();
    const root = container.firstElementChild as HTMLElement;
    expect(root.className).toContain("flex-1");
  });

  it("root container has min-w-0 to allow shrinking below intrinsic width", () => {
    const { container } = renderTabBar();
    const root = container.firstElementChild as HTMLElement;
    expect(root.className).toContain("min-w-0");
  });

  it("each tab has grow so it expands to fill the container", () => {
    renderTabBar();
    // All tab text spans are inside tab divs — check the parent divs
    TABS.forEach((tab) => {
      const label = screen.getByText(tab.name);
      const tabEl = label.parentElement as HTMLElement;
      expect(tabEl.className).toContain("grow");
    });
  });

  it("each tab has shrink so it compresses when container is too narrow", () => {
    renderTabBar();
    TABS.forEach((tab) => {
      const label = screen.getByText(tab.name);
      const tabEl = label.parentElement as HTMLElement;
      expect(tabEl.className).toContain("shrink");
    });
  });

  it("each tab has max-w-[180px] to cap width when few tabs are open", () => {
    renderTabBar();
    TABS.forEach((tab) => {
      const label = screen.getByText(tab.name);
      const tabEl = label.parentElement as HTMLElement;
      expect(tabEl.className).toContain("max-w-[180px]");
    });
  });

  it("each tab has min-w-[36px] floor so tabs never collapse to zero", () => {
    renderTabBar();
    TABS.forEach((tab) => {
      const label = screen.getByText(tab.name);
      const tabEl = label.parentElement as HTMLElement;
      expect(tabEl.className).toContain("min-w-[36px]");
    });
  });

  it("tab label has truncate so text clips instead of overflowing", () => {
    renderTabBar();
    TABS.forEach((tab) => {
      const label = screen.getByText(tab.name);
      expect(label.className).toContain("truncate");
    });
  });
});

describe("TabBar behavior", () => {
  it("renders all tabs", () => {
    renderTabBar();
    TABS.forEach((tab) => {
      expect(screen.getByText(tab.name)).toBeInTheDocument();
    });
  });

  it("clicking a tab calls onTabClick with the correct path", () => {
    const { props } = renderTabBar();
    fireEvent.click(screen.getByText("Beta").parentElement!);
    expect(props.onTabClick).toHaveBeenCalledWith("/b.md");
  });

  it("clicking the close button calls onTabClose and does not trigger onTabClick", () => {
    const { props } = renderTabBar();
    const closeBtn = screen.getByRole("button", { name: "Close Beta" });
    fireEvent.click(closeBtn);
    expect(props.onTabClose).toHaveBeenCalledWith("/b.md");
    expect(props.onTabClick).not.toHaveBeenCalled();
  });

  it("active tab has active styles", () => {
    renderTabBar({ activeTabPath: "/b.md" });
    const activeLabel = screen.getByText("Beta");
    const activeTab = activeLabel.parentElement as HTMLElement;
    expect(activeTab.className).toContain("bg-surface");
    expect(activeTab.className).toContain("text-text-primary");
  });

  it("inactive tabs do not have active styles", () => {
    renderTabBar({ activeTabPath: "/b.md" });
    const inactiveLabel = screen.getByText("Alpha");
    const inactiveTab = inactiveLabel.parentElement as HTMLElement;
    expect(inactiveTab.className).not.toContain("bg-surface ");
  });

  it("dirty indicator is shown for dirty paths", () => {
    renderTabBar({ dirtyPaths: new Set(["/b.md"]) });
    const closeWrapper = screen
      .getByRole("button", { name: "Close Beta" })
      .parentElement!;
    const dot = closeWrapper.querySelector("span");
    expect(dot).toBeInTheDocument();
  });

  it("dirty indicator is absent for clean paths", () => {
    renderTabBar({ dirtyPaths: new Set() });
    const closeWrapper = screen
      .getByRole("button", { name: "Close Alpha" })
      .parentElement!;
    const dot = closeWrapper.querySelector("span");
    expect(dot).not.toBeInTheDocument();
  });

  it("renders no tabs when tabs array is empty", () => {
    renderTabBar({ tabs: [], activeTabPath: null });
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
