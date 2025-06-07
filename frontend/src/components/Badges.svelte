<script lang="ts">
  import type { Badge } from "../api";
  import BadgeDisplay from "./Badge.svelte";

  const { badges }: { badges: Badge[] | null } = $props();
</script>

<div class="badges">
  <div class="row">
    <div class="badge">
      <BadgeDisplay badge={badges?.[0]} />
    </div>
    <div class="badge">
      <BadgeDisplay badge={badges?.[2]} />
    </div>
    <div class="badge">
      <BadgeDisplay badge={badges?.[4]} />
    </div>
    <div class="badge">
      <BadgeDisplay badge={badges?.[6]} />
    </div>
  </div>
  <div class="row">
    <div class="badge">
      <BadgeDisplay badge={badges?.[1]} />
    </div>
    <div class="badge">
      <BadgeDisplay badge={badges?.[3]} />
    </div>
    <div class="badge">
      <BadgeDisplay badge={badges?.[5]} />
    </div>
  </div>
</div>

<style>
  .badges {
    display: grid;
    padding: 0 0.5rem;
    grid-template-rows: 1fr 1fr;

    .row {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 0.25rem;
    }

    .badge {
      width: 1.5rem;
      height: 1.5rem;
      background: #dedec4;
      border-radius: 100%;
    }
  }
</style>
