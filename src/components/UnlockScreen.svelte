<script lang="ts">
  import { onMount } from "svelte";
  import { isUnlocked, entries } from "../lib/store";
  import { unlockVault, vaultExists } from "../lib/tauri";

  let password = $state("");
  let confirmPassword = $state("");
  let error = $state("");
  let loading = $state(false);
  let exists = $state<boolean | null>(null);

  onMount(async () => {
    try {
      exists = await vaultExists();
    } catch {
      exists = true;
    }
  });

  const isCreate = $derived(exists === false);

  async function submit() {
    if (!password.trim()) {
      error = isCreate ? "Choose a master password" : "Enter your master password";
      return;
    }
    if (isCreate) {
      if (password.length < 8) {
        error = "Use at least 8 characters";
        return;
      }
      if (password !== confirmPassword) {
        error = "Passwords do not match";
        return;
      }
    }

    loading = true;
    error = "";

    try {
      const list = await unlockVault(password);
      entries.set(list);
      isUnlocked.set(true);
      password = "";
      confirmPassword = "";
    } catch (e: unknown) {
      entries.set([]);
      isUnlocked.set(false);
      error =
        typeof e === "string"
          ? e
          : e instanceof Error
            ? e.message
            : "Wrong password or vault error";
    } finally {
      loading = false;
    }
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !loading) submit();
  }
</script>

<div class="h-screen w-screen flex items-center justify-center bg-sn-bg">
  <div class="w-full max-w-sm px-6">
    <div class="text-center mb-10">
      <div class="mx-auto w-16 h-16 rounded-2xl bg-sn-accent flex items-center justify-center mb-4">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2">
          <rect x="3" y="11" width="18" height="11" rx="2" />
          <path d="M7 11V7a5 5 0 0110 0v4" />
        </svg>
      </div>
      <h1 class="text-3xl font-semibold tracking-tight text-sn-text">Laspl</h1>
      <p class="mt-2 text-sn-text-secondary text-sm">
        {#if exists === null}
          Loading…
        {:else if isCreate}
          Create your vault
        {:else}
          Unlock your vault
        {/if}
      </p>
    </div>

    {#if exists !== null}
      <div class="space-y-4">
        {#if isCreate}
          <p class="text-xs text-sn-text-muted text-center leading-relaxed">
            Pick a strong master password. It encrypts your vault locally and cannot be recovered if lost.
            A backup copy (<span class="font-mono">vault.laspl.bak</span>) is kept after each save.
          </p>
        {/if}

        <input
          type="password"
          bind:value={password}
          onkeydown={handleKey}
          placeholder={isCreate ? "Master password (min 8)" : "Master password"}
          class="w-full px-4 py-3 rounded-lg bg-sn-bg-secondary border border-sn-border
                 text-sn-text placeholder-sn-text-muted focus:outline-none
                 focus:ring-2 focus:ring-sn-accent focus:border-transparent transition"
          autofocus
        />

        {#if isCreate}
          <input
            type="password"
            bind:value={confirmPassword}
            onkeydown={handleKey}
            placeholder="Confirm master password"
            class="w-full px-4 py-3 rounded-lg bg-sn-bg-secondary border border-sn-border
                   text-sn-text placeholder-sn-text-muted focus:outline-none
                   focus:ring-2 focus:ring-sn-accent focus:border-transparent transition"
          />
        {/if}

        {#if error}
          <p class="text-red-400 text-sm text-center">{error}</p>
        {/if}

        <button
          onclick={submit}
          disabled={loading}
          class="w-full py-3 rounded-lg bg-sn-accent hover:bg-sn-accent-hover
                 text-white font-medium transition disabled:opacity-60"
        >
          {#if loading}
            {isCreate ? "Creating…" : "Unlocking…"}
          {:else}
            {isCreate ? "Create vault" : "Unlock"}
          {/if}
        </button>
      </div>
    {/if}

    <p class="mt-8 text-center text-xs text-sn-text-muted">
      Local-first • Encrypted with Argon2id + AES-256-GCM
    </p>
  </div>
</div>
