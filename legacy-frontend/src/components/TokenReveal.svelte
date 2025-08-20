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
    token = `${window.location.origin}/login?token=${res.token}`;
  };
</script>

<button onclick={onClick}>
  <span class={token ? "" : "blurred"}>
    {#if token}
      {token}
    {:else}
      https://play.ctf.gg/login?token=eynEVeRGONnagiVEYouUPNEVeRgoNnalETyouDowN.nEVergOnnarunaroUNDandDeSErTyOUnEvERgONnAmAKeYOUcRY.NEVeRGOnNASAYgOoDbYeNevERGOnNAteLlALiEanDHURtYoU
    {/if}
  </span>
</button>

<style>
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
