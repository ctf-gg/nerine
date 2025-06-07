<script lang="ts">
  import type { Challenge } from "../api";
  import ChallengeDisplay from "./Challenge.svelte";

  const { challs }: { challs: Challenge[] } = $props();

  const categories = $state(new Set(challs.map((x) => x.category)));
  const categoryVisibility = $state(
    Object.fromEntries([...categories].map((c) => [c, false]))
  );
  const categoryDetails = $derived.by(() => {
    const details = Object.fromEntries(
      [...categories].map((c) => [c, { solved: 0, total: 0 }])
    );
    for (const chall of challs) {
      details[chall.category].total += 1;
      if (chall.selfSolved) details[chall.category].solved += 1;
    }

    return details;
  });
  const hasCategoryFilter = $derived(
    Object.values(categoryVisibility).some(Boolean)
  );

  let showSolved = $state(false);
  let showOnlyUnsolved = $state(false);

  const fileteredChalls = $derived.by(() => {
    let res = challs;

    if (showOnlyUnsolved) res = res.filter((c) => c.solves === 0);
    if (!showSolved) res = res.filter((c) => !c.selfSolved);

    if (hasCategoryFilter)
      res = res.filter((c) => categoryVisibility[c.category]);

    return res;
  });
</script>

<div class="filters">
  <div class="category-filters">
    {#each categories as category}
      <div class="filter">
        <input
          type="checkbox"
          id="{category}-visibility"
          bind:checked={categoryVisibility[category]}
        />
        <label for="{category}-visibility">
          {category} ({categoryDetails[category].solved} / {categoryDetails[
            category
          ].total} solved)</label
        >
      </div>
    {/each}
  </div>
  <div class="seperator"></div>
  <div class="other-filters">
    <div>
      <input type="checkbox" id="show-solved" bind:checked={showSolved} /><label
        for="show-solved"
        >Show Solved ({challs.reduce((acc, c) => acc + +c.selfSolved, 0)} / {challs.length})</label
      >
    </div>
    <div>
      <input
        type="checkbox"
        id="show-unsolved"
        bind:checked={showOnlyUnsolved}
      /><label for="show-unsolved"
        >Show Only Unsolved ({challs.reduce(
          (acc, c) => acc + (c.solves === 0 ? 1 : 0),
          0
        )})</label
      >
    </div>
  </div>
</div>
<div class="challenges">
  {#each fileteredChalls as chall}
    <ChallengeDisplay {chall} />
  {/each}
</div>

<style>
  .filters {
    border-image: url("/border-big-off-white.png") 12 / 8px round;
    border-image-outset: 2px;
    background-color: #e6e6d2;
    padding: 1rem;
    margin-bottom: 2rem;

    input[type="checkbox"] {
      accent-color: black;
      height: 1.25rem;
      width: 1.25rem;
      margin-right: 0.5rem;
    }

    label {
      font-size: 1.5rem;
    }

    .filter {
      display: flex;
      align-items: center;
    }

    .seperator {
      border-image: url("/border-big-off-white.png") 12 / 8px round;
      border-image-outset: 2px;
      width: 100%;
      height: 4px;
      margin: 0.5rem 0;
    }

    .category-filters,
    .other-filters {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
    }
  }

  .challenges {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
