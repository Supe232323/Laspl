<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { generatePassword, type GenerateMode } from "../lib/tauri";

  const dispatch = createEventDispatcher<{
    save: {
      title: string;
      username: string;
      password: string;
      url?: string;
      notes?: string;
      category?: string;
    };
    close: void;
  }>();

  let title = $state("");
  let username = $state("");
  let password = $state("");
  let url = $state("");
  let notes = $state("");
  let category = $state("");

  // Generator state
  let mode = $state<GenerateMode>("password");
  let length = $state(20);          // chars for password, words for passphrase
  let symbols = $state(true);
  let excludeAmbiguous = $state(false);
  let generating = $state(false);

  async function gen() {
    generating = true;
    try {
      password = await generatePassword({
        mode,
        length,
        symbols,
        excludeAmbiguous,
      });
    } catch (e) {
      console.error("Generator failed:", e);
      // No Math.random fallback — if Rust CSPRNG fails we surface the error
      password = "";
    } finally {
      generating = false;
    }
  }

  function save() {
    if (!title.trim() || !password) return;
    dispatch("save", {
      title: title.trim(),
      username: username.trim(),
      password,
      url: url.trim() || undefined,
      notes: notes.trim() || undefined,
      category: category.trim() || undefined,
    });
  }

  // Reset length defaults when switching mode
  function setMode(m: GenerateMode) {
    mode = m;
    length = m === "passphrase" ? 5 : 20;
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
  <div class="w-full max-w-md mx-4 rounded-xl bg-sn-bg-secondary border border-sn-border shadow-2xl">
    <div class="px-5 py-4 border-b border-sn-border flex items-center justify-between">
      <h2 class="text-lg font-semibold text-sn-text">New Entry</h2>
      <button
        class="text-sn-text-muted hover:text-sn-text text-xl leading-none"
        onclick={() => dispatch("close")}
      >
        ×
      </button>
    </div>

    <div class="p-5 space-y-4">
      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Title</label>
        <input
          bind:value={title}
          class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                 focus:outline-none focus:ring-1 focus:ring-sn-accent"
          placeholder="GitHub, Bank, etc."
        />
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Username</label>
        <input
          bind:value={username}
          class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                 focus:outline-none focus:ring-1 focus:ring-sn-accent"
        />
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Password</label>
        <div class="flex gap-2">
          <input
            bind:value={password}
            type="text"
            class="flex-1 px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text font-mono
                   focus:outline-none focus:ring-1 focus:ring-sn-accent"
          />
          <button
            onclick={gen}
            disabled={generating}
            class="px-3 py-2 rounded-lg bg-sn-accent text-white text-sm hover:bg-sn-accent-hover disabled:opacity-60"
          >
            {generating ? "…" : "Generate"}
          </button>
        </div>

        <!-- Mode toggle -->
        <div class="flex gap-2 mt-2">
          <button
            class="flex-1 py-1.5 text-xs rounded-md border transition
                   {mode === 'password'
                     ? 'bg-sn-accent text-white border-sn-accent'
                     : 'bg-sn-bg border-sn-border text-sn-text-muted hover:text-sn-text'}"
            onclick={() => setMode("password")}
          >
            Random
          </button>
          <button
            class="flex-1 py-1.5 text-xs rounded-md border transition
                   {mode === 'passphrase'
                     ? 'bg-sn-accent text-white border-sn-accent'
                     : 'bg-sn-bg border-sn-border text-sn-text-muted hover:text-sn-text'}"
            onclick={() => setMode("passphrase")}
          >
            Passphrase
          </button>
        </div>

        <!-- Options -->
        <div class="flex flex-wrap items-center gap-3 mt-2 text-xs text-sn-text-muted">
          <label class="flex items-center gap-1">
            <input
              type="range"
              min={mode === "passphrase" ? 4 : 12}
              max={mode === "passphrase" ? 8 : 64}
              bind:value={length}
              class="w-20"
            />
            {length}{mode === "passphrase" ? " words" : " chars"}
          </label>

          {#if mode === "password"}
            <label class="flex items-center gap-1 cursor-pointer">
              <input type="checkbox" bind:checked={symbols} />
              Symbols
            </label>
            <label class="flex items-center gap-1 cursor-pointer" title="Exclude 0/O, 1/l/I">
              <input type="checkbox" bind:checked={excludeAmbiguous} />
              No ambiguous
            </label>
          {/if}
        </div>
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">URL</label>
        <input
          bind:value={url}
          class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                 focus:outline-none focus:ring-1 focus:ring-sn-accent"
          placeholder="https://"
        />
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Category</label>
        <input
          bind:value={category}
          class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                 focus:outline-none focus:ring-1 focus:ring-sn-accent"
          placeholder="Dev, Personal, Finance…"
        />
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Notes</label>
        <textarea
          bind:value={notes}
          rows="2"
          class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text
                 focus:outline-none focus:ring-1 focus:ring-sn-accent resize-none"
        ></textarea>
      </div>
    </div>

    <div class="px-5 py-4 border-t border-sn-border flex justify-end gap-2">
      <button
        class="px-4 py-2 rounded-lg text-sn-text-secondary hover:bg-sn-bg transition"
        onclick={() => dispatch("close")}
      >
        Cancel
      </button>
      <button
        class="px-4 py-2 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover transition"
        onclick={save}
      >
        Save
      </button>
    </div>
  </div>
</div>
