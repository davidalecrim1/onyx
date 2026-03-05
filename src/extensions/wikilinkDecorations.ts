import { invoke } from "@tauri-apps/api/core";
import { RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import {
  Decoration,
  DecorationSet,
  EditorView,
  ViewPlugin,
  ViewUpdate,
} from "@codemirror/view";

const WIKILINK_RE = /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g;

export interface WikilinkConfig {
  vaultPath: string;
  onOpen: (path: string) => void;
  onCreate: (linkTarget: string) => void;
}

export const setWikilinkConfig = StateEffect.define<WikilinkConfig>();

export const wikilinkConfigField = StateField.define<WikilinkConfig | null>({
  create: () => null,
  update(value, transaction) {
    for (const effect of transaction.effects) {
      if (effect.is(setWikilinkConfig)) return effect.value;
    }
    return value;
  },
});

const linkMark = Decoration.mark({ class: "onyx-link" });
const hideMark = Decoration.replace({});

export const wikilinkViewPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = this._buildDecorations(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet || update.viewportChanged) {
        this.decorations = this._buildDecorations(update.view);
      }
    }

    _buildDecorations(view: EditorView): DecorationSet {
      try {
        const { state } = view;
        const cursorLines = new Set(
          state.selection.ranges.map(
            (range) => state.doc.lineAt(range.head).number,
          ),
        );

        type Entry = { from: number; to: number; value: Decoration };
        const collected: Entry[] = [];

        for (const { from, to } of view.visibleRanges) {
          let match: RegExpExecArray | null;
          WIKILINK_RE.lastIndex = 0;
          const text = state.doc.sliceString(from, to);
          while ((match = WIKILINK_RE.exec(text)) !== null) {
            const matchFrom = from + match.index;
            const matchTo = matchFrom + match[0].length;
            const lineNumber = state.doc.lineAt(matchFrom).number;
            const cursorIsHere = cursorLines.has(lineNumber);
            const noteName = match[1];
            const alias = match[2];

            if (cursorIsHere) {
              collected.push({ from: matchFrom, to: matchTo, value: linkMark });
            } else {
              const openEnd = matchFrom + 2;
              const closeStart = matchTo - 2;
              collected.push({ from: matchFrom, to: matchTo, value: linkMark });
              collected.push({
                from: matchFrom,
                to: openEnd,
                value: hideMark,
              });
              if (alias !== undefined) {
                const pipePos = matchFrom + 2 + noteName.length;
                collected.push({
                  from: openEnd,
                  to: pipePos + 1,
                  value: hideMark,
                });
              }
              collected.push({
                from: closeStart,
                to: matchTo,
                value: hideMark,
              });
            }
          }
        }

        collected.sort((a, b) => a.from - b.from || a.to - b.to);

        const builder = new RangeSetBuilder<Decoration>();
        for (const { from, to, value } of collected) {
          builder.add(from, to, value);
        }
        return builder.finish();
      } catch (error) {
        console.error("wikilinkDecorations: _buildDecorations failed", error);
        return Decoration.none;
      }
    }
  },
  {
    decorations: (plugin) => plugin.decorations,
    eventHandlers: {
      mousedown(event: MouseEvent, view: EditorView) {
        const config = view.state.field(wikilinkConfigField);
        if (!config) return;

        const coords = { x: event.clientX, y: event.clientY };
        const pos = view.posAtCoords(coords);
        if (pos === null) return;

        // Check if the click position is covered by a wikilink decoration
        let isOnWikilink = false;
        this.decorations.between(pos, pos, () => {
          isOnWikilink = true;
        });
        if (!isOnWikilink) return;

        // Find the wikilink at this position to extract the link target
        const line = view.state.doc.lineAt(pos);
        WIKILINK_RE.lastIndex = 0;
        let clickMatch: RegExpExecArray | null;
        while ((clickMatch = WIKILINK_RE.exec(line.text)) !== null) {
          const matchFrom = line.from + clickMatch.index;
          const matchTo = matchFrom + clickMatch[0].length;
          if (pos >= matchFrom && pos <= matchTo) {
            const linkTarget = clickMatch[1];
            event.preventDefault();
            invoke<string | null>("resolve_wikilink", {
              vaultPath: config.vaultPath,
              linkTarget,
            })
              .then((resolvedPath) => {
                if (resolvedPath !== null) {
                  config.onOpen(resolvedPath);
                } else {
                  config.onCreate(linkTarget);
                }
              })
              .catch((err) => console.error("resolve_wikilink failed:", err));
            return;
          }
        }
      },
    },
  },
);
