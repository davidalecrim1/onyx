import { useEffect, useRef, useState } from "react";

interface VaultEntry {
  name: string;
  path: string;
}

interface Props {
  currentVaultName: string;
  currentVaultPath: string;
  vaults: VaultEntry[];
  onSwitch: (path: string, name: string) => void;
  onOpenWelcome: () => void;
}

export default function VaultSwitcher({
  currentVaultName,
  currentVaultPath,
  vaults,
  onSwitch,
  onOpenWelcome,
}: Props) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    function handleOutsideClick(e: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleOutsideClick);
    return () => document.removeEventListener("mousedown", handleOutsideClick);
  }, [open]);

  function handleSelect(vault: VaultEntry) {
    setOpen(false);
    if (vault.path !== currentVaultPath) {
      onSwitch(vault.path, vault.name);
    }
  }

  return (
    <div ref={containerRef} className="relative border-t border-surface">
      {open && (
        <div className="absolute bottom-full left-0 right-0 overflow-hidden rounded-t-md border border-b-0 border-surface bg-background">
          {vaults.map((vault) => (
            <button
              key={vault.path}
              onClick={() => handleSelect(vault)}
              className={`flex w-full items-center gap-2 px-3 py-2 text-left text-sm transition-colors hover:bg-surface ${
                vault.path === currentVaultPath
                  ? "text-text-primary"
                  : "text-text-secondary"
              }`}
            >
              {vault.path === currentVaultPath && (
                <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-accent" />
              )}
              <span
                className={`truncate ${vault.path === currentVaultPath ? "ml-0" : "ml-3.5"}`}
              >
                {vault.name}
              </span>
            </button>
          ))}
          <button
            onClick={() => {
              setOpen(false);
              onOpenWelcome();
            }}
            className="flex w-full items-center gap-2 border-t border-surface px-3 py-2 text-left text-sm text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
          >
            <span className="ml-3.5 truncate">Open another vault...</span>
          </button>
        </div>
      )}
      <button
        onClick={() => setOpen((prev) => !prev)}
        className="relative flex w-full items-center justify-center px-3 py-2 text-sm text-text-secondary transition-colors hover:bg-surface hover:text-text-primary"
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 12 12"
          fill="none"
          className={`absolute right-3 shrink-0 transition-transform ${open ? "rotate-180" : ""}`}
        >
          <path
            d="M2 4l4 4 4-4"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
        <span className="truncate">{currentVaultName}</span>
      </button>
    </div>
  );
}
