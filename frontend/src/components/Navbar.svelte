<script lang="ts">
  import type { Event, Profile } from "$lib/api";
  import { page } from "$app/state";

  const {
    event,
    profile,
    teamId,
  }: { event: Event; profile: Profile | null; teamId: string  | null } = $props();
</script>

<nav class="nav-container">
  <ul>
    <li class="event-name"><a href="/">{event.name}</a></li>
  </ul>
  <ul>
    {@render route("Home", "/")}
    {@render route("Challenges", "/challenges")}
    {@render route("Leaderboard", "/leaderboard")}
  </ul>
  <ul class="auth-items">
    {#if profile && teamId}
      <!-- <li>{team.name}</li> TODO(aiden): dunno whether i should leave this here or not -->
      {@render route("Profile", `/profile/${teamId}`)}
      {@render route("Logout", "/logout")}
    {:else}
      {@render route("Login", `/login`)}
      {@render route("Register", "/register")}
    {/if}
  </ul>
</nav>

{#snippet route(name: string, path: string)}
  <li>
    <a href={path} class={[page.url.pathname == path && "selected"]}>{name}</a>
  </li>
{/snippet}

<style>
  a {
    text-decoration: none;
    color: inherit;

    display: block;
    padding: 0.75rem 0.5rem;

    &:hover {
      background: color-mix(in oklch, var(--text-primary) 10%, transparent);
    }

    &.selected {
      background: var(--bg-accent);
      box-shadow: inset 0 4px 0 0 var(--text-accent);
    }
  }

  nav {
    position: sticky;
    top: 0;
    z-index: 100;
    border-bottom: 1px solid var(--text-primary);
    backdrop-filter: blur(0.25rem);
    padding: 0 0.75rem;
    display: grid;
    align-items: center;
    grid-template-columns: 1fr auto 1fr;

    ul {
      list-style: none;
      padding: 0;
      margin: 0;
      display: flex;
    }

    .event-name {
      font-weight: 700;
    }

    .auth-items {
      margin-left: auto;
    }
  }
</style>
