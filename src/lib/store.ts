import { writable, derived } from "svelte/store";
import type { PasswordEntry, View } from "./types";

export const isUnlocked = writable(false);
export const entries = writable<PasswordEntry[]>([]);
export const selectedId = writable<string | null>(null);
export const searchQuery = writable("");
export const currentView = writable<View>("all");
export const currentCategory = writable<string | null>(null);

/** Idle auto-lock timeout in ms (5 minutes). Set 0 to disable. */
export const autoLockMs = writable(5 * 60 * 1000);

export const filteredEntries = derived(
  [entries, searchQuery, currentView, currentCategory],
  ([$entries, $search, $view, $category]) => {
    let list = $entries;

    if ($view === "trash") {
      list = list.filter((e) => e.deleted);
    } else {
      list = list.filter((e) => !e.deleted);

      if ($view === "favorites") {
        list = list.filter((e) => e.favorite);
      } else if ($view === "recent") {
        list = list
          .filter((e) => e.lastUsedAt)
          .sort((a, b) => (b.lastUsedAt ?? 0) - (a.lastUsedAt ?? 0));
      } else if ($view === "category" && $category) {
        list = list.filter((e) => e.category === $category);
      }
    }

    if ($search.trim()) {
      const q = $search.toLowerCase();
      list = list.filter(
        (e) =>
          e.title.toLowerCase().includes(q) ||
          e.username.toLowerCase().includes(q) ||
          (e.url && e.url.toLowerCase().includes(q)) ||
          (e.notes && e.notes.toLowerCase().includes(q)) ||
          (e.category && e.category.toLowerCase().includes(q))
      );
    }

    // For non-recent views: newest first
    if ($view !== "recent") {
      list = [...list].sort((a, b) => b.updatedAt - a.updatedAt);
    }

    return list;
  }
);

export const selectedEntry = derived(
  [entries, selectedId],
  ([$entries, $id]) => $entries.find((e) => e.id === $id) ?? null
);

/** Unique categories derived from entries (excluding deleted). */
export const categories = derived(entries, ($entries) => {
  const set = new Set<string>();
  for (const e of $entries) {
    if (!e.deleted && e.category?.trim()) {
      set.add(e.category.trim());
    }
  }
  return Array.from(set).sort((a, b) => a.localeCompare(b));
});
