import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import WelcomePage from "./pages/WelcomePage";
import EditorPage from "./pages/EditorPage";

interface VaultEntry {
  name: string;
  path: string;
}

export type AppView =
  | { kind: "welcome" }
  | { kind: "editor"; vaultPath: string; vaultName: string };

export default function App() {
  const [view, setView] = useState<AppView>({ kind: "welcome" });
  const [knownVaults, setKnownVaults] = useState<VaultEntry[]>([]);

  const refreshKnownVaults = useCallback(() => {
    invoke<VaultEntry[]>("get_known_vaults")
      .then(setKnownVaults)
      .catch(() => {});
  }, []);

  useEffect(() => {
    refreshKnownVaults();
  }, [refreshKnownVaults]);

  useEffect(() => {
    invoke<{ name: string; path: string } | null>("get_last_active_vault")
      .then((entry) => {
        if (entry) {
          setView({
            kind: "editor",
            vaultPath: entry.path,
            vaultName: entry.name,
          });
          refreshKnownVaults();
        }
      })
      .catch(() => {});
  }, []);

  function handleVaultOpened(vaultPath: string, vaultName: string) {
    refreshKnownVaults();
    setView({ kind: "editor", vaultPath, vaultName });
  }

  const handleSwitchVault = useCallback(
    async (path: string, name: string) => {
      try {
        await invoke("open_vault", { path });
      } catch {
        // Already registered — open_vault is idempotent.
      }
      refreshKnownVaults();
      setView({ kind: "editor", vaultPath: path, vaultName: name });
    },
    [refreshKnownVaults],
  );

  if (view.kind === "editor") {
    return (
      <EditorPage
        vaultPath={view.vaultPath}
        vaultName={view.vaultName}
        knownVaults={knownVaults}
        onClose={() => setView({ kind: "welcome" })}
        onSwitchVault={handleSwitchVault}
      />
    );
  }

  return <WelcomePage onVaultOpened={handleVaultOpened} />;
}
