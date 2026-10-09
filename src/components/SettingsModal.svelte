<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { autoLockMs, isUnlocked, entries, selectedId } from "../lib/store";
  import {
    changeMasterPassword,
    exportVault,
    importVault,
    lockVault,
  } from "../lib/tauri";
  import { i18n, locale, LOCALES, type Locale } from "../lib/i18n";

  const dispatch = createEventDispatcher<{ close: void }>();

  let tab = $state<"security" | "backup">("security");
  let currentPw = $state("");
  let newPw = $state("");
  let confirmPw = $state("");
  let pwError = $state("");
  let pwSuccess = $state("");
  let pwLoading = $state(false);

  let exportStatus = $state("");
  let importStatus = $state("");
  let importData = $state("");

  const lockOptions = $derived([
    { label: $i18n("settings.lock.1m"), value: 60_000 },
    { label: $i18n("settings.lock.5m"), value: 5 * 60_000 },
    { label: $i18n("settings.lock.15m"), value: 15 * 60_000 },
    { label: $i18n("settings.lock.30m"), value: 30 * 60_000 },
    { label: $i18n("settings.lock.never"), value: 0 },
  ]);

  async function changePassword() {
    pwError = "";
    pwSuccess = "";
    if (!currentPw || !newPw) {
      pwError = $i18n("settings.pwFill");
      return;
    }
    if (newPw.length < 8) {
      pwError = $i18n("settings.pwMin");
      return;
    }
    if (newPw !== confirmPw) {
      pwError = $i18n("settings.pwMatch");
      return;
    }
    pwLoading = true;
    try {
      await changeMasterPassword(currentPw, newPw);
      pwSuccess = $i18n("settings.pwChanged");
      currentPw = "";
      newPw = "";
      confirmPw = "";
    } catch (e: unknown) {
      pwError =
        typeof e === "string"
          ? e
          : e instanceof Error
            ? e.message
            : $i18n("settings.pwMatch");
    } finally {
      pwLoading = false;
    }
  }

  async function doExport() {
    exportStatus = "";
    try {
      const data = await exportVault();
      const blob = new Blob([data], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `laspl-backup-${new Date().toISOString().slice(0, 10)}.laspl`;
      a.click();
      URL.revokeObjectURL(url);
      exportStatus = $i18n("settings.exportOk");
    } catch (e: unknown) {
      exportStatus =
        typeof e === "string"
          ? e
          : e instanceof Error
            ? e.message
            : "Export failed";
    }
  }

  async function doImport() {
    importStatus = "";
    if (!importData.trim()) {
      importStatus = $i18n("settings.importEmpty");
      return;
    }
    try {
      await importVault(importData.trim());
      importStatus = $i18n("settings.importOk");
      try {
        await lockVault();
      } catch {
        // ignore
      }
      entries.set([]);
      selectedId.set(null);
      isUnlocked.set(false);
      dispatch("close");
    } catch (e: unknown) {
      importStatus =
        typeof e === "string"
          ? e
          : e instanceof Error
            ? e.message
            : "Import failed";
    }
  }

  function onFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      importData = String(reader.result ?? "");
    };
    reader.readAsText(file);
  }

  function onLocaleChange(e: Event) {
    const v = (e.target as HTMLSelectElement).value as Locale;
    locale.set(v);
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
  <div class="w-full max-w-md mx-4 rounded-xl bg-sn-bg-secondary border border-sn-border shadow-2xl max-h-[90vh] overflow-y-auto">
    <div class="px-5 py-4 border-b border-sn-border flex items-center justify-between sticky top-0 bg-sn-bg-secondary">
      <h2 class="text-lg font-semibold text-sn-text">{$i18n("settings.title")}</h2>
      <button
        class="text-sn-text-muted hover:text-sn-text text-xl leading-none"
        onclick={() => dispatch("close")}
      >
        ×
      </button>
    </div>

    <div class="flex border-b border-sn-border">
      <button
        class="flex-1 py-2.5 text-sm transition
               {tab === 'security'
                 ? 'text-sn-accent border-b-2 border-sn-accent'
                 : 'text-sn-text-muted hover:text-sn-text'}"
        onclick={() => (tab = "security")}
      >
        {$i18n("settings.security")}
      </button>
      <button
        class="flex-1 py-2.5 text-sm transition
               {tab === 'backup'
                 ? 'text-sn-accent border-b-2 border-sn-accent'
                 : 'text-sn-text-muted hover:text-sn-text'}"
        onclick={() => (tab = "backup")}
      >
        {$i18n("settings.backup")}
      </button>
    </div>

    <div class="p-5 space-y-5">
      {#if tab === "security"}
        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">
            {$i18n("settings.language")}
          </label>
          <select
            class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                   focus:outline-none focus:ring-1 focus:ring-sn-accent"
            value={$locale}
            onchange={onLocaleChange}
          >
            {#each LOCALES as loc}
              <option value={loc.id}>{loc.native}</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">
            {$i18n("settings.autoLock")}
          </label>
          <select
            class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                   focus:outline-none focus:ring-1 focus:ring-sn-accent"
            value={$autoLockMs}
            onchange={(e) =>
              autoLockMs.set(Number((e.target as HTMLSelectElement).value))}
          >
            {#each lockOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <div class="border-t border-sn-border pt-4">
          <h3 class="text-sm font-medium text-sn-text mb-3">{$i18n("settings.changePw")}</h3>
          <div class="space-y-3">
            <input
              type="password"
              bind:value={currentPw}
              placeholder={$i18n("settings.currentPw")}
              class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                     focus:outline-none focus:ring-1 focus:ring-sn-accent"
            />
            <input
              type="password"
              bind:value={newPw}
              placeholder={$i18n("settings.newPw")}
              class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                     focus:outline-none focus:ring-1 focus:ring-sn-accent"
            />
            <input
              type="password"
              bind:value={confirmPw}
              placeholder={$i18n("settings.confirmPw")}
              class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                     focus:outline-none focus:ring-1 focus:ring-sn-accent"
            />
            {#if pwError}
              <p class="text-red-400 text-sm">{pwError}</p>
            {/if}
            {#if pwSuccess}
              <p class="text-green-400 text-sm">{pwSuccess}</p>
            {/if}
            <button
              onclick={changePassword}
              disabled={pwLoading}
              class="w-full py-2 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover
                     transition disabled:opacity-60"
            >
              {pwLoading ? $i18n("settings.changing") : $i18n("settings.changeBtn")}
            </button>
          </div>
        </div>
      {:else}
        <div>
          <h3 class="text-sm font-medium text-sn-text mb-2">{$i18n("settings.export")}</h3>
          <p class="text-xs text-sn-text-muted mb-3">
            {$i18n("settings.exportHint")}
          </p>
          <button
            onclick={doExport}
            class="w-full py-2 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover transition"
          >
            {$i18n("settings.exportBtn")}
          </button>
          {#if exportStatus}
            <p class="text-sm text-sn-text-muted mt-2">{exportStatus}</p>
          {/if}
        </div>

        <div class="border-t border-sn-border pt-4">
          <h3 class="text-sm font-medium text-sn-text mb-2">{$i18n("settings.import")}</h3>
          <p class="text-xs text-sn-text-muted mb-3">
            {$i18n("settings.importHint")}
          </p>
          <input
            type="file"
            accept=".laspl,.json,text/plain"
            onchange={onFile}
            class="block w-full text-sm text-sn-text-muted mb-2 file:mr-3 file:py-1.5 file:px-3
                   file:rounded-md file:border-0 file:bg-sn-bg file:text-sn-text"
          />
          <textarea
            bind:value={importData}
            rows="3"
            placeholder={$i18n("settings.importPaste")}
            class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text text-xs
                   font-mono focus:outline-none focus:ring-1 focus:ring-sn-accent resize-none mb-2"
          ></textarea>
          <button
            onclick={doImport}
            class="w-full py-2 rounded-lg border border-sn-border text-sn-text
                   hover:bg-sn-bg transition"
          >
            {$i18n("settings.importBtn")}
          </button>
          {#if importStatus}
            <p class="text-sm text-sn-text-muted mt-2">{importStatus}</p>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>
