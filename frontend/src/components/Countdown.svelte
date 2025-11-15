<script lang="ts">
  import type { Event } from "$lib/api";
  import { onMount, onDestroy } from "svelte";
  const {
    showHeading = true,
    autoReload = false,
    event,
  }: { showHeading?: boolean; autoReload?: boolean; event: Event } = $props();

  type Counter = {
    days: number;
    hours: number;
    minutes: number;
    seconds: number;
  };

  let timeLeft = $state<Counter | null>(null);
  let isCountingToEnd = $state(false);
  let eventHasEnded = $state(false);

  let intervalId: number | undefined;

  function updateCountdown() {
    const now = new Date().getTime();
    const startTime = event.start_time.getTime();
    const endTime = event.end_time.getTime();

    // if we were counting to start, and the event started, and we're auto reloading, reload the page.
    if (autoReload && !isCountingToEnd && !eventHasEnded && now > startTime) {
      if (intervalId) {
        clearInterval(intervalId);
      }

      setTimeout(() => {
        document.location.reload();
      }, 1000);
    }

    if (now < startTime) {
      isCountingToEnd = false;
      eventHasEnded = false;
      const distance = startTime - now;
      timeLeft = {
        days: Math.floor(distance / (1000 * 60 * 60 * 24)),
        hours: Math.floor(
          (distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)
        ),
        minutes: Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60)),
        seconds: Math.floor((distance % (1000 * 60)) / 1000),
      };
    } else if (now < endTime) {
      isCountingToEnd = true;
      eventHasEnded = false;
      const distance = endTime - now;
      timeLeft = {
        days: Math.floor(distance / (1000 * 60 * 60 * 24)),
        hours: Math.floor(
          (distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)
        ),
        minutes: Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60)),
        seconds: Math.floor((distance % (1000 * 60)) / 1000),
      };
    } else {
      isCountingToEnd = false;
      eventHasEnded = true;
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
    <h2 class="countdown-heading">
      {#if eventHasEnded}
        {event.name} has ended
      {:else if isCountingToEnd}
        {event.name} ends in:
      {:else}
        {event.name} starts in:
      {/if}
    </h2>
  {/if}
  <div class="countdown-display">
    <div class="time-unit">
      <span class="number"
        >{timeLeft ? timeLeft.days.toString().padStart(2, "0") : "--"}</span
      >
      <span class="label">Days</span>
    </div>
    <div class="separator">:</div>
    <div class="time-unit">
      <span class="number"
        >{timeLeft ? timeLeft.hours.toString().padStart(2, "0") : "--"}</span
      >
      <span class="label">Hours</span>
    </div>
    <div class="separator">:</div>
    <div class="time-unit">
      <span class="number"
        >{timeLeft ? timeLeft.minutes.toString().padStart(2, "0") : "--"}</span
      >
      <span class="label">Minutes</span>
    </div>
    <div class="separator">:</div>
    <div class="time-unit">
      <span class="number"
        >{timeLeft ? timeLeft.seconds.toString().padStart(2, "0") : "--"}</span
      >
      <span class="label">Seconds</span>
    </div>
  </div>
</div>

<style>
  .countdown-container {
    display: grid;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--text-neutral);
    padding: 1rem;
  }

  .countdown-heading {
    text-align: center;
    font-size: 2.25rem;
  }

  .countdown-display {
    display: flex;
    align-items: center;
  }

  .time-unit {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0 1rem;
  }

  .number {
    font-size: 3.5rem;
    line-height: 1;
  }

  .label {
    font-size: 1.25rem;
  }

  .separator {
    font-size: 3.5rem;
    line-height: 1;
    padding-bottom: 2.5rem;
  }

  @media (max-width: 768px) {
    .countdown-heading {
      font-size: 2rem;
    }

    .countdown-display {
      gap: 0.5rem;
    }
    
    .time-unit {
      padding: 0.5rem;
    }
    
    .number {
      font-size: 2.5rem;
    }
    
    .label {
      font-size: 0.875rem;
    }
    
    .separator {
      font-size: 2.5rem;
      padding-bottom: 1.5rem;
    }
  }
</style>
