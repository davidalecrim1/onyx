import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
  RangeSetBuilder,
  StateEffect,
  StateField,
  Transaction,
} from "@codemirror/state";
import type { EditorState } from "@codemirror/state";
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

const resolveImage = StateEffect.define<{ specifier: string; src: string }>();

const resolvedImageCache = StateField.define<Map<string, string>>({
  create: () => new Map(),
  update(cache, transaction) {
    let next = cache;
    for (const effect of transaction.effects) {
      if (effect.is(resolveImage)) {
        if (next === cache) next = new Map(cache);
        next.set(effect.value.specifier, effect.value.src);
      }
    }
    return next;
  },
});

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

function collectImageSpecifiers(
  state: EditorState,
): Array<{ specifier: string; from: number; to: number }> {
  const cursorLines = new Set(
    state.selection.ranges.map((range) => state.doc.lineAt(range.head).number),
  );
  const results: Array<{ specifier: string; from: number; to: number }> = [];
  const text = state.doc.sliceString(0, state.doc.length);

  WIKIIMAGE_RE.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = WIKIIMAGE_RE.exec(text)) !== null) {
    const matchFrom = match.index;
    const matchTo = matchFrom + match[0].length;
    const lineNumber = state.doc.lineAt(matchFrom).number;
    if (cursorLines.has(lineNumber)) continue;
    results.push({ specifier: match[1].trim(), from: matchFrom, to: matchTo });
  }

  MDIMAGE_RE.lastIndex = 0;
  while ((match = MDIMAGE_RE.exec(text)) !== null) {
    const matchFrom = match.index;
    const matchTo = matchFrom + match[0].length;
    const lineNumber = state.doc.lineAt(matchFrom).number;
    if (cursorLines.has(lineNumber)) continue;
    results.push({ specifier: match[2].trim(), from: matchFrom, to: matchTo });
  }

  return results;
}

function buildImageDecorations(state: EditorState): DecorationSet {
  try {
    const cache = state.field(resolvedImageCache);
    const entries = collectImageSpecifiers(state);

    type Entry = { from: number; to: number; value: Decoration };
    const collected: Entry[] = [];

    for (const { specifier, from, to } of entries) {
      const src = cache.get(specifier);
      if (!src) continue;
      collected.push({
        from,
        to,
        value: Decoration.replace({
          widget: new ImageWidget(src),
          block: true,
        }),
      });
    }

    collected.sort((a, b) => a.from - b.from || a.to - b.to);
    const builder = new RangeSetBuilder<Decoration>();
    for (const { from, to, value } of collected) {
      builder.add(from, to, value);
    }
    return builder.finish();
  } catch (error) {
    console.error("imageDecorations: buildImageDecorations failed", error);
    return Decoration.none;
  }
}

const imageDecorationField = StateField.define<DecorationSet>({
  create(state) {
    return buildImageDecorations(state);
  },
  update(decorations, transaction) {
    const needsRebuild =
      transaction.docChanged ||
      transaction.selection ||
      transaction.effects.some((effect) => effect.is(resolveImage));
    if (!needsRebuild) return decorations.map(transaction.changes);
    return buildImageDecorations(transaction.state);
  },
  provide(field) {
    return EditorView.decorations.from(field);
  },
});

/// Thin side-effect plugin that resolves unresolved image specifiers asynchronously.
const imageResolverPlugin = ViewPlugin.fromClass(
  class {
    private pending = new Set<string>();

    constructor(view: EditorView) {
      this.resolveUnknown(view);
    }

    update(update: ViewUpdate) {
      if (
        update.docChanged ||
        update.selectionSet ||
        update.transactions.some((tr) =>
          tr.effects.some((effect) => effect.is(setImageConfig)),
        )
      ) {
        this.resolveUnknown(update.view);
      }
    }

    private resolveUnknown(view: EditorView) {
      const state = view.state;
      const cache = state.field(resolvedImageCache);
      const config = state.field(imageConfigField);
      const entries = collectImageSpecifiers(state);

      for (const { specifier } of entries) {
        if (cache.has(specifier) || this.pending.has(specifier)) continue;

        if (/^https?:\/\//i.test(specifier)) {
          view.dispatch({
            effects: resolveImage.of({ specifier, src: specifier }),
          });
          continue;
        }

        if (!config) continue;

        this.pending.add(specifier);
        const { vaultPath, filePath } = config;
        const isWikilink =
          !specifier.includes("/") && !specifier.startsWith(".");

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
            this.pending.delete(specifier);
            if (!absPath) return;
            const src = convertFileSrc(absPath);
            view.dispatch({
              effects: resolveImage.of({ specifier, src }),
            });
          })
          .catch(() => {
            this.pending.delete(specifier);
          });
      }
    }
  },
);

export const imageDecorations = [
  resolvedImageCache,
  imageDecorationField,
  imageResolverPlugin,
];
