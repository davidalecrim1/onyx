import { usePanelStore } from "../stores/panelStore";

interface Props {
  panelId: string;
  tooltip?: string;
}

/// Icon button that toggles a named panel open/closed.
export default function PanelToggleButton({ panelId, tooltip }: Props) {
  const isOpen = usePanelStore(
    (state) => state.panels[panelId]?.isOpen ?? false,
  );
  const toggle = usePanelStore((state) => state.togglePanel);

  return (
    <button
      onClick={() => toggle(panelId)}
      className={`rounded p-1.5 transition-colors hover:bg-surface-hover ${
        isOpen ? "text-text-primary" : "text-text-secondary"
      }`}
      aria-label={tooltip}
      title={tooltip}
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 16 16"
        fill="none"
        aria-hidden="true"
      >
        <rect
          x="1"
          y="2"
          width="14"
          height="12"
          rx="1.5"
          stroke="currentColor"
          strokeWidth="1.2"
        />
        <line
          x1="5.5"
          y1="2"
          x2="5.5"
          y2="14"
          stroke="currentColor"
          strokeWidth="1.2"
        />
      </svg>
    </button>
  );
}
