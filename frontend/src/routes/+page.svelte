<script>
  import Countdown from "../components/Countdown.svelte";
  import { marked } from "marked";
  const { data } = $props();
  const { event } = data;
</script>

<div class="content-container">
  <Countdown {event} />
  <div class="description">
    {@html marked(event.description)}
  </div>
</div>

<style>
  .description {
    margin-top: 1rem;
  }
</style>
