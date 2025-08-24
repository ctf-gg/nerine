<script lang="ts">
  import type { LeaderboardEntry } from "$lib/api";

  const {
    teams,
    yourTeam,
  }: { teams: LeaderboardEntry[]; yourTeam: string | null } = $props();
</script>

<div class="leaderboard-container">
  <table class="leaderboard">
    <thead>
      <tr class="headers">
        <th>Rank</th>
        <th>Name</th>
        <th>Points</th>
      </tr>
    </thead>
    <tbody>
      {#each teams as t, i}
        <tr class={[yourTeam === t.id && "your-team"]}>
          <td>
            <a href={"/profile/" + t.id}>{i + 1}</a>
          </td>
          <td>
            <a href={"/profile/" + t.id}>
              {t.name}
            </a>
          </td>
          <td>
            <a href={"/profile/" + t.id}>{t.score}</a>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .leaderboard-container {
    border: 1px solid var(--text-primary);
    padding: 1rem;
  }
  .leaderboard {
    width: 100%;
    text-align: center;
    border-spacing: 0;

    tbody tr {
      transition: all 300ms;
      &:hover {
        background: var(--bg-neutral);
      }
    }

    td {
      border: none;
      border-bottom: 1px solid var(--text-primary);
      padding: 0;
      a {
        display: block;
        padding: 0.15rem 0;
        text-decoration: none;
      }
    }

    .your-team {
      background: var(--bg-accent);
      color: var(--text-accent);
      font-weight: 600;
    }

    th {
      padding: 0;
    }

    .headers {
      z-index: 10;
      position: sticky;
      top: 3rem;
      background: var(--bg-primary);
      th {
        border-bottom: 1px solid var(--text-primary);
        padding: 0.5rem;
      }
    }
  }

  @media (max-width: 768px) {
    .headers th {
      padding: 0 0.25rem;
      font-size: 0.9rem;
    }
  }
</style>
