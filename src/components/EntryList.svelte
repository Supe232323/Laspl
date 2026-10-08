<script lang="ts">
  import { filteredEntries, selectedId, searchQuery } from "../lib/store";
  import type { PasswordEntry } from "../lib/types";

  function select(entry: PasswordEntry) {
    selectedId.set(entry.id);
  }

  function formatDate(ts: number) {
    if (!ts) return "";
    return new Date(ts).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }
</script>

<div class="w-80 flex-shrink-0 border-r border-sn-border flex flex-col bg-sn-bg">
  <div class="p-3 border-b border-sn-border">
    <div class="relative">
      <input
        type="text"
        data-search-input
        bind:value={$searchQuery}
        placeholder="Search… ⌘K"
        class="w-full pl-9 pr-3 py-2 text-sm rounded-lg bg-sn-bg-secondary border border-sn-border
               text-sn-text placeholder-sn-text-muted focus:outline-none focus:ring-1 focus:ring-sn-accent"
      />
      <span class="absolute left-3 top-1/2 -translate-y-1/2 text-sn-text-muted text-sm">🔍</span>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto">
    {#each $filteredEntries as entry (entry.id)}
      <button
        class="w-full text-left px-4 py-3 border-b border-sn-border transition
               {$selectedId === entry.id
                 ? 'bg-sn-highlight border-l-2 border-l-sn-accent'
                 : 'hover:bg-sn-bg-secondary border-l-2 border-l-transparent'}"
        onclick={() => select(entry)}
      >
        <div class="flex items-start gap-2">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-1.5">
              {#if entry.favorite}
                <span class="text-yellow-400 text-xs">★</span>
              {/if}
              <span class="font-medium text-sn-text truncate">{entry.title}</span>
            </div>
            <div class="text-sm text-sn-text-secondary truncate mt-0.5">
              {entry.username}
            </div>
            <div class="text-xs text-sn-text-muted mt-1">
              {formatDate(entry.updatedAt)}
            </div>
          </div>
        </div>
      </button>
    {:else}
      <div class="p-8 text-center text-sn-text-muted text-sm">
        No items found
      </div>
    {/each}
  </div>
</div>
