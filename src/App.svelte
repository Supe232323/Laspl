<script lang="ts">
  import { isUnlocked } from "./lib/store";
  import UnlockScreen from "./components/UnlockScreen.svelte";
  import MainLayout from "./components/MainLayout.svelte";
</script>

{#if $isUnlocked}
  <MainLayout />
{:else}
  <UnlockScreen />
{/if}
