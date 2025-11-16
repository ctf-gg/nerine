<script lang="ts">
  import Countdown from "../../components/Countdown.svelte";
  import type { PageProps } from "./$types";
  import Leaderboard from "./Leaderboard.svelte";
  import { browser } from "$app/environment";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";

  const { data }: PageProps = $props();
  let leaderboard = $derived(data.leaderboard);
  let division = $state(data.division);
  const event = $derived(data.event);
  const changeDivision = (newDivision: string | null) => {
    if (browser) {
      const current = $page.url.searchParams.get('division')
      const url = new URL($page.url)
      if (newDivision) url.searchParams.set('division', newDivision);
      else url.searchParams.delete('division');
      goto(url.toString(), { replaceState: true, noScroll: true, invalidateAll: true });
    }
    division = newDivision;
  }
</script>

<div class="content-container">
  {#if leaderboard}
    {#if Object.keys(event.divisions).length > 0}
      <div class="division-selector">
	<button class={{ selected: division == null }} onclick={() => changeDivision(null)}>Open</button>
	{#each Object.entries(event.divisions) as [id, name]}
	  <button class={{ selected: division == id }} onclick={() => changeDivision(id)}>{name}</button>
	{/each}
      </div>
    {/if}
    <Leaderboard
      teams={leaderboard.filter((team) => team.score > 0)}
      yourTeam={data.teamId}
    />
  {:else}
    <Countdown {event} />
  {/if}
</div>

<style>
  .division-selector {
    display: flex;
    border: 1px solid var(--text-neutral);
    margin-bottom: 1rem;
    button {
      flex-grow: 1;
      background: transparent;
      color: var(--text-primary);
      &:not(:last-child) {
	border-right: 1px solid var(--text-neutral);
      }
      &.selected {
	background: var(--text-primary);
	color: var(--bg-primary);
      }
    }
  }
</style>
