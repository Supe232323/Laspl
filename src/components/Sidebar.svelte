<script lang="ts">
  import {
    currentView,
    currentCategory,
    isUnlocked,
    entries,
    searchQuery,
    categories,
    selectedId,
  } from "../lib/store";
  import { lockVault, purgeDeleted } from "../lib/tauri";
  import type { View } from "../lib/types";
  import { createEventDispatcher } from "svelte";
  import { i18n } from "../lib/i18n";

  const dispatch = createEventDispatcher<{ add: void; settings: void }>();

  const views = $derived([
    { id: "all" as View, label: $i18n("sidebar.all"), icon: "🔐" },
    { id: "favorites" as View, label: $i18n("sidebar.favorites"), icon: "★" },
    { id: "recent" as View, label: $i18n("sidebar.recent"), icon: "🕒" },
    { id: "trash" as View, label: $i18n("sidebar.trash"), icon: "🗑" },
  ]);

  let confirmPurge = $state(false);

  function setView(view: View, category: string | null = null) {
    currentView.set(view);
    currentCategory.set(category);
    selectedId.set(null);
  }

  async function lock() {
    try {
      await lockVault();
    } catch {
      // ignore
    }
    entries.set([]);
    selectedId.set(null);
    isUnlocked.set(false);
  }

  async function emptyTrash() {
    try {
      await purgeDeleted();
    } catch {
      // demo
    }
    entries.update((list) => list.filter((e) => !e.deleted));
    confirmPurge = false;
    selectedId.set(null);
  }

  let trashCount = $derived($entries.filter((e) => e.deleted).length);
</script>

<aside class="w-56 flex-shrink-0 bg-sn-bg border-r border-sn-border flex flex-col h-full">
  <div class="px-4 py-4 flex items-center justify-between">
    <span class="font-semibold text-sn-text tracking-tight">{$i18n("app.name")}</span>
    <button
      class="w-7 h-7 rounded-full bg-sn-accent text-white text-sm flex items-center justify-center
             hover:bg-sn-accent-hover transition"
      title={$i18n("sidebar.add")}
      onclick={() => dispatch("add")}
    >
      +
    </button>
  </div>

  <div class="px-3 mb-3">
    <input
      type="text"
      bind:value={$searchQuery}
      placeholder={$i18n("sidebar.search")}
      class="w-full px-3 py-1.5 text-sm rounded-full bg-sn-bg-secondary border border-sn-border
             text-sn-text placeholder-sn-text-muted focus:outline-none focus:ring-1 focus:ring-sn-accent"
    />
  </div>

  <div class="px-2 flex-1 overflow-y-auto">
    <div class="text-xs font-medium text-sn-text-muted uppercase tracking-wider px-2 mb-1">
      {$i18n("sidebar.views")}
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
        {#if v.id === "trash" && trashCount > 0}
          <span class="text-xs text-sn-text-muted">{trashCount}</span>
        {/if}
      </button>
    {/each}

    {#if $categories.length > 0}
      <div
        class="mt-5 text-xs font-medium text-sn-text-muted uppercase tracking-wider px-2 mb-1"
      >
        {$i18n("sidebar.categories")}
      </div>
      {#each $categories as cat}
        <button
          class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm transition
                 {$currentView === 'category' && $currentCategory === cat
                   ? 'bg-sn-highlight text-sn-accent font-medium'
                   : 'text-sn-text-secondary hover:bg-sn-bg-secondary'}"
          onclick={() => setView("category", cat)}
        >
          <span class="text-base w-5 text-center">📁</span>
          <span class="flex-1 text-left truncate">{cat}</span>
        </button>
      {/each}
    {/if}

    {#if $currentView === "trash" && trashCount > 0}
      <div class="mt-4 px-2">
        {#if confirmPurge}
          <div class="p-2 rounded-md bg-red-500/10 border border-red-500/30 text-xs">
            <p class="text-sn-text mb-2">{$i18n("sidebar.purgeConfirm")}</p>
            <div class="flex gap-1">
              <button
                class="flex-1 py-1 rounded text-sn-text-muted hover:bg-sn-bg-secondary"
                onclick={() => (confirmPurge = false)}
              >
                {$i18n("sidebar.cancel")}
              </button>
              <button
                class="flex-1 py-1 rounded bg-red-500 text-white"
                onclick={emptyTrash}
              >
                {$i18n("sidebar.empty")}
              </button>
            </div>
          </div>
        {:else}
          <button
            class="w-full py-1.5 text-xs text-red-400 hover:bg-sn-bg-secondary rounded-md transition"
            onclick={() => (confirmPurge = true)}
          >
            {$i18n("sidebar.emptyTrash")}
          </button>
        {/if}
      </div>
    {/if}
  </div>

  <div class="p-3 border-t border-sn-border space-y-1">
    <button
      onclick={() => dispatch("settings")}
      class="w-full py-2 text-sm text-sn-text-secondary hover:text-sn-text
             hover:bg-sn-bg-secondary rounded-md transition"
    >
      {$i18n("sidebar.settings")}
    </button>
    <button
      onclick={lock}
      class="w-full py-2 text-sm text-sn-text-secondary hover:text-sn-text
             hover:bg-sn-bg-secondary rounded-md transition"
    >
      {$i18n("sidebar.lock")}
    </button>
  </div>
</aside>
