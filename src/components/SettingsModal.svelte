<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { autoLockMs, isUnlocked, entries, selectedId } from "../lib/store";
  import {
    changeMasterPassword,
    exportVault,
    importVault,
    lockVault,
  } from "../lib/tauri";

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

  const lockOptions = [
    { label: "1 minute", value: 60_000 },
    { label: "5 minutes", value: 5 * 60_000 },
    { label: "15 minutes", value: 15 * 60_000 },
    { label: "30 minutes", value: 30 * 60_000 },
    { label: "Never", value: 0 },
  ];

  async function changePassword() {
    pwError = "";
    pwSuccess = "";
    if (!currentPw || !newPw) {
      pwError = "Fill in all fields";
      return;
    }
    if (newPw.length < 8) {
      pwError = "New password must be at least 8 characters";
      return;
    }
    if (newPw !== confirmPw) {
      pwError = "New passwords do not match";
      return;
    }
    pwLoading = true;
    try {
      await changeMasterPassword(currentPw, newPw);
      pwSuccess = "Master password changed";
      currentPw = "";
      newPw = "";
      confirmPw = "";
    } catch (e: unknown) {
      pwError =
        typeof e === "string"
          ? e
          : e instanceof Error
            ? e.message
            : "Failed to change password";
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
      exportStatus = "Backup downloaded";
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
      importStatus = "Paste vault data or choose a file";
      return;
    }
    try {
      await importVault(importData.trim());
      importStatus = "Imported — lock and unlock with the vault's master password";
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
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
  <div class="w-full max-w-md mx-4 rounded-xl bg-sn-bg-secondary border border-sn-border shadow-2xl">
    <div class="px-5 py-4 border-b border-sn-border flex items-center justify-between">
      <h2 class="text-lg font-semibold text-sn-text">Settings</h2>
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
        Security
      </button>
      <button
        class="flex-1 py-2.5 text-sm transition
               {tab === 'backup'
                 ? 'text-sn-accent border-b-2 border-sn-accent'
                 : 'text-sn-text-muted hover:text-sn-text'}"
        onclick={() => (tab = "backup")}
      >
        Backup
      </button>
    </div>

    <div class="p-5 space-y-5">
      {#if tab === "security"}
        <div>
          <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1.5">
            Auto-lock after
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
          <h3 class="text-sm font-medium text-sn-text mb-3">Change master password</h3>
          <div class="space-y-3">
            <input
              type="password"
              bind:value={currentPw}
              placeholder="Current password"
              class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                     focus:outline-none focus:ring-1 focus:ring-sn-accent"
            />
            <input
              type="password"
              bind:value={newPw}
              placeholder="New password"
              class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                     focus:outline-none focus:ring-1 focus:ring-sn-accent"
            />
            <input
              type="password"
              bind:value={confirmPw}
              placeholder="Confirm new password"
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
              {pwLoading ? "Changing…" : "Change password"}
            </button>
          </div>
        </div>
      {:else}
        <div>
          <h3 class="text-sm font-medium text-sn-text mb-2">Export vault</h3>
          <p class="text-xs text-sn-text-muted mb-3">
            Downloads an encrypted backup of your vault file. Keep it safe.
          </p>
          <button
            onclick={doExport}
            class="w-full py-2 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover transition"
          >
            Download backup
          </button>
          {#if exportStatus}
            <p class="text-sm text-sn-text-muted mt-2">{exportStatus}</p>
          {/if}
        </div>

        <div class="border-t border-sn-border pt-4">
          <h3 class="text-sm font-medium text-sn-text mb-2">Import vault</h3>
          <p class="text-xs text-sn-text-muted mb-3">
            Replaces the current vault with a backup. You will need to unlock with the backup's master password.
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
            placeholder="Or paste vault JSON here…"
            class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text text-xs
                   font-mono focus:outline-none focus:ring-1 focus:ring-sn-accent resize-none mb-2"
          ></textarea>
          <button
            onclick={doImport}
            class="w-full py-2 rounded-lg border border-sn-border text-sn-text
                   hover:bg-sn-bg transition"
          >
            Import
          </button>
          {#if importStatus}
            <p class="text-sm text-sn-text-muted mt-2">{importStatus}</p>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>
