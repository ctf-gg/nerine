<script lang="ts">
  import Countdown from "../../components/Countdown.svelte";
  import type { PageProps } from "./$types";
  import Challenges from "./Challenges.svelte";

  const { data }: PageProps = $props();
  const { challs, event } = data;
</script>

<div class="content-container">
  {#if challs}
    <Challenges {challs} {event} />
  {:else}
    <Countdown {event} />
  {/if}
</div>
