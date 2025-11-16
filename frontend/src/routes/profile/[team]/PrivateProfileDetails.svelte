<script lang="ts">
  import type { ApiError, Team, Event, PrivateProfile } from "$lib/api";
  import { invalidateAll } from '$app/navigation';
  import { isError, updateProfile } from "$lib/api";
  import TokenReveal from "./TokenReveal.svelte";

  const { event, profile }: { event: Event; profile: PrivateProfile } = $props();

  let name = $state(profile.name);
  let email = $state(profile.email);
  let division = $state(profile.division);

  let status: { message: string } | ApiError | null = $state(null);

  const submit = async (e: SubmitEvent) => {
    e.preventDefault();
    status = null;

    const res = await updateProfile(email, name, division);
    if (isError(res)) {
      status = res;
    } else if ("message" in res && typeof res.message === "string") {
      status = res; // bad programming :>
      invalidateAll();
    } else {
      invalidateAll();
    }
  };
</script>

<div class="priv-details">
  <div class="priv-warning">ONLY YOUR TEAM CAN SEE THIS</div>
  <div class="reveal">
    <h2>Invite URL</h2>
    <TokenReveal />
    Send this link to your teammates to invite them to your team!
  </div>
  <div>
    <h2>Update Team Details:</h2>
    <form id="update" onsubmit={submit}>
      <label for="update-email">Email:</label>
      <input
        type="text"
        id="update-email"
        name="email"
        placeholder="Email"
        bind:value={email}
      />

      <label for="update-name">Team Name:</label>
      <input
        type="text"
        id="update-name"
        name="name"
        placeholder="Team Name"
        bind:value={name}
      />

      <label for="update-division">Division:</label>
      <select id="update-division" name="division" bind:value={division}>
	<option value={null}>No Division</option>
	{#each Object.entries(event.divisions) as [id, name]}
	  <option value={id}>{name}</option>
	{/each}
      </select>

      <button type="submit">Update</button>
    </form>
    {#if status}
      <div class="{isError(status) ? 'error' : 'success'}-text">
        {status.message}
      </div>
    {/if}
  </div>
</div>

<style>
  .priv-details {
    margin-top: 1rem;
    padding: 1rem;
    border: 1px solid var(--text-accent);
    background: var(--bg-accent);
    position: relative;
    .priv-warning {
      position: absolute;
      font-weight: 600;
      font-size: 0.75rem;
      color: var(--text-accent);
      right: 0.5rem;
      top: 0.25rem;
    }

    h2 {
      text-align: center;
      margin-bottom: 0.5rem;
    }

    .reveal {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      overflow: hidden;
      padding: 0.25rem 0.25rem 0.25rem 0;
      margin-bottom: 0.5rem;
    }
  }

  #update {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    input, select {
      margin-bottom: 1rem;
    }
  }
</style>
