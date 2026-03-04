import { type ReactNode } from "react";
import PanelToggleButton from "./PanelToggleButton";
import ResizablePanel from "./ResizablePanel";

interface Props {
  sidebar: ReactNode;
  tabBar: ReactNode;
  children: ReactNode;
}

/// Top-level layout shell. Add future panels here by extending Props.
export default function AppLayout({ sidebar, tabBar, children }: Props) {
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
        <div className="flex items-center border-b border-surface">
          <PanelToggleButton
            panelId="fileTree"
            tooltip="Toggle sidebar (Cmd+B)"
          />
          {tabBar}
        </div>
        {children}
      </div>
    </div>
  );
}
