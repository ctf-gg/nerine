<script lang="ts">
  import favicon from "$lib/assets/favicon.svg";
  import type { LayoutProps } from "./$types";

  import "../base.css";
  import Navbar from "../components/Navbar.svelte";
  import { browser } from "$app/environment";

  let { children, data }: LayoutProps = $props();
  const { authedProfile, teamId, event } = data;
  // TODO(aiden): very bad solution, please fix!
  let theme: "light" | "dark" = $state(browser ? localStorage["nerine-theme"] ?? "light" : "light");

  $effect(() => {
    if (!localStorage["nerine-theme"])
      theme = window?.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";

    localStorage["nerine-theme"] = theme;
  });
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<div class="theme-{theme}">
  <Navbar
    {event}
    {teamId}
    profile={authedProfile}
    bind:theme
  />
  <div>
    {@render children?.()}
  </div>
</div>
