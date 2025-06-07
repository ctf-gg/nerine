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
    {#if c.selfSolved}
      <h2>Solved by you!</h2>
    {/if}
    <h2>{c.solves} solves / {c.points} points</h2>
  </div>
  <h2 class="author">{c.author}</h2>
  <p class="description">{c.description}</p>
  <form class="submit" onsubmit={submit}>
    <input type="text" name="flag" placeholder="flag"
      onchange={() => correct = null}
      bind:this={flagInput}
      class={[{ correct: correct === true, incorrect: correct === false }]} />
    <button type="submit">submit</button>
  </form>
</div>

<style>
  .challenge {
    border-image: url("/border-big-off-white.png") 12 / 8px round;
    border-image-outset: 2px;
    background-color: #f1f1df;
    padding: 1rem;

    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-family: "Smiley Sans";
      h1 {
        font-size: 2rem;
      }
    }
    .author {
      font-family: "Smiley Sans";
      margin-top: -0.5rem;
    }

    .description {
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
