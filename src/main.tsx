import React from "react";
import ReactDOM from "react-dom/client";
import { attachConsole } from "@tauri-apps/plugin-log";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { applyTheme, mergeTheme, DEFAULT_PALETTE, OnyxPalette } from "./theme";
import "./index.css";

attachConsole();

async function bootstrap() {
  let palette = DEFAULT_PALETTE;
  try {
    const raw = await invoke<string>("load_theme");
    const overrides = JSON.parse(raw) as Partial<OnyxPalette>;
    palette = mergeTheme(overrides);
  } catch {
    // Fall back to defaults — do not block startup on theme load failure
  }
  applyTheme(palette);

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </React.StrictMode>,
  );
}

bootstrap();
