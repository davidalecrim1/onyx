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

class BulletWidget extends WidgetType {
  toDOM(): HTMLElement {
    const span = document.createElement("span");
    span.className = "onyx-bullet";
    span.textContent = "•";
    return span;
  }

  eq(): boolean {
    return true;
  }
}

class CheckboxWidget extends WidgetType {
  constructor(
    private readonly view: EditorView,
    private readonly checked: boolean,
  ) {
    super();
  }

  toDOM(): HTMLElement {
    const wrapper = document.createElement("span");
    wrapper.className = `onyx-task-checkbox${this.checked ? " onyx-task-checkbox--checked" : ""}`;
    wrapper.setAttribute("role", "checkbox");
    wrapper.setAttribute("aria-checked", String(this.checked));
    wrapper.setAttribute("tabindex", "0");

    if (this.checked) {
      wrapper.innerHTML =
        `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">` +
        `<rect x="1" y="1" width="14" height="14" rx="3" fill="currentColor"/>` +
        `<path d="M4 8l2.5 2.5L12 5.5" stroke="#1a1d23" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>` +
        `</svg>`;
    } else {
      wrapper.innerHTML =
        `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">` +
        `<rect x="1" y="1" width="14" height="14" rx="3" stroke="currentColor" stroke-width="1.5"/>` +
        `</svg>`;
    }

    const toggle = (event: Event) => {
      event.preventDefault();
      const pos = this.view.posAtDOM(wrapper);
      const line = this.view.state.doc.lineAt(pos);
      const lineText = this.view.state.doc.sliceString(line.from, line.to);
      const uncheckedMatch = lineText.match(/\[ \]/);
      const checkedMatch = lineText.match(/\[x\]|\[X\]/);
      if (uncheckedMatch && uncheckedMatch.index !== undefined) {
        const from = line.from + uncheckedMatch.index;
        this.view.dispatch({
          changes: { from, to: from + 3, insert: "[x]" },
        });
      } else if (checkedMatch && checkedMatch.index !== undefined) {
        const from = line.from + checkedMatch.index;
        this.view.dispatch({
          changes: { from, to: from + 3, insert: "[ ]" },
        });
      }
    };

    wrapper.addEventListener("click", toggle);
    wrapper.addEventListener("keydown", (event) => {
      if (event.key === " " || event.key === "Enter") toggle(event);
    });

    return wrapper;
  }

  eq(other: CheckboxWidget): boolean {
    return this.checked === other.checked;
  }
}

class FrontmatterWidget extends WidgetType {
  toDOM(): HTMLElement {
    const span = document.createElement("span");
    span.className = "onyx-frontmatter-pill";
    span.textContent = "Frontmatter";
    return span;
  }

  eq(): boolean {
    return true;
  }
}

const CALLOUT_TYPES: Record<
  string,
  { borderColor: string; icon: string; cssClass: string }
> = {
  note: { borderColor: "#74ade8", icon: "ℹ", cssClass: "onyx-callout-note" },
  info: { borderColor: "#74ade8", icon: "ℹ", cssClass: "onyx-callout-note" },
  tip: { borderColor: "#4db89a", icon: "💡", cssClass: "onyx-callout-tip" },
  hint: { borderColor: "#4db89a", icon: "💡", cssClass: "onyx-callout-hint" },
  warning: {
    borderColor: "#e8c074",
    icon: "⚠",
    cssClass: "onyx-callout-warning",
  },
  danger: {
    borderColor: "#e87474",
    icon: "✕",
    cssClass: "onyx-callout-danger",
  },
  error: { borderColor: "#e87474", icon: "✕", cssClass: "onyx-callout-error" },
  quote: {
    borderColor: "#a9afbc",
    icon: "\u201C",
    cssClass: "onyx-callout-quote",
  },
  cite: {
    borderColor: "#a9afbc",
    icon: "\u201C",
    cssClass: "onyx-callout-cite",
  },
};

const CALLOUT_RE = /^>\s*\[!([\w-]+)\]/i;

class TableWidget extends WidgetType {
  constructor(private readonly rawText: string) {
    super();
  }

  eq(other: TableWidget): boolean {
    return this.rawText === other.rawText;
  }

  toDOM(): HTMLElement {
    const lines = this.rawText.split("\n").filter((line) => line.trim() !== "");
    const separatorIndex = lines.findIndex((line) => /^[|\s\-:]+$/.test(line));

    const alignments: Array<"left" | "center" | "right"> = [];
    if (separatorIndex !== -1) {
      const sepLine = lines[separatorIndex];
      const cells = sepLine
        .split("|")
        .map((c) => c.trim())
        .filter((c) => c !== "");
      for (const cell of cells) {
        if (cell.startsWith(":") && cell.endsWith(":")) {
          alignments.push("center");
        } else if (cell.endsWith(":")) {
          alignments.push("right");
        } else {
          alignments.push("left");
        }
      }
    }

    const parseRow = (line: string): string[] =>
      line
        .split("|")
        .map((c) => c.trim())
        .filter((_, index, arr) => index !== 0 || arr[0] !== "")
        .filter(
          (_, index, arr) =>
            index !== arr.length - 1 || arr[arr.length - 1] !== "",
        );

    const dataLines = lines.filter((_, index) => index !== separatorIndex);
    const headerRow = dataLines[0];
    const bodyRows = dataLines.slice(1);

    const table = document.createElement("table");
    table.className = "onyx-table";

    const thead = document.createElement("thead");
    const headerTr = document.createElement("tr");
    const headerCells = parseRow(headerRow);
    headerCells.forEach((cell, index) => {
      const th = document.createElement("th");
      th.textContent = cell;
      if (alignments[index]) th.style.textAlign = alignments[index];
      headerTr.appendChild(th);
    });
    thead.appendChild(headerTr);
    table.appendChild(thead);

    const tbody = document.createElement("tbody");
    for (const row of bodyRows) {
      const tr = document.createElement("tr");
      const cells = parseRow(row);
      cells.forEach((cell, index) => {
        const td = document.createElement("td");
        td.textContent = cell;
        if (alignments[index]) td.style.textAlign = alignments[index];
        tr.appendChild(td);
      });
      tbody.appendChild(tr);
    }
    table.appendChild(tbody);

    return table;
  }
}

