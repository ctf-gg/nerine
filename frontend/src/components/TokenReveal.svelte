<script lang="ts">
  import { genToken, isError } from "../api";

  let token: string | null = $state(null);
  const onClick = async () => {
    if (token) {
      await navigator.clipboard.writeText(token);
      alert("copied");
    }
    const res = await genToken();
    if (isError(res)) return alert(res);
    token = res.token;
  };
</script>

<button onclick={onClick}>
  <span class={token ? "" : "blurred"}>
    {#if token}
      {token}
    {:else}
      eynEVeRGONnagiVEYouUPNEVeRgoNnalETyouDowN.nEVergOnnarunaroUNDandDeSErTyOUnEvERgONnAmAKeYOUcRY.NEVeRGOnNASAYgOoDbYeNevERGOnNAteLlALiEanDHURtYoU
    {/if}
  </span>
</button>

<style>
  button {
    overflow: hidden;
    max-width: 80ch;
    padding: 0.5rem;
    text-align: left;
    background: transparent;
    border: 1px solid black;
    font-family: monospace;
    cursor: pointer;
  }

  span {
    line-break: anywhere;
    transition: 200ms;
  }
  .blurred {
    filter: blur(2px);
  }
</style>
