import { useState } from "react";
import WelcomePage from "./pages/WelcomePage";
import EditorPage from "./pages/EditorPage";

export type AppView =
  | { kind: "welcome" }
  | { kind: "editor"; vaultPath: string; vaultName: string };

export default function App() {
  const [view, setView] = useState<AppView>({ kind: "welcome" });

  if (view.kind === "editor") {
    return (
      <EditorPage
        vaultPath={view.vaultPath}
        vaultName={view.vaultName}
        onClose={() => setView({ kind: "welcome" })}
      />
    );
  }

  return (
    <WelcomePage
      onVaultOpened={(vaultPath, vaultName) =>
        setView({ kind: "editor", vaultPath, vaultName })
      }
    />
  );
}
