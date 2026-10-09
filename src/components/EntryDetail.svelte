<script lang="ts">
  import { selectedEntry, entries, selectedId, currentView } from "../lib/store";
  import {
    updateEntry,
    deleteEntry,
    restoreEntry,
    touchEntry,
    generatePassword,
    copyWithClear,
    type GenerateMode,
  } from "../lib/tauri";
  import { i18n, locale } from "../lib/i18n";

  let showPassword = $state(false);
  let editing = $state(false);
  let confirmDelete = $state(false);
  let copied = $state<string | null>(null);

  let editTitle = $state("");
  let editUsername = $state("");
  let editPassword = $state("");
  let editUrl = $state("");
  let editNotes = $state("");
  let editCategory = $state("");
  let genMode = $state<GenerateMode>("password");
  let genLength = $state(20);
  let genSymbols = $state(true);
  let genExcludeAmbiguous = $state(false);
  let generating = $state(false);
  let saving = $state(false);

  $effect(() => {
    const id = $selectedEntry?.id;
    editing = false;
    confirmDelete = false;
    showPassword = false;
    void id;
  });

  function startEdit() {
    if (!$selectedEntry) return;
    const e = $selectedEntry;
    editTitle = e.title;
    editUsername = e.username;
    editPassword = e.password;
    editUrl = e.url ?? "";
    editNotes = e.notes ?? "";
    editCategory = e.category ?? "";
    editing = true;
  }

  function cancelEdit() {
    editing = false;
  }

  async function saveEdit() {
    if (!$selectedEntry || !editTitle.trim() || !editPassword) return;
    saving = true;
    const updated = {
      ...$selectedEntry,
      title: editTitle.trim(),
      username: editUsername.trim(),
      password: editPassword,
      url: editUrl.trim() || undefined,
      notes: editNotes.trim() || undefined,
      category: editCategory.trim() || undefined,
      updatedAt: Date.now(),
    };
    try {
      await updateEntry(updated);
    } catch {
      // ignore
    }
    entries.update((list) =>
      list.map((e) => (e.id === updated.id ? updated : e))
    );
    editing = false;
    saving = false;
  }

  async function gen() {
    generating = true;
    try {
      editPassword = await generatePassword({
        mode: genMode,
        length: genLength,
        symbols: genSymbols,
        excludeAmbiguous: genExcludeAmbiguous,
      });
    } catch (e) {
      console.error("Generator failed:", e);
    } finally {
      generating = false;
    }
  }

  function setGenMode(m: GenerateMode) {
    genMode = m;
    genLength = m === "passphrase" ? 5 : 20;
  }

  async function openSite() {
    if (!$selectedEntry?.url) return;
    let href = $selectedEntry.url;
    if (!/^https?:\/\//i.test(href)) href = "https://" + href;
    window.open(href, "_blank", "noopener");
    if ($selectedEntry.username) {
      await copy($selectedEntry.username, "username");
    }
  }

  async function copy(text: string, label: string) {
    await copyWithClear(text, 30_000);
    copied = label;
    setTimeout(() => {
      if (copied === label) copied = null;
    }, 1500);

    if (label === "password" && $selectedEntry) {
      const id = $selectedEntry.id;
      try {
        await touchEntry(id);
      } catch {
        // ignore
      }
      const now = Date.now();
      entries.update((list) =>
        list.map((e) => (e.id === id ? { ...e, lastUsedAt: now } : e))
      );
    }
  }

  function formatDate(ts: number) {
    if (!ts) return "—";
    const loc = $locale === "zh" ? "zh-CN" : $locale;
    return new Date(ts).toLocaleString(loc, {
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
    const updated = {
      ...$selectedEntry,
      favorite: !$selectedEntry.favorite,
      updatedAt: Date.now(),
    };
    try {
      await updateEntry(updated);
    } catch {
      // ignore
    }
    entries.update((list) =>
      list.map((e) => (e.id === updated.id ? updated : e))
    );
  }

  async function softDelete() {
    if (!$selectedEntry) return;
    const id = $selectedEntry.id;
    try {
      await deleteEntry(id);
    } catch {
      // ignore
    }
    entries.update((list) =>
      list.map((e) =>
        e.id === id ? { ...e, deleted: true, updatedAt: Date.now() } : e
      )
    );
    selectedId.set(null);
    confirmDelete = false;
  }

  async function restore() {
    if (!$selectedEntry) return;
    const id = $selectedEntry.id;
    try {
      await restoreEntry(id);
    } catch {
      // ignore
    }
    entries.update((list) =>
      list.map((e) =>
        e.id === id ? { ...e, deleted: false, updatedAt: Date.now() } : e
      )
    );
    selectedId.set(null);
  }
</script>

<div class="flex-1 flex flex-col bg-sn-bg min-w-0">
  {#if $selectedEntry}
    <div class="px-6 py-4 border-b border-sn-border flex items-center justify-between">
      <div class="min-w-0">
        <h1 class="text-xl font-semibold text-sn-text truncate">
          {editing ? $i18n("detail.editEntry") : $selectedEntry.title}
        </h1>
        {#if !editing}
          <p class="text-sm text-sn-text-muted mt-0.5">
            {formatDate($selectedEntry.updatedAt)}
          </p>
        {/if}
      </div>
      <div class="flex items-center gap-1 text-sn-text-secondary flex-shrink-0">
        {#if $selectedEntry.deleted}
          <button
            class="px-3 py-1.5 text-sm rounded-md bg-sn-accent text-white hover:bg-sn-accent-hover transition"
            onclick={restore}
          >
            {$i18n("detail.restore")}
          </button>
        {:else if !editing}
          <button
            class="p-2 hover:bg-sn-bg-secondary rounded-md transition"
            title={$i18n("detail.copyPassword")}
            onclick={() => copy($selectedEntry!.password, "password")}
          >
            {copied === "password" ? "✓" : "📋"}
          </button>
          <button
            class="p-2 hover:bg-sn-bg-secondary rounded-md transition"
            title={$i18n("detail.edit")}
            onclick={startEdit}
          >
            ✏️
          </button>
          <button
            class="p-2 hover:bg-sn-bg-secondary rounded-md transition"
            title={$i18n("detail.favorite")}
            onclick={toggleFavorite}
          >
            {$selectedEntry.favorite ? "★" : "☆"}
          </button>
          <button
            class="p-2 hover:bg-sn-bg-secondary rounded-md transition text-red-400"
            title={$i18n("detail.delete")}
            onclick={() => (confirmDelete = true)}
          >
            🗑
          </button>
        {/if}
      </div>
    </div>

    {#if confirmDelete}
      <div class="mx-6 mt-4 p-4 rounded-lg bg-red-500/10 border border-red-500/30">
        <p class="text-sm text-sn-text mb-3">
          {$i18n("detail.moveTrash", { title: $selectedEntry.title })}
        </p>
        <div class="flex gap-2 justify-end">
          <button
            class="px-3 py-1.5 text-sm rounded-md text-sn-text-secondary hover:bg-sn-bg-secondary"
            onclick={() => (confirmDelete = false)}
          >
            {$i18n("detail.cancel")}
          </button>
          <button
            class="px-3 py-1.5 text-sm rounded-md bg-red-500 text-white hover:bg-red-600"
            onclick={softDelete}
          >
            {$i18n("detail.moveToTrash")}
          </button>
        </div>
      </div>
    {/if}

    <div class="flex-1 overflow-y-auto px-6 py-6 space-y-5">
      {#if editing}
        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">{$i18n("detail.title")}</label>
          <input
            bind:value={editTitle}
            class="w-full px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text
                   focus:outline-none focus:ring-1 focus:ring-sn-accent"
          />
        </div>

        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">{$i18n("detail.username")}</label>
          <input
            bind:value={editUsername}
            class="w-full px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text
                   focus:outline-none focus:ring-1 focus:ring-sn-accent"
          />
        </div>

        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">{$i18n("detail.password")}</label>
          <div class="flex gap-2">
            <input
              bind:value={editPassword}
              type="text"
              class="flex-1 px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text font-mono
                     focus:outline-none focus:ring-1 focus:ring-sn-accent"
            />
            <button
              onclick={gen}
              disabled={generating}
              class="px-3 py-2.5 rounded-lg bg-sn-accent text-white text-sm hover:bg-sn-accent-hover disabled:opacity-60"
            >
              {generating ? "…" : $i18n("detail.generate")}
            </button>
          </div>
          <div class="flex gap-2 mt-2">
            <button
              class="flex-1 py-1.5 text-xs rounded-md border transition
                     {genMode === 'password'
                       ? 'bg-sn-accent text-white border-sn-accent'
                       : 'bg-sn-bg border-sn-border text-sn-text-muted'}"
              onclick={() => setGenMode("password")}
            >
              {$i18n("detail.random")}
            </button>
            <button
              class="flex-1 py-1.5 text-xs rounded-md border transition
                     {genMode === 'passphrase'
                       ? 'bg-sn-accent text-white border-sn-accent'
                       : 'bg-sn-bg border-sn-border text-sn-text-muted'}"
              onclick={() => setGenMode("passphrase")}
            >
              {$i18n("detail.passphrase")}
            </button>
          </div>
          <div class="flex flex-wrap items-center gap-3 mt-2 text-xs text-sn-text-muted">
            <label class="flex items-center gap-1">
              <input
                type="range"
                min={genMode === "passphrase" ? 4 : 12}
                max={genMode === "passphrase" ? 8 : 64}
                bind:value={genLength}
                class="w-20"
              />
              {genLength} {genMode === "passphrase" ? $i18n("detail.words") : $i18n("detail.chars")}
            </label>
            {#if genMode === "password"}
              <label class="flex items-center gap-1 cursor-pointer">
                <input type="checkbox" bind:checked={genSymbols} />
                {$i18n("detail.symbols")}
              </label>
              <label class="flex items-center gap-1 cursor-pointer">
                <input type="checkbox" bind:checked={genExcludeAmbiguous} />
                {$i18n("detail.noAmbiguous")}
              </label>
            {/if}
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">{$i18n("detail.url")}</label>
          <input
            bind:value={editUrl}
            class="w-full px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text
                   focus:outline-none focus:ring-1 focus:ring-sn-accent"
            placeholder="https://"
          />
        </div>

        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">{$i18n("detail.category")}</label>
          <input
            bind:value={editCategory}
            class="w-full px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text
                   focus:outline-none focus:ring-1 focus:ring-sn-accent"
            placeholder={$i18n("new.placeholder.category")}
          />
        </div>

        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">{$i18n("detail.notes")}</label>
          <textarea
            bind:value={editNotes}
            rows="3"
            class="w-full px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text
                   focus:outline-none focus:ring-1 focus:ring-sn-accent resize-none"
          ></textarea>
        </div>

        <div class="flex justify-end gap-2 pt-2">
          <button
            class="px-4 py-2 rounded-lg text-sn-text-secondary hover:bg-sn-bg-secondary transition"
            onclick={cancelEdit}
          >
            {$i18n("detail.cancel")}
          </button>
          <button
            class="px-4 py-2 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover transition disabled:opacity-60"
            onclick={saveEdit}
            disabled={saving || !editTitle.trim() || !editPassword}
          >
            {saving ? $i18n("detail.saving") : $i18n("detail.save")}
          </button>
        </div>
      {:else}
        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase tracking-wider mb-1.5">
            {$i18n("detail.username")}
          </label>
          <div class="flex items-center gap-2">
            <div class="flex-1 px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text">
              {$selectedEntry.username || "—"}
            </div>
            <button
              class="px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border
                     text-sn-text-secondary hover:text-sn-text transition"
              onclick={() => copy($selectedEntry!.username, "username")}
            >
              {copied === "username" ? $i18n("detail.copied") : $i18n("detail.copy")}
            </button>
          </div>
        </div>

        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase tracking-wider mb-1.5">
            {$i18n("detail.password")}
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
              {showPassword ? $i18n("detail.hide") : $i18n("detail.show")}
            </button>
            <button
              class="px-3 py-2.5 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover transition"
              onclick={() => copy($selectedEntry!.password, "password")}
            >
              {copied === "password" ? $i18n("detail.copied") : $i18n("detail.copy")}
            </button>
          </div>
        </div>

        {#if $selectedEntry.url}
          <div>
            <label class="block text-xs font-medium text-sn-text-muted uppercase tracking-wider mb-1.5">
              {$i18n("detail.website")}
            </label>
            <div class="flex items-center gap-2">
              <a
                href={$selectedEntry.url}
                target="_blank"
                rel="noopener"
                class="flex-1 px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border
                       text-sn-accent hover:underline truncate"
              >
                {$selectedEntry.url}
              </a>
              <button
                class="px-3 py-2.5 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover transition text-sm whitespace-nowrap"
                onclick={openSite}
                title={$i18n("detail.openTitle")}
              >
                {$i18n("detail.open")}
              </button>
            </div>
          </div>
        {/if}

        {#if $selectedEntry.notes}
          <div>
            <label class="block text-xs font-medium text-sn-text-muted uppercase tracking-wider mb-1.5">
              {$i18n("detail.notes")}
            </label>
            <div class="px-3 py-2.5 rounded-lg bg-sn-bg-secondary border border-sn-border text-sn-text whitespace-pre-wrap">
              {$selectedEntry.notes}
            </div>
          </div>
        {/if}

        <div class="pt-4 border-t border-sn-border text-xs text-sn-text-muted space-y-1">
          <div>{$i18n("detail.categoryLabel", { cat: $selectedEntry.category ?? $i18n("detail.none") })}</div>
          <div>{$i18n("detail.created")}: {formatDate($selectedEntry.createdAt)}</div>
          {#if $selectedEntry.lastUsedAt}
            <div>{$i18n("detail.lastUsed")}: {formatDate($selectedEntry.lastUsedAt)}</div>
          {/if}
          {#if $selectedEntry.deleted}
            <div class="text-red-400">{$i18n("detail.inTrash")}</div>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <div class="flex-1 flex items-center justify-center text-sn-text-muted">
      <div class="text-center">
        <p class="text-lg">
          {$currentView === "trash" ? $i18n("detail.trashEmpty") : $i18n("detail.selectItem")}
        </p>
        <p class="text-sm mt-1">
          {$currentView === "trash"
            ? $i18n("detail.trashHint")
            : $i18n("detail.orCreate")}
        </p>
      </div>
    </div>
  {/if}
</div>
