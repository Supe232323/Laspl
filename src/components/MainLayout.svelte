<script lang="ts">
  import Sidebar from "./Sidebar.svelte";
  import EntryList from "./EntryList.svelte";
  import EntryDetail from "./EntryDetail.svelte";
  import NewEntryModal from "./NewEntryModal.svelte";
  import SettingsModal from "./SettingsModal.svelte";
  import { entries, selectedId, isUnlocked, autoLockMs } from "../lib/store";
  import { addEntry, lockVault } from "../lib/tauri";

  let showNew = $state(false);
  let showSettings = $state(false);

  let lastActivity = $state(Date.now());
  let lockTimer: ReturnType<typeof setInterval> | null = null;

  function bumpActivity() {
    lastActivity = Date.now();
  }

  $effect(() => {
    const timeout = $autoLockMs;
    if (lockTimer) {
      clearInterval(lockTimer);
      lockTimer = null;
    }
    if (timeout <= 0 || !$isUnlocked) return;

    lockTimer = setInterval(async () => {
      if (Date.now() - lastActivity >= timeout) {
        try {
          await lockVault();
        } catch {
          // ignore
        }
        entries.set([]);
        selectedId.set(null);
        isUnlocked.set(false);
      }
    }, 5_000);

    return () => {
      if (lockTimer) clearInterval(lockTimer);
    };
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
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="h-screen w-screen flex overflow-hidden bg-sn-bg"
  onmousemove={bumpActivity}
  onkeydown={bumpActivity}
  onclick={bumpActivity}
>
  <Sidebar
    on:add={() => (showNew = true)}
    on:settings={() => (showSettings = true)}
  />
  <EntryList />
  <EntryDetail />
</div>

{#if showNew}
  <NewEntryModal on:save={handleSave} on:close={() => (showNew = false)} />
{/if}

{#if showSettings}
  <SettingsModal on:close={() => (showSettings = false)} />
{/if}
