interface Tab {
  path: string;
  name: string;
}

interface Props {
  tabs: Tab[];
  activeTabPath: string | null;
  dirtyPaths: Set<string>;
  onTabClick: (path: string) => void;
  onTabClose: (path: string) => void;
}

export default function TabBar({
  tabs,
  activeTabPath,
  dirtyPaths,
  onTabClick,
  onTabClose,
}: Props) {
  return (
    <div
      data-tauri-drag-region
      className="flex h-full min-w-0 flex-1 items-end bg-background px-2"
    >
      <div className="flex min-w-0 overflow-hidden">
        {tabs.map((tab) => {
          const isActive = tab.path === activeTabPath;
          const isDirty = dirtyPaths.has(tab.path);
          return (
            <div
              key={tab.path}
              onClick={() => onTabClick(tab.path)}
              className={`group relative flex min-w-[36px] basis-[180px] shrink grow-0 cursor-pointer items-center gap-2 rounded-t-lg px-4 py-1.5 text-sm transition-colors ${
                isActive
                  ? "bg-surface text-text-primary"
                  : "text-text-secondary hover:bg-surface-hover hover:text-text-primary"
              }`}
            >
              <span className="truncate">{tab.name}</span>
              <div className="relative flex h-4 w-4 shrink-0 items-center justify-center">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onTabClose(tab.path);
                  }}
                  className="absolute inset-0 flex items-center justify-center opacity-0 transition-opacity hover:text-text-primary group-hover:opacity-100"
                  aria-label={`Close ${tab.name}`}
                >
                  ×
                </button>
                {isDirty && (
                  <span className="h-2 w-2 rounded-full bg-text-secondary transition-opacity group-hover:opacity-0" />
                )}
              </div>
            </div>
          );
        })}
      </div>
      {/* Fills remaining space; this is what the user actually drags to move the window. */}
      <div data-tauri-drag-region className="min-w-0 flex-1 cursor-default" />
    </div>
  );
}

export type { Tab };
