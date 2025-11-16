<script lang="ts">
  import { onMount } from "svelte";
  import type { Challenge, Event } from "$lib/api";
  import ChallengeDisplay from "./Challenge.svelte";

  const { challs, event, yourTeam }: { challs: Challenge[]; event: Event; yourTeam: string | null } = $props();

  const categories = $state(new Set(challs.map((x) => x.category)));
  let filters = $state({
    category: Object.fromEntries([...categories].map((c) => [c, false])),
    solved: false,
    onlyUnsolved: false,
  })

  const categoryDetails = $derived.by(() => {
    const details = Object.fromEntries(
      [...categories].map((c) => [c, { solved: 0, total: 0 }])
    );
    for (const chall of challs) {
      details[chall.category].total += 1;
      if (chall.solved_at) details[chall.category].solved += 1;
    }

    return details;
  });

  const hasCategoryFilter = $derived(
    Object.values(filters.category).some(Boolean)
  );

  const unblooded = $derived(
    challs.reduce((acc, c) => acc + (c.solves === 0 ? 1 : 0), 0)
  );

  const fileteredChalls = $derived.by(() => {
    let res = challs;

    if (filters.onlyUnsolved) res = res.filter((c) => c.solves === 0);
    if (!filters.solved) res = res.filter((c) => !c.solved_at);

    if (hasCategoryFilter)
      res = res.filter((c) => filters.category[c.category]);

    return res;
  });

  onMount(() => {
    const savedFilters = JSON.parse(localStorage.getItem("nerine-challenge-filters"));
    if (!savedFilters) return;

    if (savedFilters.category)
      for (const [category, on] of Object.entries(savedFilters.category))
	if (typeof filters.category[category] === "boolean") filters.category[category] = on

    filters.onlyUnsolved = !!savedFilters.onlyUnsolved;
    filters.solved = !!savedFilters.solved;
  })

  $effect(() => {
    localStorage.setItem("nerine-challenge-filters", JSON.stringify(filters));
  })
</script>

<div class="filters">
  <div class="category-filters">
    {#each categories as category}
      <div class="filter">
        <input
          type="checkbox"
          id="{category}-visibility"
          bind:checked={filters.category[category]}
        />
        <label for="{category}-visibility">
          {category} ({categoryDetails[category].solved} / {categoryDetails[
            category
          ].total} solved)</label
        >
      </div>
    {/each}
  </div>
  <hr />
  <div class="other-filters">
    <div>
      <input type="checkbox" id="show-solved" bind:checked={filters.solved} /><label
        for="show-solved"
        >Show Solved ({challs.reduce((acc, c) => acc + (c.solved_at ? 1 : 0), 0)}
        / {challs.length})</label
      >
    </div>
    {#if unblooded}
      <div>
        <input
          type="checkbox"
          id="show-unsolved"
          bind:checked={filters.onlyUnsolved}
        /><label for="show-unsolved">Show Only Unblooded ({unblooded})</label>
      </div>
    {/if}
  </div>
</div>
<div class="challenges">
  {#each fileteredChalls as chall (chall.id)}
    <ChallengeDisplay {chall} {event} {yourTeam} />
  {/each}
</div>

<style>
  .filters {
    border: 1px solid var(--text-neutral);
    padding: 1rem;
    margin-bottom: 2rem;

    input[type="checkbox"] {
      accent-color: var(--text-accent);
      height: 1.25rem;
      width: 1.25rem;
      margin-right: 0.5rem;
    }

    label {
      white-space: nowrap;
    }

    .filter {
      display: flex;
      align-items: center;
    }

    hr {
      margin: 0.5rem 0;
      height: 0;
      width: 100%;
      border: none;
      border-bottom: 1px solid color-mix(in srgb, var(--text-neutral) 50%, transparent);
      margin: 0.5rem 0;
    }

    .category-filters,
    .other-filters {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 0.5rem;
      div {
        display: flex;
        align-items: center;
      }
    }

    @media (max-width: 768px) {
      .category-filters,
      .other-filters {
        grid-template-columns: 1fr;
      }
    }
  }

  .challenges {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
