import {
  autocompletion,
  type CompletionContext,
  type CompletionResult,
} from "@codemirror/autocomplete";

const TAG_BODY = /[a-zA-Z0-9_-]/;

function tagCompletionSource(tags: string[]) {
  return (context: CompletionContext): CompletionResult | null => {
    const { state, pos } = context;

    // Walk backwards to find whether we're inside a #tag token.
    let scanPos = pos - 1;
    while (
      scanPos >= 0 &&
      TAG_BODY.test(state.doc.sliceString(scanPos, scanPos + 1))
    ) {
      scanPos--;
    }

    // The character at scanPos should be `#`.
    if (state.doc.sliceString(scanPos, scanPos + 1) !== "#") return null;

    // The `#` must be at the start of content or preceded by whitespace so we
    // don't treat things like color codes (#ff0000) as tags.
    const charBeforeHash =
      scanPos > 0 ? state.doc.sliceString(scanPos - 1, scanPos) : " ";
    if (
      charBeforeHash !== " " &&
      charBeforeHash !== "\t" &&
      charBeforeHash !== "\n"
    ) {
      return null;
    }

    const hashPos = scanPos;
    const partial = state.doc.sliceString(hashPos + 1, pos).toLowerCase();

    // The first character after # must be a letter (to exclude headings like `# Title`).
    if (partial.length > 0 && !/^[a-zA-Z]/.test(partial)) return null;

    const filtered = tags.filter((tag) =>
      tag.toLowerCase().startsWith(partial),
    );

    if (filtered.length === 0 && !context.explicit) return null;

    return {
      from: hashPos,
      options: filtered.map((tag) => ({ label: `#${tag}`, type: "keyword" })),
    };
  };
}

/// Returns a CodeMirror extension that provides `#tag` autocomplete from the supplied tag list.
export function tagAutocomplete(tags: string[]) {
  return autocompletion({ override: [tagCompletionSource(tags)] });
}
