<script lang="ts">
  import { type Challenge } from "../api";
  const { chall: c }: { chall: Challenge } = $props();

  const submit = () => {};
</script>

<div class="challenge">
  <div class="header">
    <h1>{c.category}/{c.name}</h1>
    <h2>{c.solves} solves / {c.points} points</h2>
  </div>
  <h2 class="author">{c.author}</h2>
  <p class="description">{c.description}</p>
  <form class="submit">
    <input type="text" name="flag" placeholder="flag" />
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
      background-color: #f1f1df;
    }
  }
</style>
