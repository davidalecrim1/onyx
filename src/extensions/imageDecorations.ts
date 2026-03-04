import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  RangeSetBuilder,
  StateEffect,
  StateField,
  Transaction,
} from "@codemirror/state";
import {
  Decoration,
  DecorationSet,
  EditorView,
  ViewPlugin,
  ViewUpdate,
  WidgetType,
} from "@codemirror/view";

const WIKIIMAGE_RE = /!\[\[([^\]]+)\]\]/g;
const MDIMAGE_RE = /!\[([^\]]*)\]\(([^)]+)\)/g;

export interface ImageConfig {
  vaultPath: string;
  filePath: string;
}

export const setImageConfig = StateEffect.define<ImageConfig>();

export const imageConfigField = StateField.define<ImageConfig | null>({
  create: () => null,
  update(value, transaction: Transaction) {
    for (const effect of transaction.effects) {
      if (effect.is(setImageConfig)) return effect.value;
    }
    return value;
  },
});

const triggerRedraw = StateEffect.define<null>();

class ImageWidget extends WidgetType {
  constructor(private readonly src: string) {
    super();
  }

  eq(other: ImageWidget): boolean {
    return this.src === other.src;
  }

  toDOM(): HTMLElement {
    const img = document.createElement("img");
    img.className = "onyx-image";
    img.src = this.src;
    img.onerror = () => {
      const broken = document.createElement("div");
      broken.className = "onyx-image-broken";
      broken.textContent = "[Image not found]";
      img.replaceWith(broken);
    };
    return img;
  }
}

export const imageViewPlugin = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    // Resolved src URLs keyed by the raw specifier (path/URL in the markdown)
    private readonly resolved = new Map<string, string>();

    constructor(view: EditorView) {
      this.decorations = this._buildDecorations(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet || update.viewportChanged) {
        this.decorations = this._buildDecorations(update.view);
        return;
      }
      for (const effect of update.transactions.flatMap((t) => t.effects)) {
        if (effect.is(triggerRedraw)) {
          this.decorations = this._buildDecorations(update.view);
          break;
        }
      }
    }

    _buildDecorations(view: EditorView): DecorationSet {
      const { state } = view;
      const config = state.field(imageConfigField);
      const cursorLines = new Set(
        state.selection.ranges.map(
          (range) => state.doc.lineAt(range.head).number,
        ),
      );

      type Entry = { from: number; to: number; value: Decoration };
      const collected: Entry[] = [];

      for (const { from, to } of view.visibleRanges) {
        const text = state.doc.sliceString(from, to);

        WIKIIMAGE_RE.lastIndex = 0;
        let match: RegExpExecArray | null;
        while ((match = WIKIIMAGE_RE.exec(text)) !== null) {
          const matchFrom = from + match.index;
          const matchTo = matchFrom + match[0].length;
          const lineNumber = state.doc.lineAt(matchFrom).number;
          if (cursorLines.has(lineNumber)) continue;

          const specifier = match[1].trim();
          this._resolveAndDecorate(view, specifier, matchFrom, matchTo, config, "wikilink", collected);
        }

        MDIMAGE_RE.lastIndex = 0;
        while ((match = MDIMAGE_RE.exec(text)) !== null) {
          const matchFrom = from + match.index;
          const matchTo = matchFrom + match[0].length;
          const lineNumber = state.doc.lineAt(matchFrom).number;
          if (cursorLines.has(lineNumber)) continue;

          const specifier = match[2].trim();
          this._resolveAndDecorate(view, specifier, matchFrom, matchTo, config, "mdimage", collected);
        }
      }

      collected.sort((a, b) => a.from - b.from || a.to - b.to);
      const builder = new RangeSetBuilder<Decoration>();
      for (const { from, to, value } of collected) {
        builder.add(from, to, value);
      }
      return builder.finish();
    }

    _resolveAndDecorate(
      view: EditorView,
      specifier: string,
      matchFrom: number,
      matchTo: number,
      config: ImageConfig | null,
      _kind: string,
      collected: Array<{ from: number; to: number; value: Decoration }>,
    ) {
      if (this.resolved.has(specifier)) {
        const src = this.resolved.get(specifier)!;
        collected.push({
          from: matchFrom,
          to: matchTo,
          value: Decoration.replace({
            widget: new ImageWidget(src),
            block: true,
          }),
        });
        return;
      }

      // Remote URLs are used directly — no resolution needed
      if (/^https?:\/\//i.test(specifier)) {
        this.resolved.set(specifier, specifier);
        collected.push({
          from: matchFrom,
          to: matchTo,
          value: Decoration.replace({
            widget: new ImageWidget(specifier),
            block: true,
          }),
        });
        return;
      }

      if (!config) return;

      // Kick off async resolution; redraw when done
      const { vaultPath, filePath } = config;
      const isWikilink = !specifier.includes("/") && !specifier.startsWith(".");

      const resolve = isWikilink
        ? invoke<string | null>("resolve_wikilink", {
            vaultPath,
            linkTarget: specifier,
          }).then((absPath) => absPath)
        : invoke<string>("resolve_asset_path", {
            vaultPath,
            filePath,
            relativePath: specifier,
          }).then((absPath) => absPath);

      resolve
        .then((absPath) => {
          if (!absPath) return;
          const src = convertFileSrc(absPath);
          this.resolved.set(specifier, src);
          view.dispatch({ effects: triggerRedraw.of(null) });
        })
        .catch(() => {});
    }
  },
  { decorations: (plugin) => plugin.decorations },
);
