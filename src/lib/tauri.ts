import { invoke } from "@tauri-apps/api/core";
import type { PasswordEntry } from "./types";

/** Map Rust snake_case → frontend camelCase */
function mapEntry(raw: any): PasswordEntry {
  return {
    id: raw.id,
    title: raw.title,
    username: raw.username,
    password: raw.password,
    url: raw.url ?? undefined,
    notes: raw.notes ?? undefined,
    favorite: raw.favorite ?? false,
    category: raw.category ?? undefined,
    deleted: raw.deleted ?? false,
    createdAt: raw.created_at,
    updatedAt: raw.updated_at,
    lastUsedAt: raw.last_used_at ?? undefined,
  };
}

export async function unlockVault(password: string): Promise<PasswordEntry[]> {
  const raw = await invoke<any[]>("unlock_vault", { password });
  return raw.map(mapEntry);
}

export async function lockVault(): Promise<void> {
  await invoke("lock_vault");
}

export async function listEntries(): Promise<PasswordEntry[]> {
  const raw = await invoke<any[]>("list_entries");
  return raw.map(mapEntry);
}

export async function addEntry(input: {
  title: string;
  username: string;
  password: string;
  url?: string;
  notes?: string;
  category?: string;
}): Promise<PasswordEntry> {
  const raw = await invoke<any>("add_entry", {
    title: input.title,
    username: input.username,
    password: input.password,
    url: input.url ?? null,
    notes: input.notes ?? null,
    category: input.category ?? null,
  });
  return mapEntry(raw);
}

export async function updateEntry(entry: PasswordEntry): Promise<void> {
  await invoke("update_entry", {
    entry: {
      id: entry.id,
      title: entry.title,
      username: entry.username,
      password: entry.password,
      url: entry.url ?? null,
      notes: entry.notes ?? null,
      favorite: entry.favorite,
      category: entry.category ?? null,
      deleted: entry.deleted,
      created_at: entry.createdAt,
      updated_at: entry.updatedAt,
      last_used_at: entry.lastUsedAt ?? null,
    },
  });
}

export async function deleteEntry(id: string): Promise<void> {
  await invoke("delete_entry", { id });
}

export async function restoreEntry(id: string): Promise<void> {
  await invoke("restore_entry", { id });
}

export async function purgeDeleted(): Promise<void> {
  await invoke("purge_deleted");
}

export async function touchEntry(id: string): Promise<void> {
  await invoke("touch_entry", { id });
}

export async function changeMasterPassword(
  currentPassword: string,
  newPassword: string
): Promise<void> {
  await invoke("change_master_password", {
    currentPassword,
    newPassword,
  });
}

export async function exportVault(): Promise<string> {
  return invoke<string>("export_vault");
}

export async function importVault(data: string): Promise<void> {
  await invoke("import_vault", { data });
}

export type GenerateMode = "password" | "passphrase";

export async function generatePassword(
  opts: {
    mode?: GenerateMode;
    length?: number;
    symbols?: boolean;
    excludeAmbiguous?: boolean;
  } = {}
): Promise<string> {
  const {
    mode = "password",
    length = mode === "passphrase" ? 5 : 20,
    symbols = true,
    excludeAmbiguous = false,
  } = opts;

  return invoke<string>("generate_password", {
    mode,
    length,
    symbols,
    excludeAmbiguous,
  });
}

/** Copy text to clipboard and clear after `clearAfterMs` (default 30s). */
export async function copyWithClear(
  text: string,
  clearAfterMs = 30_000
): Promise<void> {
  await navigator.clipboard.writeText(text);
  if (clearAfterMs > 0) {
    setTimeout(async () => {
      try {
        const current = await navigator.clipboard.readText();
        if (current === text) {
          await navigator.clipboard.writeText("");
        }
      } catch {
        // Clipboard read may be denied — ignore
      }
    }, clearAfterMs);
  }
}
