import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import WelcomePage from "./pages/WelcomePage";
import EditorPage from "./pages/EditorPage";

interface VaultEntry {
  name: string;
  path: string;
}

export type AppView =
  | { kind: "welcome" }
  | { kind: "editor"; vaultPath: string; vaultName: string };

function getVaultFromUrl(): string | null {
  const params = new URLSearchParams(window.location.search);
  const vault = params.get("vault");
  return vault ?? null;
}

export default function App() {
  const [view, setView] = useState<AppView>({ kind: "welcome" });
  const [knownVaults, setKnownVaults] = useState<VaultEntry[]>([]);

  const refreshKnownVaults = useCallback(() => {
    invoke<VaultEntry[]>("get_known_vaults")
      .then(setKnownVaults)
      .catch((err) => console.error("Failed to load known vaults:", err));
  }, []);

  useEffect(() => {
    refreshKnownVaults();
  }, [refreshKnownVaults]);

  useEffect(() => {
    const vaultFromUrl = getVaultFromUrl();

    if (vaultFromUrl) {
      invoke<{ name: string; root: string }>("open_vault", {
        path: vaultFromUrl,
      })
        .then((vault) => {
          setView({
            kind: "editor",
            vaultPath: vault.root,
            vaultName: vault.name,
          });
          refreshKnownVaults();
        })
        .catch((err) =>
          console.error("Failed to open vault from URL param:", err),
        );
      return;
    }

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
      .catch((err) =>
        console.error("Failed to restore last active vault:", err),
      );
  }, []);

  function handleVaultOpened(vaultPath: string, vaultName: string) {
    refreshKnownVaults();
    setView({ kind: "editor", vaultPath, vaultName });
  }

  const handleSwitchVault = useCallback(async (path: string, _name: string) => {
    try {
      await invoke("open_vault_window", { path });
    } catch (err) {
      console.error("Failed to open vault window:", err);
    }
  }, []);

  const handleCloseVault = useCallback(async () => {
    try {
      await getCurrentWindow().close();
    } catch (err) {
      console.error("Failed to close window:", err);
    }
  }, []);

  const handleOpenWelcome = useCallback(async () => {
    try {
      await invoke("open_welcome_window");
    } catch (err) {
      console.error("Failed to open welcome window:", err);
    }
  }, []);

  if (view.kind === "editor") {
    return (
      <EditorPage
        vaultPath={view.vaultPath}
        vaultName={view.vaultName}
        knownVaults={knownVaults}
        onClose={handleCloseVault}
        onSwitchVault={handleSwitchVault}
        onOpenWelcome={handleOpenWelcome}
      />
    );
  }

  return <WelcomePage onVaultOpened={handleVaultOpened} />;
}
