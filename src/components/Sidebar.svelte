<script lang="ts">
  import { currentView, currentCategory, isUnlocked, entries } from "../lib/store";
  import { lockVault } from "../lib/tauri";
  import type { View } from "../lib/types";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher<{ add: void }>();

  const views: { id: View; label: string; icon: string }[] = [
    { id: "all", label: "All Items", icon: "🔐" },
    { id: "favorites", label: "Favorites", icon: "★" },
    { id: "recent", label: "Recently Used", icon: "🕒" },
    { id: "trash", label: "Trash", icon: "🗑" },
  ];

  const categories = ["Dev", "Personal", "Finance"];

  function setView(view: View, category: string | null = null) {
    currentView.set(view);
    currentCategory.set(category);
  }

  async function lock() {
    try {
      await lockVault();
    } catch {
      // ignore in demo mode
    }
    entries.set([]);
    isUnlocked.set(false);
  }
</script>

<aside class="w-56 flex-shrink-0 bg-sn-bg border-r border-sn-border flex flex-col h-full">
  <div class="px-4 py-4 flex items-center justify-between">
    <span class="font-semibold text-sn-text tracking-tight">Laspl</span>
    <button
      class="w-7 h-7 rounded-full bg-sn-accent text-white text-sm flex items-center justify-center
             hover:bg-sn-accent-hover transition"
      title="Add new"
      onclick={() => dispatch("add")}
    >
      +
    </button>
  </div>

  <div class="px-3 mb-3">
    <input
      type="text"
      placeholder="Search tags…"
      class="w-full px-3 py-1.5 text-sm rounded-full bg-sn-bg-secondary border border-sn-border
             text-sn-text placeholder-sn-text-muted focus:outline-none focus:ring-1 focus:ring-sn-accent"
    />
  </div>

  <div class="px-2 flex-1 overflow-y-auto">
    <div class="text-xs font-medium text-sn-text-muted uppercase tracking-wider px-2 mb-1">
      Views
    </div>
    {#each views as v}
      <button
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm transition
               {$currentView === v.id && !$currentCategory
                 ? 'bg-sn-highlight text-sn-accent font-medium'
                 : 'text-sn-text-secondary hover:bg-sn-bg-secondary'}"
        onclick={() => setView(v.id)}
      >
        <span class="text-base w-5 text-center">{v.icon}</span>
        <span class="flex-1 text-left">{v.label}</span>
      </button>
    {/each}

    <div class="mt-5 text-xs font-medium text-sn-text-muted uppercase tracking-wider px-2 mb-1 flex items-center justify-between">
      <span>Categories</span>
      <button class="text-sn-text-muted hover:text-sn-text">+</button>
    </div>
    {#each categories as cat}
      <button
        class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm transition
               {$currentView === 'category' && $currentCategory === cat
                 ? 'bg-sn-highlight text-sn-accent font-medium'
                 : 'text-sn-text-secondary hover:bg-sn-bg-secondary'}"
        onclick={() => setView("category", cat)}
      >
        <span class="text-base w-5 text-center">📁</span>
        <span class="flex-1 text-left">{cat}</span>
      </button>
    {/each}
  </div>

  <div class="p-3 border-t border-sn-border">
    <button
      onclick={lock}
      class="w-full py-2 text-sm text-sn-text-secondary hover:text-sn-text
             hover:bg-sn-bg-secondary rounded-md transition"
    >
      Lock Vault
    </button>
  </div>
</aside>
