import { ViewPlugin, type ViewUpdate } from "@codemirror/view";

/// Scrolls the nearest scrollable ancestor to keep the cursor visible.
/// CodeMirror's built-in scrollIntoView is a no-op when .cm-scroller has
/// overflow:visible, so we drive the parent container directly.
export const parentScrollIntoView = ViewPlugin.fromClass(
  class {
    update(update: ViewUpdate) {
      if (!update.selectionSet && !update.docChanged) return;

      const view = update.view;
      const scroller = findScrollParent(view.dom);
      if (!scroller) return;

      // coordsAtPos returns null for off-screen positions, so we compute
      // the cursor's offset from the top of the editor DOM element instead.
      const head = view.state.selection.main.head;
      const editorTop = view.dom.getBoundingClientRect().top + scroller.scrollTop - scroller.getBoundingClientRect().top;
      const lineInfo = view.lineBlockAt(head);
      const cursorTop = editorTop + lineInfo.top;
      const cursorBottom = editorTop + lineInfo.bottom;

      const visibleTop = scroller.scrollTop;
      const visibleBottom = scroller.scrollTop + scroller.clientHeight;
      const margin = 40;

      if (cursorTop < visibleTop + margin) {
        scroller.scrollTop = Math.max(0, cursorTop - margin);
      } else if (cursorBottom > visibleBottom - margin) {
        scroller.scrollTop = cursorBottom - scroller.clientHeight + margin;
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
