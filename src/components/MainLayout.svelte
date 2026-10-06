<script lang="ts">
  import Sidebar from "./Sidebar.svelte";
  import EntryList from "./EntryList.svelte";
  import EntryDetail from "./EntryDetail.svelte";
  import NewEntryModal from "./NewEntryModal.svelte";
  import { entries, selectedId } from "../lib/store";
  import { addEntry } from "../lib/tauri";

  let showNew = $state(false);

  async function handleSave(e: CustomEvent) {
    const data = e.detail;
    try {
      const entry = await addEntry(data);
      entries.update((list) => [entry, ...list]);
      selectedId.set(entry.id);
    } catch (err) {
      // Demo fallback
      const now = Date.now();
      const entry = {
        id: crypto.randomUUID(),
        ...data,
        favorite: false,
        createdAt: now,
        updatedAt: now,
      };
      entries.update((list) => [entry, ...list]);
      selectedId.set(entry.id);
    }
    showNew = false;
  }
</script>

<div class="h-screen w-screen flex overflow-hidden bg-sn-bg">
  <Sidebar on:add={() => (showNew = true)} />
  <EntryList />
  <EntryDetail />
</div>

{#if showNew}
  <NewEntryModal on:save={handleSave} on:close={() => (showNew = false)} />
{/if}
