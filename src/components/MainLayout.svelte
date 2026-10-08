<script lang="ts">
  import Sidebar from "./Sidebar.svelte";
  import EntryList from "./EntryList.svelte";
  import EntryDetail from "./EntryDetail.svelte";
  import NewEntryModal from "./NewEntryModal.svelte";
  import SettingsModal from "./SettingsModal.svelte";
  import {
    entries,
    selectedId,
    selectedEntry,
    isUnlocked,
    autoLockMs,
    lastActivity,
    bumpActivity,
  } from "../lib/store";
  import { addEntry, lockVault, copyWithClear } from "../lib/tauri";

  let showNew = $state(false);
  let showSettings = $state(false);
  let lockTimer: ReturnType<typeof setInterval> | null = null;
  let tick = $state(0);

  function onActivity() {
    bumpActivity();
  }

  $effect(() => {
    const timeout = $autoLockMs;
    if (lockTimer) {
      clearInterval(lockTimer);
      lockTimer = null;
    }
    if (timeout <= 0 || !$isUnlocked) return;

    lockTimer = setInterval(async () => {
      tick++;
      if (Date.now() - $lastActivity >= timeout) {
        try {
          await lockVault();
        } catch {
          // ignore
        }
        entries.set([]);
        selectedId.set(null);
        isUnlocked.set(false);
      }
    }, 1000);

    return () => {
      if (lockTimer) clearInterval(lockTimer);
    };
  });

  const lockLabel = $derived.by(() => {
    void tick;
    const timeout = $autoLockMs;
    if (!$isUnlocked || timeout <= 0) return null;
    const left = Math.max(0, timeout - (Date.now() - $lastActivity));
    const s = Math.ceil(left / 1000);
    const m = Math.floor(s / 60);
    const r = s % 60;
    return `${m}:${r.toString().padStart(2, "0")}`;
  });

  async function handleSave(e: CustomEvent) {
    const data = e.detail;
    try {
      const entry = await addEntry(data);
      entries.update((list) => [entry, ...list]);
      selectedId.set(entry.id);
    } catch {
      const now = Date.now();
      const entry = {
        id: crypto.randomUUID(),
        ...data,
        favorite: false,
        deleted: false,
        createdAt: now,
        updatedAt: now,
      };
      entries.update((list) => [entry, ...list]);
      selectedId.set(entry.id);
    }
    showNew = false;
  }

  async function doLock() {
    try {
      await lockVault();
    } catch {
      // ignore
    }
    entries.set([]);
    selectedId.set(null);
    isUnlocked.set(false);
  }

  async function onKeydown(e: KeyboardEvent) {
    onActivity();
    const meta = e.metaKey || e.ctrlKey;
    if (!meta) return;

    if (e.key === "k" || e.key === "/") {
      e.preventDefault();
      const el = document.querySelector<HTMLInputElement>("[data-search-input]");
      el?.focus();
      el?.select();
      return;
    }
    if (e.key === "n") {
      e.preventDefault();
      showNew = true;
      return;
    }
    if (e.key === "l") {
      e.preventDefault();
      await doLock();
      return;
    }
    if (e.key === "c" && $selectedEntry) {
      const t = e.target as HTMLElement | null;
      if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable)) {
        return;
      }
      e.preventDefault();
      await copyWithClear($selectedEntry.password, 30_000);
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="h-screen w-screen flex flex-col overflow-hidden bg-sn-bg"
  onmousemove={onActivity}
  onkeydown={onKeydown}
  onclick={onActivity}
>
  {#if lockLabel}
    <div class="h-6 flex items-center justify-center text-[11px] text-sn-text-muted bg-sn-bg-secondary border-b border-sn-border">
      Auto-lock in {lockLabel}
      <span class="mx-2 text-sn-border">·</span>
      <span class="text-sn-text-muted/80">⌘K search · ⌘N new · ⌘L lock · ⌘C copy password</span>
    </div>
  {/if}

  <div class="flex-1 flex min-h-0">
    <Sidebar
      on:add={() => (showNew = true)}
      on:settings={() => (showSettings = true)}
    />
    <EntryList />
    <EntryDetail />
  </div>
</div>

{#if showNew}
  <NewEntryModal on:save={handleSave} on:close={() => (showNew = false)} />
{/if}

{#if showSettings}
  <SettingsModal on:close={() => (showSettings = false)} />
{/if}
