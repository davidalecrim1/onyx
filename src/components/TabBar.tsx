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
  if (tabs.length === 0) return null;

  return (
    <div className="flex overflow-x-auto border-b border-surface bg-background px-2 pt-1.5">
      {tabs.map((tab) => {
        const isActive = tab.path === activeTabPath;
        const isDirty = dirtyPaths.has(tab.path);
        return (
          <div
            key={tab.path}
            className={`group relative flex shrink-0 items-center gap-2 rounded-t-lg px-4 py-1.5 text-sm transition-colors ${
              isActive
                ? "bg-surface text-text-primary"
                : "text-text-secondary hover:bg-surface-hover hover:text-text-primary"
            }`}
          >
            <button onClick={() => onTabClick(tab.path)}>{tab.name}</button>
            <div className="relative flex h-4 w-4 items-center justify-center">
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
