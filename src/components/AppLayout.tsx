import { type ReactNode } from "react";
import PanelToggleButton from "./PanelToggleButton";
import ResizablePanel from "./ResizablePanel";
import { usePanelStore } from "../stores/panelStore";
import { getKeybindingLabel } from "../hooks/useKeybindings";

const TRAFFIC_LIGHT_WIDTH = 156;
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
          className="flex shrink-0 border-b border-surface"
          style={{
            height: TRAFFIC_LIGHT_HEIGHT + 8,
            paddingLeft: sidebarOpen ? 0 : TRAFFIC_LIGHT_WIDTH,
          }}
        >
          <div className="flex items-center px-1">
            <PanelToggleButton
              panelId="fileTree"
              tooltip={`Toggle sidebar (${getKeybindingLabel("view.toggleSidebar") ?? "unbound"})`}
            />
          </div>
          <div className="flex flex-1 items-end pt-2">{tabBar}</div>
        </div>

        {children}
      </div>
    </div>
  );
}
