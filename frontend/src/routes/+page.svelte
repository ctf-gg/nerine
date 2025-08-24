<script>
  import Countdown from "../components/Countdown.svelte";
  import { event } from "$lib/event";
  import { marked } from "marked";
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
