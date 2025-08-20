<script lang="ts">
  import { event } from "$lib/event";
  import Countdown from "../../components/Countdown.svelte";
  import type { PageProps } from "./$types";
  import Challenges from "./Challenges.svelte";

  const { data }: PageProps = $props();
  const { challs } = data;
</script>

<div class="content-container">
  {#if challs}
    <Challenges {challs} />
  {:else}
    <Countdown {event} />
  {/if}
</div>
