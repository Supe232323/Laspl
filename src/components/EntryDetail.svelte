<script lang="ts">
  import { selectedEntry, entries, selectedId } from "../lib/store";
  import { updateEntry, deleteEntry } from "../lib/tauri";

  let showPassword = $state(false);

  function copy(text: string) {
    navigator.clipboard.writeText(text);
  }

  function formatDate(ts: number) {
    return new Date(ts).toLocaleString(undefined, {
      weekday: "long",
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }

  async function toggleFavorite() {
    if (!$selectedEntry) return;
    const updated = { ...$selectedEntry, favorite: !$selectedEntry.favorite, updatedAt: Date.now() };
    try {
      await updateEntry(updated);
    } catch {
      // demo mode
    }
    entries.update((list) =>
      list.map((e) => (e.id === updated.id ? updated : e))
    );
  }

  async function remove() {
    if (!$selectedEntry) return;
    const id = $selectedEntry.id;
    try {
      await deleteEntry(id);
    } catch {
      // demo mode
    }
    entries.update((list) => list.filter((e) => e.id !== id));
    selectedId.set(null);
  }
</script>

<div class="flex-1 flex flex-col bg-sn-bg min-w-0">
  {#if $selectedEntry}
    <div class="px-6 py-4 border-b border-sn-border flex items-center justify-between">
      <div>
        <h1 class="text-xl font-semibold text-sn-text">{$selectedEntry.title}</h1>
        <p class="text-sm text-sn-text-muted mt-0.5">
          {formatDate($selectedEntry.updatedAt)}
        </p>
      </div>
      <div class="flex items-center gap-1 text-sn-text-secondary">
        <button
          class="p-2 hover:bg-sn-bg-secondary rounded-md transition"
          title="Copy password"
          onclick={() => copy($selectedEntry!.password)}
        >
          📋
        </button>
        <button
          class="p-2 hover:bg-sn-bg-secondary rounded-md transition"
          title="Favorite"
          onclick={toggleFavorite}
        >
          {$selectedEntry.favorite ? "★" : "☆"}
        </button>
        <button
          class="p-2 hover:bg-sn-bg-secondary rounded-md transition text-red-400"
          title="Delete"
          onclick={remove}
        >
          🗑
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-6 py-6 space-y-6">
      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase tracking-wider mb-1.5">
          Username
        </label>
        <div class="flex items-center gap-2">
          <div class="flex-1 px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text">
            {$selectedEntry.username}
          </div>
          <button
            class="px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border
                   text-sn-text-secondary hover:text-sn-text transition"
            onclick={() => copy($selectedEntry!.username)}
          >
            Copy
          </button>
        </div>
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase tracking-wider mb-1.5">
          Password
        </label>
        <div class="flex items-center gap-2">
          <div class="flex-1 px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text font-mono">
            {showPassword ? $selectedEntry.password : "••••••••••••••••"}
          </div>
          <button
            class="px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border
                   text-sn-text-secondary hover:text-sn-text transition"
            onclick={() => (showPassword = !showPassword)}
          >
            {showPassword ? "Hide" : "Show"}
          </button>
          <button
            class="px-3 py-2.5 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover transition"
            onclick={() => copy($selectedEntry!.password)}
          >
            Copy
          </button>
        </div>
      </div>

      {#if $selectedEntry.url}
        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase tracking-wider mb-1.5">
            Website
          </label>
          <a
            href={$selectedEntry.url}
            target="_blank"
            rel="noopener"
            class="block px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border
                   text-sn-accent hover:underline truncate"
          >
            {$selectedEntry.url}
          </a>
        </div>
      {/if}

      {#if $selectedEntry.notes}
        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase tracking-wider mb-1.5">
            Notes
          </label>
          <div class="px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text whitespace-pre-wrap">
            {$selectedEntry.notes}
          </div>
        </div>
      {/if}

      <div class="pt-4 border-t border-sn-border text-xs text-sn-text-muted space-y-1">
        <div>Category: {$selectedEntry.category ?? "None"}</div>
        <div>Created: {formatDate($selectedEntry.createdAt)}</div>
      </div>
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center text-sn-text-muted">
      <div class="text-center">
        <p class="text-lg">Select an item</p>
        <p class="text-sm mt-1">or create a new one with the + button</p>
      </div>
    </div>
  {/if}
</div>
