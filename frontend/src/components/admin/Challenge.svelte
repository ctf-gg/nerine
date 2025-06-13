<script lang="ts">
  import { marked } from "marked";
  import type { AdminChallenge } from "../../admin";
  const { chall: c }: { chall: AdminChallenge } = $props();
</script>

<div class="challenge">
  <div class="header">
    <h1>{c.category}/{c.name}</h1>

    <h2>({c.public_id})</h2>
  </div>
  <div class="subheader">
    <h2 class="author">{c.author}</h2>
  </div>
  <p class="description">{@html marked(c.description)}</p>
  <div class="attachments">
    {#if c.attachments}
      {#each Object.entries(c.attachments) as [name, url]}
        <a href={url} download><button>{name}</button></a>
      {/each}
    {/if}
  </div>
  <div>
    {c.flag}
  </div>
</div>

<style>
  .challenge {
    border-image: url("/border-big-off-white.png") 12 / 8px round;
    border-image-outset: 2px;
    background-color: #e6e6d2;
    padding: 1rem;

    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      h1 {
        font-size: 2rem;
      }
    }

    .subheader {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .author,
    .solve-text {
      margin-top: -0.5rem;
    }

    .description {
      font-family: "Satoshi", sans-serif;
      margin-top: 0.25rem;
      margin-bottom: 1rem;
    }

    .attachments {
      display: flex;
      gap: 0.5rem;
      button {
        font-size: 1rem;
        margin-bottom: 0.5rem;
      }
    }
  }

  .submit {
    width: 100%;
    display: flex;

    input {
      flex-grow: 1;
      transition: background-color 300ms ease-out;

      &.correct {
        background-color: lightgreen;
      }

      &.incorrect {
        background-color: pink;
      }
    }
  }
</style>
