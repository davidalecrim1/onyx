import { useCallback, useRef, type ReactNode } from "react";
import { usePanelStore } from "../stores/panelStore";

interface Props {
  panelId: string;
  side: "left" | "right" | "bottom";
  children: ReactNode;
  className?: string;
}

/// Wraps content in a panel resizable by dragging its inner edge.
export default function ResizablePanel({
  panelId,
  side,
  children,
  className = "",
}: Props) {
  const panel = usePanelStore((state) => state.panels[panelId]);
  const resize = usePanelStore((state) => state.resize);
  const dragRef = useRef<{ startPos: number; startSize: number } | null>(null);

  const handlePointerDown = useCallback(
    (event: React.PointerEvent<HTMLDivElement>) => {
      event.preventDefault();
      const startPos = side === "bottom" ? event.clientY : event.clientX;
      const startSize = side === "bottom" ? panel.height : panel.width;
      dragRef.current = { startPos, startSize };

      const target = event.currentTarget;
      target.setPointerCapture(event.pointerId);

      function onPointerMove(moveEvent: PointerEvent) {
        if (!dragRef.current) return;
        const currentPos =
          side === "bottom" ? moveEvent.clientY : moveEvent.clientX;
        const delta = currentPos - dragRef.current.startPos;
        const direction = side === "right" || side === "bottom" ? -1 : 1;
        resize(panelId, dragRef.current.startSize + delta * direction);
      }

      function onPointerUp(upEvent: PointerEvent) {
        dragRef.current = null;
        target.removeEventListener("pointermove", onPointerMove);
        target.removeEventListener("pointerup", onPointerUp);
        target.releasePointerCapture(upEvent.pointerId);
      }

      target.addEventListener("pointermove", onPointerMove);
      target.addEventListener("pointerup", onPointerUp);
    },
    [panelId, panel, side, resize],
  );

  if (!panel?.isOpen) return null;

  const sizeStyle =
    side === "left" || side === "right"
      ? { width: `${panel.width}px` }
      : { height: `${panel.height}px` };

  const handleClass =
    side === "left"
      ? "right-0 top-0 bottom-0 w-1 cursor-col-resize"
      : side === "right"
        ? "left-0 top-0 bottom-0 w-1 cursor-col-resize"
        : "top-0 left-0 right-0 h-1 cursor-row-resize";

  return (
    <div className={`relative shrink-0 ${className}`} style={sizeStyle}>
      {children}
      <div
        onPointerDown={handlePointerDown}
        className={`absolute ${handleClass} z-10 transition-colors hover:bg-surface-active`}
      />
    </div>
  );
}
