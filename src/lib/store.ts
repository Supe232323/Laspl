import { writable, derived } from "svelte/store";
import type { PasswordEntry, View } from "./types";

export const isUnlocked = writable(false);
export const entries = writable<PasswordEntry[]>([]);
export const selectedId = writable<string | null>(null);
export const searchQuery = writable("");
export const currentView = writable<View>("all");
export const currentCategory = writable<string | null>(null);

export const filteredEntries = derived(
  [entries, searchQuery, currentView, currentCategory],
  ([$entries, $search, $view, $category]) => {
    let list = $entries;

    if ($view === "favorites") {
      list = list.filter((e) => e.favorite);
    } else if ($view === "trash") {
      // placeholder for soft-delete later
      list = [];
    } else if ($view === "category" && $category) {
      list = list.filter((e) => e.category === $category);
    }

    if ($search.trim()) {
      const q = $search.toLowerCase();
      list = list.filter(
        (e) =>
          e.title.toLowerCase().includes(q) ||
          e.username.toLowerCase().includes(q) ||
          (e.url && e.url.toLowerCase().includes(q)) ||
          (e.notes && e.notes.toLowerCase().includes(q))
      );
    }

    // newest first
    return list.sort((a, b) => b.updatedAt - a.updatedAt);
  }
);

export const selectedEntry = derived(
  [entries, selectedId],
  ([$entries, $id]) => $entries.find((e) => e.id === $id) ?? null
);
