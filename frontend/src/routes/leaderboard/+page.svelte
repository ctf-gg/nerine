<script lang="ts">
  import Countdown from "../../components/Countdown.svelte";
  import type { PageProps } from "./$types";
  import Leaderboard from "./Leaderboard.svelte";

  const { data }: PageProps = $props();
  const { leaderboard, event } = data;
</script>

<div class="content-container">
  {#if leaderboard}
    <Leaderboard
      teams={leaderboard.filter((team) => team.score > 0)}
      yourTeam={data.teamId}
    />
  {:else}
    <Countdown {event} />
  {/if}
</div>
