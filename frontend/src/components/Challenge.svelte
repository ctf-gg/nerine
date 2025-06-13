<script lang="ts">
  import { type Challenge, isError, submitFlag } from "../api";
  const { chall: c }: { chall: Challenge } = $props();

  let flagInput: HTMLInputElement;
  let correct = $state<boolean | null>(null);
  async function submit(e: SubmitEvent) {
    e.preventDefault();
    const j = await submitFlag(c.id, flagInput.value);
    flagInput.value = "";
    if (isError(j)) {
      alert("uwuups! " + j.message);
    }
    correct = !isError(j);
  }
</script>

<div class="challenge">
  <div class="header">
    <h1>{c.category}/{c.name}</h1>

    <h2>{c.solves} solves / {c.points} points</h2>
  </div>
  <div class="subheader">
    <h2 class="author">{c.author}</h2>
    {#if c.selfSolved}
      <h2 class="solve-text">Solved by you!</h2>
    {/if}
  </div>
  <p class="description">{c.description}</p>
  <div class="attachments">
    {#if c.attachments}
      {#each Object.entries(c.attachments) as [name, url]}
        <a href={url}><button>{name}</button></a>
      {/each}
    {/if}
  </div>
  <form class="submit" onsubmit={submit}>
    <input
      type="text"
      name="flag"
      placeholder="flag"
      autocomplete="off"
      onchange={() => (correct = null)}
      bind:this={flagInput}
      class={[{ correct: correct === true, incorrect: correct === false }]}
    />
    <button type="submit">submit</button>
  </form>
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
      margin-bottom: 1.25rem;
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
