import { type ReactNode } from "react";
import PanelToggleButton from "./PanelToggleButton";
import ResizablePanel from "./ResizablePanel";
import { usePanelStore } from "../stores/panelStore";
import { getKeybindingLabel } from "../hooks/useKeybindings";

const TRAFFIC_LIGHT_WIDTH = 78;
const TRAFFIC_LIGHT_HEIGHT = 38;

interface Props {
  sidebar: ReactNode;
  tabBar: ReactNode;
  children: ReactNode;
}

/// Top-level layout shell. Add future panels here by extending Props.
export default function AppLayout({ sidebar, tabBar, children }: Props) {
  const sidebarOpen = usePanelStore(
    (state) => state.panels["fileTree"]?.isOpen ?? false,
  );

  return (
    <div className="flex h-full bg-background text-text-primary">
      <ResizablePanel
        panelId="fileTree"
        side="left"
        className="flex flex-col border-r border-surface"
      >
        {sidebar}
      </ResizablePanel>

      <div className="flex flex-1 flex-col overflow-hidden">
        <div
          className="flex items-center border-b border-surface"
          style={{ paddingTop: TRAFFIC_LIGHT_HEIGHT }}
        >
          {!sidebarOpen && (
            <div
              className="shrink-0"
              style={{ width: TRAFFIC_LIGHT_WIDTH }}
              aria-hidden="true"
            />
          )}
          <PanelToggleButton
            panelId="fileTree"
            tooltip={`Toggle sidebar (${getKeybindingLabel("view.toggleSidebar") ?? "unbound"})`}
          />
          {tabBar}
        </div>
        {children}
      </div>
    </div>
  );
}