class CalloutWidget extends WidgetType {
  constructor(
    private readonly calloutType: string,
    private readonly bodyLines: string[],
  ) {
    super();
  }

  eq(other: CalloutWidget): boolean {
    return (
      this.calloutType === other.calloutType &&
      this.bodyLines.join("\n") === other.bodyLines.join("\n")
    );
  }

  toDOM(): HTMLElement {
    const typeKey = this.calloutType.toLowerCase();
    const config = CALLOUT_TYPES[typeKey] ?? {
      borderColor: "#a9afbc",
      icon: "•",
      cssClass: "onyx-callout-unknown",
    };

    const wrapper = document.createElement("div");
    wrapper.className = `onyx-callout ${config.cssClass}`;

    const title = document.createElement("div");
    title.className = "onyx-callout-title";

    const icon = document.createElement("span");
    icon.className = "onyx-callout-icon";
    icon.textContent = config.icon;
    title.appendChild(icon);

    const label = document.createElement("span");
    label.textContent =
      this.calloutType.charAt(0).toUpperCase() + this.calloutType.slice(1);
    title.appendChild(label);

    wrapper.appendChild(title);

    const body = document.createElement("div");
    body.className = "onyx-callout-body";
    body.textContent = this.bodyLines
      .map((line) => line.replace(/^>\s?/, ""))
      .join("\n");
    wrapper.appendChild(body);

    return wrapper;
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

  if (state.doc.lines >= 1 && state.doc.line(1).text === "---") {
    let fmClosingLine = -1;
    for (let lineNum = 2; lineNum <= state.doc.lines; lineNum++) {
      if (state.doc.line(lineNum).text === "---") {
        fmClosingLine = lineNum;
        break;
      }
    }
    if (fmClosingLine !== -1) {
      let fmHasCursor = false;
      for (let ln = 1; ln <= fmClosingLine; ln++) {
        if (cursorLines.has(ln)) {
          fmHasCursor = true;
          break;
        }
      }
      if (!fmHasCursor) {
        const fmFrom = state.doc.line(1).from;
        const fmTo = state.doc.line(fmClosingLine).to;
        push(
          fmFrom,
          fmTo,
          Decoration.replace({
            widget: new FrontmatterWidget(),
            block: true,
          }),
        );
      }
    }
  }

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

          case "ListMark": {
            if (cursorIsHere) break;
            const markText = state.doc.sliceString(node.from, node.to);
            // Only handle unordered markers; ordered markers like "1." are left as-is
            if (markText !== "-" && markText !== "*" && markText !== "+") break;
            const parentItem = node.node.parent;
            const hasTaskMarker = parentItem?.getChild("TaskMarker") !== null;
            // If a TaskMarker follows, replace only the mark character to avoid
            // adjacent replacement artifacts; otherwise swallow the trailing space too
            const replaceTo = hasTaskMarker ? node.to : node.to + 1;
            push(
              node.from,
              replaceTo,
              Decoration.replace({ widget: new BulletWidget() }),
            );
            break;
          }

          case "TaskMarker": {
            if (cursorIsHere) break;
            const markerText = state.doc.sliceString(node.from, node.to);
            const isChecked = markerText === "[x]" || markerText === "[X]";
            // TaskMarker spans "[ ]" or "[x]"; swallow the trailing space too.
            // Guard against going past line end (mirrors HeaderMark pattern).
            const taskLineEnd = state.doc.lineAt(node.to).to;
            push(
              node.from,
              Math.min(node.to + 1, taskLineEnd),
              Decoration.replace({
                widget: new CheckboxWidget(view, isChecked),
              }),
            );
            break;
          }

          case "Table": {
            const tableLineStart = state.doc.lineAt(node.from).number;
            const tableLineEnd = state.doc.lineAt(node.to).number;
            let tableHasCursor = false;
            for (let ln = tableLineStart; ln <= tableLineEnd; ln++) {
              if (cursorLines.has(ln)) {
                tableHasCursor = true;
                break;
              }
            }
            if (!tableHasCursor) {
              push(
                node.from,
                node.to,
                Decoration.replace({
                  widget: new TableWidget(
                    state.doc.sliceString(node.from, node.to),
                  ),
                  block: true,
                }),
              );
            }
            break;
          }

          case "Blockquote": {
            const bqLineStart = state.doc.lineAt(node.from).number;
            const bqLineEnd = state.doc.lineAt(node.to).number;
            let bqHasCursor = false;
            for (let ln = bqLineStart; ln <= bqLineEnd; ln++) {
              if (cursorLines.has(ln)) {
                bqHasCursor = true;
                break;
              }
            }
            if (bqHasCursor) break;

            const firstLine = state.doc.line(bqLineStart).text;
            const calloutMatch = CALLOUT_RE.exec(firstLine);
            if (!calloutMatch) break;

            const calloutType = calloutMatch[1];
            const bodyLines: string[] = [];
            for (let ln = bqLineStart + 1; ln <= bqLineEnd; ln++) {
              bodyLines.push(state.doc.line(ln).text);
            }

            push(
              node.from,
              node.to,
              Decoration.replace({
                widget: new CalloutWidget(calloutType, bodyLines),
                block: true,
              }),
            );
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
