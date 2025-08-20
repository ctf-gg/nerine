<script lang="ts">
  import favicon from "$lib/assets/favicon.svg";
  import type { LayoutProps } from "./$types";

  import "../base.css";
  import Navbar from "../components/Navbar.svelte";
  import { event } from "$lib/event";

  let { children, data }: LayoutProps = $props();
  const { profile, teamId } = data;
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<div class="theme-light">
  <Navbar {event} {profile} {teamId} />
  <div>
    {@render children?.()}
  </div>
</div>
