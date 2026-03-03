import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface VaultInfo {
  name: string;
  root: string;
}

interface Props {
  onVaultOpened: (vaultPath: string, vaultName: string) => void;
}

async function getDefaultVaultDir(): Promise<string | undefined> {
  try {
    return await invoke<string>("get_default_vault_dir");
  } catch {
    return undefined;
  }
}

export default function WelcomePage({ onVaultOpened }: Props) {
  const [error, setError] = useState<string | null>(null);

  async function handleCreateVault() {
    setError(null);
    try {
      const defaultPath = await getDefaultVaultDir();
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Choose folder for new vault",
        defaultPath,
      });
      if (!selected) return;
      const vault = await invoke<VaultInfo>("create_vault", { path: selected });
      await invoke("maximize_window").catch(() => {});
      onVaultOpened(vault.root, vault.name);
    } catch (err) {
      setError(String(err));
    }
  }

  async function handleOpenVault() {
    setError(null);
    try {
      const defaultPath = await getDefaultVaultDir();
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Open vault",
        defaultPath,
      });
      if (!selected) return;
      const vault = await invoke<VaultInfo>("open_vault", { path: selected });
      await invoke("maximize_window").catch(() => {});
      onVaultOpened(vault.root, vault.name);
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <div
      data-tauri-drag-region
      className="flex h-full flex-col bg-background text-text-primary"
    >
      <div className="flex flex-1 flex-col items-center justify-center">
        <h1 className="mb-2 text-4xl font-semibold tracking-tight">Onyx</h1>
        <p className="mb-10 text-text-secondary">
          Your personal markdown workspace
        </p>

        <div className="flex gap-4">
          <button
            onClick={handleCreateVault}
            className="rounded-md bg-accent px-5 py-2.5 text-sm font-medium text-background transition-opacity hover:opacity-80"
          >
            Create Vault
          </button>
          <button
            onClick={handleOpenVault}
            className="rounded-md bg-surface px-5 py-2.5 text-sm font-medium text-text-primary transition-colors hover:bg-surface-hover"
          >
            Open Vault
          </button>
        </div>

        {error && (
          <p className="mt-6 max-w-sm text-center text-sm text-red-400">
            {error}
          </p>
        )}
      </div>
    </div>
  );
}
