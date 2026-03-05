import { ViewPlugin, type ViewUpdate } from "@codemirror/view";

/// Scrolls the nearest scrollable ancestor to keep the cursor visible,
/// since the editor uses overflow:visible on .cm-scroller.
export const parentScrollIntoView = ViewPlugin.fromClass(
  class {
    update(update: ViewUpdate) {
      if (!update.selectionSet && !update.docChanged) return;

      const view = update.view;
      const head = view.state.selection.main.head;
      const coords = view.coordsAtPos(head);
      if (!coords) return;

      const scroller = findScrollParent(view.dom);
      if (!scroller) return;

      const scrollerRect = scroller.getBoundingClientRect();
      const margin = 40;

      if (coords.top < scrollerRect.top + margin) {
        scroller.scrollTop -= scrollerRect.top + margin - coords.top;
      } else if (coords.bottom > scrollerRect.bottom - margin) {
        scroller.scrollTop += coords.bottom - (scrollerRect.bottom - margin);
      }
    }
  },
);

function findScrollParent(element: HTMLElement): HTMLElement | null {
  let current = element.parentElement;
  while (current) {
    const overflow = getComputedStyle(current).overflowY;
    if (overflow === "auto" || overflow === "scroll") return current;
    current = current.parentElement;
  }
  return null;
}
