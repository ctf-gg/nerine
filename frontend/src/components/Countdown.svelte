<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EVENT_START } from "../event";

  const { showHeading = true } = $props();

  type Counter = {
    days: number;
    hours: number;
    minutes: number;
    seconds: number;
  };

  let timeLeft = $state<Counter | null>(null);

  let intervalId: NodeJS.Timeout | undefined;

  function updateCountdown() {
    const now = new Date().getTime();
    const distance = EVENT_START.getTime() - now;

    if (distance > 0) {
      timeLeft = {
        days: Math.floor(distance / (1000 * 60 * 60 * 24)),
        hours: Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)),
        minutes: Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60)),
        seconds: Math.floor((distance % (1000 * 60)) / 1000),
      };
    } else {
      if (document.location.pathname !== "/") {
        if (intervalId) {
          clearInterval(intervalId);
        }

        setTimeout(() => {
          document.location.reload();
        }, 1000);
      }
      timeLeft = { days: 0, hours: 0, minutes: 0, seconds: 0 };
    }
  }

  onMount(() => {
    updateCountdown();
    intervalId = setInterval(updateCountdown, 1000);
  });

  onDestroy(() => {
    if (intervalId) {
      clearInterval(intervalId);
    }
  });
</script>

<div class="countdown-container">
  {#if showHeading}
    <h2 class="countdown-heading">smileyCTF starts in:</h2>
  {/if}
  <div class="countdown-display">
    <div class="time-unit">
      <span class="number">{timeLeft ? timeLeft.days.toString().padStart(2, "0") : "--"}</span>
      <span class="label">Days</span>
    </div>
    <div class="separator">:</div>
    <div class="time-unit">
      <span class="number">{timeLeft ? timeLeft.hours.toString().padStart(2, "0") : "--"}</span>
      <span class="label">Hours</span>
    </div>
    <div class="separator">:</div>
    <div class="time-unit">
      <span class="number">{timeLeft ? timeLeft.minutes.toString().padStart(2, "0") : "--"}</span>
      <span class="label">Minutes</span>
    </div>
    <div class="separator">:</div>
    <div class="time-unit">
      <span class="number">{timeLeft ? timeLeft.seconds.toString().padStart(2, "0") : "--"}</span>
      <span class="label">Seconds</span>
    </div>
  </div>
</div>

<style>
  .countdown-container {
    display: grid;
    align-items: center;
    justify-content: center;
    font-family: "Smiley Sans", sans-serif;
    padding: 1rem 0;
  }

  .countdown-heading {
    text-align: center;
    font-size: 3rem;
  }

  .countdown-display {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .time-unit {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 1rem;
  }

  .number {
    font-size: 4rem;
    line-height: 1;
  }

  .label {
    font-size: 1.25rem;
  }

  .separator {
    font-size: 4rem;
    padding-bottom: 2.5rem;
  }
</style>
