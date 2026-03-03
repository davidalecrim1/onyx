import { syntaxTree } from "@codemirror/language";
import { RangeSetBuilder } from "@codemirror/state";
import {
  Decoration,
  DecorationSet,
  EditorView,
  ViewPlugin,
  ViewUpdate,
  WidgetType,
} from "@codemirror/view";

class HorizontalRuleWidget extends WidgetType {
  toDOM(): HTMLElement {
    const hr = document.createElement("hr");
    hr.className = "onyx-hr";
    return hr;
  }

  eq(): boolean {
    return true;
  }
}

const hideMark = Decoration.replace({});

function buildDecorations(view: EditorView): DecorationSet {
  const { state } = view;
  const cursorLines = new Set(
    state.selection.ranges.map((range) => state.doc.lineAt(range.head).number),
  );

  type DecoEntry = { from: number; to: number; value: Decoration };
  const collected: DecoEntry[] = [];

  const push = (from: number, to: number, value: Decoration) => {
    collected.push({ from, to, value });
  };

  for (const { from, to } of view.visibleRanges) {
    syntaxTree(state).iterate({
      from,
      to,
      enter(node) {
        const lineNumber = state.doc.lineAt(node.from).number;
        const cursorIsHere = cursorLines.has(lineNumber);

        switch (node.type.name) {
          case "ATXHeading1":
          case "ATXHeading2":
          case "ATXHeading3":
          case "ATXHeading4":
          case "ATXHeading5":
          case "ATXHeading6": {
            const level = parseInt(node.type.name.slice(-1));
            push(
              node.from,
              node.to,
              Decoration.mark({ class: `onyx-h${level}` }),
            );
            break;
          }

          case "HeaderMark": {
            if (!cursorIsHere) {
              // The HeaderMark node covers only the `#` characters; the trailing
              // space before the heading text is not part of the node, so we
              // extend the hidden range by one to swallow it.
              const afterMark = node.to + 1;
              const lineEnd = state.doc.lineAt(node.to).to;
              push(node.from, Math.min(afterMark, lineEnd), hideMark);
            }
            break;
          }

          case "StrongEmphasis": {
            // Apply bold to the full range; marks are hidden separately
            push(node.from, node.to, Decoration.mark({ class: "onyx-bold" }));
            break;
          }

          case "Emphasis": {
            push(node.from, node.to, Decoration.mark({ class: "onyx-italic" }));
            break;
          }

          case "EmphasisMark": {
            if (!cursorIsHere) {
              push(node.from, node.to, hideMark);
            }
            break;
          }

          case "Strikethrough": {
            push(node.from, node.to, Decoration.mark({ class: "onyx-strike" }));
            break;
          }

          case "StrikethroughMark": {
            if (!cursorIsHere) {
              push(node.from, node.to, hideMark);
            }
            break;
          }

          case "InlineCode": {
            push(
              node.from,
              node.to,
              Decoration.mark({ class: "onyx-code-inline" }),
            );
            break;
          }

          case "CodeMark": {
            // Only hide inline code marks; fenced code fence marks are handled via CodeInfo/FencedCode
            const parent = node.node.parent;
            if (parent?.type.name === "InlineCode" && !cursorIsHere) {
              push(node.from, node.to, hideMark);
            }
            break;
          }

          case "FencedCode": {
            // Decorate each line of the code body
            const codeText = node.node.getChild("CodeText");
            if (codeText) {
              let lineStart = state.doc.lineAt(codeText.from).number;
              const lineEnd = state.doc.lineAt(codeText.to).number;
              while (lineStart <= lineEnd) {
                const line = state.doc.line(lineStart);
                push(
                  line.from,
                  line.from,
                  Decoration.line({ class: "onyx-codeblock" }),
                );
                lineStart++;
              }
            }

            // Hide fence markers (``` or ~~~) when cursor is off those lines
            for (const child of node.node
              .cursor()
              .node.getChildren("CodeMark")) {
              const fenceLineNumber = state.doc.lineAt(child.from).number;
              if (!cursorLines.has(fenceLineNumber)) {
                push(child.from, child.to, hideMark);
              }
            }
            break;
          }

          case "HorizontalRule": {
            if (!cursorIsHere) {
              push(
                node.from,
                node.to,
                Decoration.replace({
                  widget: new HorizontalRuleWidget(),
                  block: true,
                }),
              );
            }
            break;
          }

          case "Link": {
            push(node.from, node.to, Decoration.mark({ class: "onyx-link" }));

            if (!cursorIsHere) {
              // Hide [, ](url) leaving only the label visible
              let openBracket: { from: number; to: number } | null = null;
              let closeBracketAndUrl: { from: number; to: number } | null =
                null;

              node.node.cursor().iterate((child) => {
                if (child.type.name === "LinkMark") {
                  const text = state.doc.sliceString(child.from, child.to);
                  if (text === "[") {
                    openBracket = { from: child.from, to: child.to };
                  } else if (text === "]") {
                    // The closing bracket and everything after (](url)) up to link end
                    closeBracketAndUrl = { from: child.from, to: node.to };
                  }
                }
              });

              if (openBracket !== null) {
                push(
                  (openBracket as { from: number; to: number }).from,
                  (openBracket as { from: number; to: number }).to,
                  hideMark,
                );
              }
              if (closeBracketAndUrl !== null) {
                push(
                  (closeBracketAndUrl as { from: number; to: number }).from,
                  (closeBracketAndUrl as { from: number; to: number }).to,
                  hideMark,
                );
              }
            }
            break;
          }
        }
      },
    });
  }

  collected.sort((a, b) => a.from - b.from || a.to - b.to);

  const builder = new RangeSetBuilder<Decoration>();
  for (const { from, to, value } of collected) {
    builder.add(from, to, value);
  }
  return builder.finish();
}

export const markdownDecorations = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;

    constructor(view: EditorView) {
      this.decorations = buildDecorations(view);
    }

    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet || update.viewportChanged) {
        this.decorations = buildDecorations(update.view);
      }
    }
  },
  { decorations: (plugin) => plugin.decorations },
);
