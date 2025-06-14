<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  const { showHeading = true, isTitle = false, event } = $props();

  type Counter = {
    days: number;
    hours: number;
    minutes: number;
    seconds: number;
  };

  let timeLeft = $state<Counter | null>(null);
  let isCountingToEnd = $state(false);
  let eventHasEnded = $state(false);

  let intervalId: NodeJS.Timeout | undefined;

  function updateCountdown() {
    const now = new Date().getTime();
    const startTime = event.start_time.getTime();
    const endTime = event.end_time.getTime();

    if (now < startTime) {
      // Event hasn't started yet, countdown to start
      isCountingToEnd = false;
      eventHasEnded = false;
      const distance = startTime - now;
      timeLeft = {
        days: Math.floor(distance / (1000 * 60 * 60 * 24)),
        hours: Math.floor(
          (distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60),
        ),
        minutes: Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60)),
        seconds: Math.floor((distance % (1000 * 60)) / 1000),
      };
    } else if (now < endTime) {
      // Event has started but hasn't ended, countdown to end
      isCountingToEnd = true;
      eventHasEnded = false;
      const distance = endTime - now;
      timeLeft = {
        days: Math.floor(distance / (1000 * 60 * 60 * 24)),
        hours: Math.floor(
          (distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60),
        ),
        minutes: Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60)),
        seconds: Math.floor((distance % (1000 * 60)) / 1000),
      };
    } else {
      // Event has ended
      isCountingToEnd = false;
      eventHasEnded = true;
      timeLeft = { days: 0, hours: 0, minutes: 0, seconds: 0 };

      if (document.location.pathname !== "/") {
        if (intervalId) {
          clearInterval(intervalId);
        }

        setTimeout(() => {
          document.location.reload();
        }, 1000);
      }
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
    {#if isTitle}
      <h2 class="countdown-title">
        {#if eventHasEnded}
          has ended
        {:else if isCountingToEnd}
          ends in
        {:else}
          starts in
        {/if}
      </h2>
    {:else}
      <h2 class="countdown-heading">
        {#if eventHasEnded}
          smileyCTF has ended
        {:else if isCountingToEnd}
          smileyCTF ends in:
        {:else}
          smileyCTF starts in:
        {/if}
      </h2>
    {/if}
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
  }

  .countdown-heading {
    text-align: center;
    font-size: 3rem;
  }

  .countdown-title {
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

  @media (max-width: 440px) {
    .countdown-heading {
      font-size: 2rem;
    }
    .countdown-title {
      font-size: 2.5rem;
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
