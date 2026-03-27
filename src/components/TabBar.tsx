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
      {tabs.map((tab) => {
        const isActive = tab.path === activeTabPath;
        const isDirty = dirtyPaths.has(tab.path);
        return (
          <div
            key={tab.path}
            onClick={() => onTabClick(tab.path)}
            className={`group relative flex min-w-[36px] max-w-[180px] shrink grow cursor-pointer items-center gap-2 rounded-t-lg px-4 py-1.5 text-sm transition-colors ${
              isActive
                ? "bg-surface text-text-primary"
                : "bg-[#1e2128] text-text-secondary hover:bg-surface-hover hover:text-text-primary"
            }`}
          >
            <span className="truncate">{tab.name.replace(/\.md$/i, "")}</span>
            <div className="relative ml-auto flex h-4 w-4 shrink-0 items-center justify-center">
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
  );
}

export type { Tab };
