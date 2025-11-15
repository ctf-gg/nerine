<script lang="ts">
  import { fade, fly } from "svelte/transition";

  const { link } = $props();
  let show = $state(false);
  const copy = (e: MouseEvent) => {
    e.preventDefault();
    navigator.clipboard.writeText(link);
    if (!show) setTimeout(() => (show = false), 1250);
    show = true;
  };
</script>

<div class="link-container">
  <!-- svelte-ignore a11y_invalid_attribute -->
  <a href="javascript:void(0)" onclick={copy}>{link}</a>

  {#if show}
    <div class="copied" in:fly={{ y: 25 }} out:fade={{ duration: 200 }}>
      Copied
    </div>
  {/if}
</div>

<style>
  .link-container {
    display: inline-grid;
    align-items: center;
    justify-items: center;
    * {
      grid-row-start: 1;
      grid-column-start: 1;
    }

    .copied {
      padding: 0.25rem;
      background: var(--bg-primary);
      border: 1px solid var(--text-neutral);
      position: absolute;
      transform: translateY(-2rem);
      z-index: 9999;
    }
  }
</style>
