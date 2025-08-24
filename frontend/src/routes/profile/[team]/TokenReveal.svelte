<script lang="ts">
  import { page } from "$app/state";
  import { genToken, isError } from "$lib/api";
  import { fly, fade } from "svelte/transition";

  let token: string | null = $state(null);
  let show = $state(false);
  const onClick = async () => {
    if (token) {
      await navigator.clipboard.writeText(token);
      if (!show) setTimeout(() => (show = false), 1250);
      show = true;
    }
    const res = await genToken();
    if (isError(res)) return alert(res);
    token = `${window.location.origin}/login?token=${res.token}`;
  };
</script>

<div class="button-container">
  <button onclick={onClick}>
    <span class={token ? "" : "blurred"}>
      {#if token}
        {token}
      {:else}
        {page.url
          .origin}/login?token=eynEVeRGONnagiVEYouUPNEVeRgoNnalETyouDowN.nEVergOnnarunaroUNDandDeSErTyOUnEvERgONnAmAKeYOUcRY.NEVeRGOnNASAYgOoDbYeNevERGOnNAteLlALiEanDHURtYoU
      {/if}
    </span>
  </button>

  {#if show}
    <div class="copied" in:fly={{ y: -25 }} out:fade={{ duration: 200 }}>
      Copied
    </div>
  {/if}
</div>

<style>
  .button-container {
    display: inline-grid;
    * {
      grid-row-start: 1;
      grid-column-start: 1;
    }

    .copied {
      align-self: center;
      justify-self: center;
      padding: 0.25rem;
      background: var(--bg-secondary);
      border: 1px solid var(--text-primary);
      position: absolute;
      transform: translateY(2.5rem);
    }
  }
  
  button {
    overflow: hidden;
    font-size: 1rem;
    text-align: left;
    font-family: monospace;
    cursor: pointer;
    white-space: nowrap;
  }

  span {
    line-break: anywhere;
    transition: 200ms;
  }
  .blurred {
    filter: blur(2px);
  }
</style>
