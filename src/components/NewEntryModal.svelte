<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { entries } from "../lib/store";
  import { generatePassword, type GenerateMode } from "../lib/tauri";
  import { scorePassword, findDuplicatePasswords } from "../lib/strength";

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

  let mode = $state<GenerateMode>("password");
  let length = $state(20);
  let symbols = $state(true);
  let excludeAmbiguous = $state(false);
  let separator = $state("-");
  let generating = $state(false);
  let dupOverride = $state(false);

  const SEPARATORS = [
    { value: "-", label: "-" },
    { value: " ", label: "space" },
    { value: ".", label: "." },
    { value: "_", label: "_" },
    { value: "", label: "none" },
  ];

  // Theoretical entropy from generator parameters (bits)
  // EFF long list = 7776 words → log2(7776) ≈ 12.925 bits/word
  const entropyBits = $derived.by(() => {
    if (mode === "passphrase") {
      return Math.round(length * 12.925);
    }
    // charset size estimate
    let size = 50; // base without ambiguous
    if (!excludeAmbiguous) size += 5;
    if (symbols) size += 24;
    return Math.round(length * Math.log2(size));
  });

  const entropyLabel = $derived.by(() => {
    if (entropyBits >= 128) return "Excellent";
    if (entropyBits >= 80) return "Strong";
    if (entropyBits >= 60) return "Good";
    if (entropyBits >= 40) return "Fair";
    return "Weak";
  });

  const entropyColor = $derived.by(() => {
    if (entropyBits >= 128) return "bg-emerald-500";
    if (entropyBits >= 80) return "bg-green-500";
    if (entropyBits >= 60) return "bg-yellow-500";
    if (entropyBits >= 40) return "bg-orange-500";
    return "bg-red-500";
  });

  const strength = $derived(scorePassword(password));
  const dups = $derived(findDuplicatePasswords(password, $entries));

  async function gen() {
    generating = true;
    dupOverride = false;
    try {
      password = await generatePassword({
        mode,
        length,
        symbols,
        excludeAmbiguous,
        separator,
      });
    } catch (e) {
      console.error("Generator failed:", e);
      password = "";
    } finally {
      generating = false;
    }
  }

  function save() {
    if (!title.trim() || !password) return;
    if (dups.length > 0 && !dupOverride) {
      dupOverride = true;
      return;
    }
    dispatch("save", {
      title: title.trim(),
      username: username.trim(),
      password,
      url: url.trim() || undefined,
      notes: notes.trim() || undefined,
      category: category.trim() || undefined,
    });
  }

  function setMode(m: GenerateMode) {
    mode = m;
    length = m === "passphrase" ? 5 : 20;
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
  <div class="w-full max-w-md mx-4 rounded-xl bg-sn-bg-secondary border border-sn-border shadow-2xl max-h-[90vh] overflow-y-auto">
    <div class="px-5 py-4 border-b border-sn-border flex items-center justify-between sticky top-0 bg-sn-bg-secondary">
      <h2 class="text-lg font-semibold text-sn-text">New Entry</h2>
      <button class="text-sn-text-muted hover:text-sn-text text-xl leading-none" onclick={() => dispatch("close")}>×</button>
    </div>

    <div class="p-5 space-y-4">
      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Title</label>
        <input bind:value={title} class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text focus:outline-none focus:ring-1 focus:ring-sn-accent" placeholder="GitHub, Bank, etc." />
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Username</label>
        <input bind:value={username} class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text focus:outline-none focus:ring-1 focus:ring-sn-accent" />
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Password</label>
        <div class="flex gap-2">
          <input bind:value={password} type="text" class="flex-1 px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text font-mono focus:outline-none focus:ring-1 focus:ring-sn-accent" oninput={() => (dupOverride = false)} />
          <button onclick={gen} disabled={generating} class="px-3 py-2 rounded-lg bg-sn-accent text-white text-sm hover:bg-sn-accent-hover disabled:opacity-60">{generating ? "…" : "Generate"}</button>
        </div>

        <!-- Strength (heuristic) -->
        {#if password}
          <div class="mt-2 flex items-center gap-2">
            <div class="flex-1 h-1.5 rounded-full bg-sn-bg overflow-hidden flex gap-0.5">
              {#each [1, 2, 3, 4] as i}
                <div class="flex-1 rounded-full transition {strength.score >= i ? strength.color : 'bg-sn-border'}"></div>
              {/each}
            </div>
            <span class="text-xs text-sn-text-muted w-12 text-right">{strength.label}</span>
          </div>
        {/if}

        <!-- Entropy meter (theoretical from generator params) -->
        <div class="mt-2 flex items-center gap-2">
          <div class="flex-1 h-1.5 rounded-full bg-sn-bg overflow-hidden">
            <div
              class="h-full rounded-full transition-all {entropyColor}"
              style="width: {Math.min(100, (entropyBits / 128) * 100)}%"
            ></div>
          </div>
          <span class="text-xs text-sn-text-muted whitespace-nowrap">{entropyBits} bit · {entropyLabel}</span>
        </div>

        {#if dups.length > 0}
          <p class="mt-2 text-xs text-yellow-400">
            Same password as: {dups.slice(0, 3).join(", ")}{dups.length > 3 ? "…" : ""}.
            {dupOverride ? " Click Save again to use it anyway." : ""}
          </p>
        {/if}

        <div class="flex gap-2 mt-2">
          <button class="flex-1 py-1.5 text-xs rounded-md border transition {mode === 'password' ? 'bg-sn-accent text-white border-sn-accent' : 'bg-sn-bg border-sn-border text-sn-text-muted'}" onclick={() => setMode("password")}>Random</button>
          <button class="flex-1 py-1.5 text-xs rounded-md border transition {mode === 'passphrase' ? 'bg-sn-accent text-white border-sn-accent' : 'bg-sn-bg border-sn-border text-sn-text-muted'}" onclick={() => setMode("passphrase")}>Passphrase</button>
        </div>

        <div class="flex flex-wrap items-center gap-3 mt-2 text-xs text-sn-text-muted">
          <label class="flex items-center gap-1">
            <input type="range" min={mode === "passphrase" ? 4 : 12} max={mode === "passphrase" ? 8 : 64} bind:value={length} class="w-20" />
            {length}{mode === "passphrase" ? " words" : " chars"}
          </label>

          {#if mode === "password"}
            <label class="flex items-center gap-1 cursor-pointer"><input type="checkbox" bind:checked={symbols} /> Symbols</label>
            <label class="flex items-center gap-1 cursor-pointer"><input type="checkbox" bind:checked={excludeAmbiguous} /> No ambiguous</label>
          {:else}
            <label class="flex items-center gap-1">
              Sep
              <select bind:value={separator} class="bg-sn-bg border border-sn-border rounded px-1.5 py-0.5 text-sn-text">
                {#each SEPARATORS as s}
                  <option value={s.value}>{s.label}</option>
                {/each}
              </select>
            </label>
          {/if}
        </div>
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">URL</label>
        <input bind:value={url} class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text focus:outline-none focus:ring-1 focus:ring-sn-accent" placeholder="https://" />
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Category</label>
        <input bind:value={category} class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text focus:outline-none focus:ring-1 focus:ring-sn-accent" placeholder="Dev, Personal, Finance…" />
      </div>

      <div>
        <label class="block text-xs font-medium text-sn-text-muted uppercase mb-1">Notes</label>
        <textarea bind:value={notes} rows="2" class="w-full px-3 py-2 rounded-lg bg-sn-bg border border-sn-border text-sn-text focus:outline-none focus:ring-1 focus:ring-sn-accent resize-none"></textarea>
      </div>
    </div>

    <div class="px-5 py-4 border-t border-sn-border flex justify-end gap-2">
      <button class="px-4 py-2 rounded-lg text-sn-text-secondary hover:bg-sn-bg transition" onclick={() => dispatch("close")}>Cancel</button>
      <button class="px-4 py-2 rounded-lg bg-sn-accent text-white hover:bg-sn-accent-hover transition" onclick={save}>
        {dups.length > 0 && !dupOverride ? "Save anyway?" : "Save"}
      </button>
    </div>
  </div>
</div>
