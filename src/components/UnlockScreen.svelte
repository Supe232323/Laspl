<script lang="ts">
  import { isUnlocked, entries } from "../lib/store";
  import { unlockVault } from "../lib/tauri";

  let password = $state("");
  let error = $state("");
  let loading = $state(false);

<script lang="ts">
  import { isUnlocked, entries } from "../lib/store";
  import { unlockVault } from "../lib/tauri";

  let password = $state("");
  let error = $state("");
  let loading = $state(false);

  async function unlock() {
    if (!password.trim()) {
      error = "Enter your master password";
      return;
    }

    loading = true;
    error = "";

    try {
      const list = await unlockVault(password);
      entries.set(list);
      isUnlocked.set(true);
      password = "";
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
    if (e.key === "Enter" && !loading) {
      unlock();
    }
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
      <p class="mt-2 text-sn-text-secondary text-sm">Unlock your vault</p>
    </div>

    <div class="space-y-4">
      <input
        type="password"
        bind:value={password}
        onkeydown={handleKey}
        placeholder="Master password"
        class="w-full px-4 py-3 rounded-lg bg-sn-bg-secondary border border-sn-border
               text-sn-text placeholder-sn-text-muted focus:outline-none
               focus:ring-2 focus:ring-sn-accent focus:border-transparent
               transition"
        autofocus
      />

      {#if error}
        <p class="text-red-400 text-sm text-center">{error}</p>
      {/if}

      <button
        onclick={unlock}
        disabled={loading}
        class="w-full py-3 rounded-lg bg-sn-accent hover:bg-sn-accent-hover
               text-white font-medium transition disabled:opacity-60"
      >
        {loading ? "Unlocking…" : "Unlock"}
      </button>
    </div>

    <p class="mt-8 text-center text-xs text-sn-text-muted">
      Local-first • Encrypted with Argon2id + AES-256-GCM
    </p>
  </div>
</div>
