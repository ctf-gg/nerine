<script lang="ts">
  import type { Event, Profile } from "$lib/api";
  import { page } from "$app/state";

  let {
    event,
    profile,
    teamId,
    theme = $bindable(),
  }: {
    event: Event;
    profile: Profile | null;
    teamId: string | null;
    theme: "light" | "dark";
  } = $props();
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
    <li>
      <button class="ghost theme-toggle" onclick={() => {
        if (theme === "light") theme = "dark";
        else theme = "light";
      }}>
        {#if theme === "light"}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="2"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M21.752 15.002A9.72 9.72 0 0 1 18 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 0 0 3 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 0 0 9.002-5.998Z"
            />
          </svg>
        {:else}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="2"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 3v2.25m6.364.386-1.591 1.591M21 12h-2.25m-.386 6.364-1.591-1.591M12 18.75V21m-4.773-4.227-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0Z"
            />
          </svg>
        {/if}
      </button>
    </li>
    {#if profile && teamId}
      <!-- <li>{team.name}</li> TODO(aiden): dunno whether i should leave this here or not -->
      {@render route("Profile", `/profile/${teamId}`)}
    <li>
      <form action="/logout" method="POST">
	<button type="submit" class="ghost">Logout</button>
      </form>
    </li>
    {:else}
      {@render route("Login", `/login`)}
      {@render route("Register", "/register")}
    {/if}
  </ul>
</nav>

{#snippet route(name: string, path: string)}
  <li>
    <a
      href={path}
      class={[page.url.pathname == path && "selected"]}
    >
      {name}
    </a>
  </li>
{/snippet}

<style>
  a,
  button.ghost {
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

  .theme-toggle {
    height: 100%;
    svg {
      height: 1rem;
      width: 1rem;
    }
  }

  nav {
    position: sticky;
    top: 0;
    z-index: 100;
    border-bottom: 1px solid var(--text-neutral);
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
