<script lang="ts">
  import type { Badge } from "../api";
  import { badges } from "../badges";

  const { badge }: { badge: Badge | undefined } = $props();

  const info = badge ? badges[badge.type] : null;

  // svelte-ignore non_reactive_update
  let dialog: HTMLDialogElement;
  const openDialog = (e: MouseEvent) => {
    dialog.showModal();
  };

  const closeDialog = () => {
    dialog.close();
  };
</script>

{#if badge && info}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <img
    class="icon"
    src="/badges/{info.icon}"
    alt={info.name}
    onclick={openDialog}
  />

  <dialog class="popup" bind:this={dialog} onclick={closeDialog}>
    <div class="badge-container">
      <div>
        <img src="/badges/{info.icon}" alt={info.name} class="icon-big" />
        <h1>{info.name}</h1>
        <h2>{info.description}</h2>
        <p>Obtained {new Date(badge.obtained).toLocaleString()} by first blooding {badge.type}/{badge.chall}</p>
      </div>
    </div>
  </dialog>
{/if}

<style>
  .icon {
    image-rendering: pixelated;
    transform: scale(1.25);
    transition: 300ms;
    cursor: pointer;
  }

  .icon:hover {
    transform: scale(1.75);
  }

  .popup {
    max-height: 100vh;
    max-width: 100vw;
    background: #000000af;
    height: 100%;
    width: 100%;
    cursor: pointer;
    border: none;
    transition: 300ms;

    .badge-container {
      display: grid;
      align-items: center;
      justify-content: center;
      height: 100%;
      width: 100%;
      color: white;

      .icon-big {
        height: 25rem;
        width: 25rem;
        image-rendering: pixelated;
        margin: auto;
      }

      h1 {
        font-size: 6rem;
      }

      h2 {
        font-size: 3rem;
        font-style: italic;
        margin-top: -2rem;
        margin-bottom: 2rem;
      }

      p {
        font-style: italic;
        font-size: 1.5rem;
      }
    }
  }
</style>
