# Tasks

## Tauri Backend / Frontend Integration Improvements

### 1. Make `OnyxError` serializable for typed error responses (high priority)

**Problem:** `OnyxError` in `src-tauri/src/error.rs` does not derive `Serialize`. Every command works around this with `.map_err(|e| e.to_string())`, so the frontend only ever receives opaque strings — no structured error types, no ability to distinguish error categories.

**Fix:**
1. Add `serde::Serialize` derive to `OnyxError`:
   ```rust
   #[derive(Debug, Serialize)]
   #[serde(tag = "kind", content = "message")]
   pub enum OnyxError { ... }
   ```
2. Change all command signatures from `Result<T, String>` to `Result<T, OnyxError>` and remove the `.map_err(|e| e.to_string())` calls in `src-tauri/src/commands.rs` (lines 47, 48, 59, 60, 71, 78, 84, 97, 104).

---

### 2. Create a typed API bindings layer on the frontend (medium priority)

**Problem:** All Tauri command names are raw strings scattered across components (`"get_file_tree"`, `"read_file"`, `"write_file"`, etc. in `src/pages/EditorPage.tsx` lines 85, 108, 131, 164 and `src/pages/WelcomePage.tsx` lines 26, 27, 43, 44). A typo silently fails at runtime with no compile-time safety.

**Fix:** Create `src/api.ts` that centralises and types all `invoke` calls:
```typescript
import { invoke } from "@tauri-apps/api/core";
import type { FileTreeEntry } from "./types";

export const api = {
  getFileTree: (vaultPath: string) =>
    invoke<FileTreeEntry[]>("get_file_tree", { vaultPath }),
  readFile: (path: string) =>
    invoke<string>("read_file", { path }),
  writeFile: (path: string, content: string) =>
    invoke<void>("write_file", { path, content }),
  createFile: (vaultPath: string, name: string) =>
    invoke<string>("create_file", { vaultPath, name }),
  createVault: (path: string) =>
    invoke<void>("create_vault", { path }),
  openVault: (path: string) =>
    invoke<FileTreeEntry[]>("open_vault", { path }),
  getKnownVaults: () =>
    invoke<string[]>("get_known_vaults"),
  maximizeWindow: () =>
    invoke<void>("maximize_window"),
};
```
Then replace all direct `invoke(...)` calls in components with the typed wrappers.

> Alternative: use [tauri-specta](https://github.com/oscartbeaumont/tauri-specta) to auto-generate bindings from Rust command signatures.

---

### 3. Add missing generic types to untyped `invoke` calls (medium priority)

**Problem:** Two `invoke` calls lack explicit return type generics, breaking consistency with the rest of the codebase:
- `src/pages/EditorPage.tsx:164` — `invoke("write_file", ...)` should be `invoke<void>(...)`
- `src/pages/WelcomePage.tsx:27` and `:44` — `invoke("maximize_window")` should be `invoke<void>(...)`

**Fix:** Add `<void>` generic to each call. Superseded if task 2 is completed first (the API layer handles this centrally).

---

### 4. Define explicit Tauri capabilities in `tauri.conf.json` (medium priority)

**Problem:** `src-tauri/tauri.conf.json` has no explicit `capabilities` declaration — the app relies on Tauri 2 defaults. The dialog plugin is actively used but not explicitly permitted, which can cause issues in stricter build targets or future Tauri updates.

**Fix:** Create a capabilities file at `src-tauri/capabilities/default.json`:
```json
{
  "identifier": "default",
  "description": "Default capabilities for Onyx",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "fs:default"
  ]
}
```
Reference it in `tauri.conf.json` under `"app.security.capabilities"`.

---

### 5. Surface file operation errors in the UI (low priority)

**Problem:** File operation errors in `src/pages/EditorPage.tsx` (lines 120, 135, 167) are only logged to the console via `.catch((err) => console.error(...))`. Users get no feedback when a read or write fails. Additionally, `maximize_window` errors in `WelcomePage.tsx` (lines 27, 44) are silently swallowed with `.catch(() => {})`.

**Fix:** Add user-facing error state for file operations (e.g., an error banner or toast). At minimum, the silent `.catch(() => {})` on `maximize_window` should log the error.

### 6. Fix the icon to match MacOS.


### 7. The Checklist is not rendered as clickable like Obsidian
- [ ] Dacid
- [ ] Alecrim
- [ ] Dos Santos
