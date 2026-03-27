export interface OnyxPalette {
  background: string;
  surface: string;
  "surface-hover": string;
  "surface-active": string;
  accent: string;
  "text-primary": string;
  "text-secondary": string;
  "code-bg": string;
  success: string;
  warning: string;
  danger: string;
}

export const DEFAULT_PALETTE: OnyxPalette = {
  background: "#282c33",
  surface: "#2f343e",
  "surface-hover": "#363c46",
  "surface-active": "#454a56",
  accent: "#74ade8",
  "text-primary": "#dce0e5",
  "text-secondary": "#a9afbc",
  "code-bg": "#22262e",
  success: "#4db89a",
  warning: "#e8c074",
  danger: "#e87474",
};

/// Merges user overrides onto the default palette. Unknown keys in overrides are ignored.
export function mergeTheme(overrides: Partial<OnyxPalette>): OnyxPalette {
  return { ...DEFAULT_PALETTE, ...overrides };
}

/// Injects the palette as CSS custom properties on :root. Replaces any previous injection.
export function applyTheme(palette: OnyxPalette): void {
  const vars = (Object.entries(palette) as [string, string][])
    .map(([key, value]) => `  --onyx-${key}: ${value};`)
    .join("\n");
  const css = `:root {\n${vars}\n}`;

  let tag = document.getElementById("onyx-theme") as HTMLStyleElement | null;
  if (!tag) {
    tag = document.createElement("style");
    tag.id = "onyx-theme";
    document.head.appendChild(tag);
  }
  tag.textContent = css;
}
