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
    favorite: raw.favorite,
    category: raw.category ?? undefined,
    createdAt: raw.created_at,
    updatedAt: raw.updated_at,
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
      created_at: entry.createdAt,
      updated_at: entry.updatedAt,
    },
  });
}

export async function deleteEntry(id: string): Promise<void> {
  await invoke("delete_entry", { id });
}

export async function generatePassword(
  length = 20,
  symbols = true
): Promise<string> {
  return invoke<string>("generate_password", { length, symbols });
}
