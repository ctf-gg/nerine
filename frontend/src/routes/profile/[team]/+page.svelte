<script lang="ts">
  import type { PageProps } from "./$types";
  import PrivateProfileDetails from "./PrivateProfileDetails.svelte";

  const { data }: PageProps = $props();
  const event = data.event;
  let profile = $derived(data.profile);

  function ordinal(i: number) {
    let j = i % 10;
    let k = i % 100;
    if (j === 1 && k !== 11) {
      return i + "st";
    }
    if (j === 2 && k !== 12) {
      return i + "nd";
    }
    if (j === 3 && k !== 13) {
      return i + "rd";
    }
    return i + "th";
  }
</script>

<div class="content-container">
  <div class="basic-details">
    <h1>{profile.name}</h1>

    {#if profile.rank !== -1}
      <div class="stats">
	{#if profile.division}
	  {event.divisions[profile.division]} Division  <br/>
	{/if}
	{ordinal(profile.rank)} place / {profile.score} points
      </div>
    {/if}
  </div>

  {#if profile.type === "private"}
    <PrivateProfileDetails {profile} {event} />
  {/if}

  <div class="solves">
    <h2>Solves:</h2>
    <table>
      <thead>
        <tr>
          <th>Challenge</th>
          <th>Points</th>
          <th>Solved At</th>
        </tr>
      </thead>
      <tbody>
        {#each profile.solves as solve}
          <tr>
            <td>{solve.category}/{solve.name}</td>
            <td>{solve.points} pts</td>
            <td>
              {new Date(solve.solved_at + "Z").toLocaleString()}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .basic-details {
    text-align: center;

    .stats {
      font-size: 1.5rem;
    }
  }

  .solves {
    margin-top: 1rem;
    border: 1px solid var(--text-neutral);
    padding: 1rem;
    text-align: center;

    h2 {
      margin-bottom: 0.5rem;
    }

    table {
      border-collapse: collapse;
    }

    tbody tr {
      transition: all 300ms;
      /* &:hover {
        background: var(--bg-neutral);
      } */
    }

    td {
      border: none;
      border-top: 1px solid var(--text-neutral);
      border-bottom: 1px solid var(--text-neutral);
      padding: 0;
      a {
        display: block;
        padding: 0.15rem 0;
        text-decoration: none;
      }
    }
  }
</style>
