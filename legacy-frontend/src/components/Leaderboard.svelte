<script lang="ts">
  import type { LeaderboardEntry } from "../api";
  import Badges from "./Badges.svelte";

  const { teams, yourTeam }: { teams: LeaderboardEntry[]; yourTeam: string | null } =
    $props();
</script>

<table class="leaderboard">
  <tbody>
    <tr class="headers">
      <th>Rank</th>
      <th>Name</th>
      <th>Points</th>
      <th>Badges</th>
    </tr>
    {#each teams as t, i}
      <tr class="entry">
        <td>
          <a href={"/profile/" + t.id}>{i + 1}</a>
        </td>
        <td>
          <a href={"/profile/" + t.id}>
            {t.name}
            {#if yourTeam === t.id}
              <span class="you-indicator">(You!)</span>
            {/if}
          </a>
        </td>
        <td>
          <a href={"/profile/" + t.id}>{t.score}</a>
        </td>
        <td>
          <Badges badges={t?.extra?.badges} />
          {#if yourTeam === t.id}
            <aside class="you-are-here">
              <div>
                <h2>You are here!</h2>
                <img src="/here.png" alt="arrow pointing to team" />
              </div>
            </aside>
          {/if}
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  .you-are-here {
    margin-bottom: 0.5rem;
    text-align: center;
    position: absolute;

    div {
      h2 {
        border-image: url("/border-big-off-white.png") 12 / 8px round;
        border-image-outset: 2px;
        font-size: 1.5rem;
        background-color: #e6e6d2;
        padding: 0.25rem 0.5rem;
      }

      img {
        margin-left: 3rem;
      }

      position: relative;
      right: -12rem;
      top: -5rem;
      z-index: 1000;
      rotate: 16deg;
    }
  }

  .leaderboard {
    width: 100%;
    font-size: 1.5rem;
    text-align: center;
    border-spacing: 0;

    th {
      padding: 0;
    }

    .entry {
      td {
        border-bottom: 1px solid #e6e6d2;
        a {
          color: inherit;
          width: 100%;
          display: block;
          padding: 0.75rem 0;
          text-decoration: none;
        }
      }
    }

    .headers {
      z-index: 10;
      position: sticky;
      top: 1px;
      th {
        padding: 0 0.5rem;
        border-image: url("/border-big-off-white.png") 12 / 8px round;
        border-image-outset: 2px;
        background-color: #e6e6d2;
      }
    }
  }

  .leaderboard :nth-child(2) {
    td {
      padding: 2rem 0;
      background: rgba(255, 255, 0, 0.1);
    }
  }

  .leaderboard :nth-child(3) {
    td {
      padding: 1.5rem 0;
      background: rgb(192, 192, 192, 0.1);
    }
  }

  .leaderboard :nth-child(4) {
    td {
      padding: 1.25rem 0;
      background: rgb(205, 127, 50, 0.1);
    }
  }

  .you-indicator {
    display: none;
    color: #f1c40f;
    font-weight: bold;
  }

  @media (max-width: 1270px) {
    .you-are-here {
      display: none;
    }

    .you-indicator {
      display: inline;
    }

    .entry:has(.you-are-here) td {
      padding: 1rem 0;
      font-size: 1.1em;
    }
  }

  @media (max-width: 768px) {
    .leaderboard {
      font-size: 1rem;
    }

    .entry td a {
      padding: 0.5rem 0.25rem;
    }

    .headers th {
      padding: 0 0.25rem;
      font-size: 0.9rem;
    }

    .leaderboard :nth-child(2) td {
      padding: 1rem 0;
    }

    .leaderboard :nth-child(3) td {
      padding: 0.75rem 0;
    }

    .leaderboard :nth-child(4) td {
      padding: 0.625rem 0;
    }

    .entry:has(.you-are-here) td {
      padding: 0.75rem 0;
      font-size: 1em;
    }
  }
</style>
