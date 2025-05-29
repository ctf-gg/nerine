<script lang="ts">
  import { type Challenge } from "../api";
  const { chall: c }: { chall: Challenge } = $props();

  const submit = () => {
    
  }
</script>

<div>
  <h1>{c.name} - {c.points} points / {c.solves} solves</h1>
  <h2>{c.author}</h2>
  <p>{c.description}</p>
  <form id="login">
    <input type="text" name="flag" placeholder="flag" />
    <button type="submit">submit</button>
  </form>
</div>
