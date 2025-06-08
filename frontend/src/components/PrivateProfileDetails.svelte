<script lang="ts">
  import { isError, updateProfile, type Team } from "../api";
  import TokenReveal from "./TokenReveal.svelte";

  const { prof } = $props();

  let name = $state(prof.name);
  let email = $state(prof.email);

  const submit = async (e: SubmitEvent) => {
    e.preventDefault();

    const res = await updateProfile(email, name);
    if (isError(res)) {
      return alert(`wuh oh! ${res.message}`); // tell user they messed up
    } else if ("message" in res && typeof res.message === "string") {
      alert(res.message);
      prof.name = res.name;
      window.location.reload();
    } else {
      const updatedTeam = res as Team;
      prof.name = updatedTeam.name;
      prof.email = updatedTeam.email;
      window.location.reload();
    }
  };
</script>

<div class="priv-details">
  <aside>
    <h2 class="header">Only your team can see this!</h2>
  </aside>
  <div class="detail-contents">
    <div class="reveal">
      <h2>Token:</h2>
      <TokenReveal />
    </div>
    <div>
      <h2>Update Team Details:</h2>
      <form id="update" onsubmit={submit}>
        <label for="update-email">Email:</label>
        <input
          type="text"
          id="update-email"
          name="email"
          placeholder="email"
          bind:value={email}
        />

        <label for="update-name">Team Name:</label>
        <input
          type="text"
          id="update-name"
          name="name"
          placeholder="team name"
          bind:value={name}
        />

        <button type="submit">update</button>
      </form>
    </div>
  </div>
</div>

<style>
  .priv-details {
    margin-top: 1rem;
    border-image: url("/border-big-off-white.png") 12 / 8px round;
    border-image-outset: 2px;
    background-color: #e6e6d2;
    padding: 1rem;

    aside {
      margin-bottom: 0.5rem;
      text-align: center;
      position: absolute;

      h2 {
        border-image: url("/border-big-off-white.png") 12 / 8px round;
        border-image-outset: 2px;
        background-color: #e6e6d2;
        padding: 0.25rem 0.5rem;
        position: relative;
        left: -6rem;
        top: -2rem;
        rotate: -4deg;
      }
    }

    .detail-contents {
      margin-top: 2rem;
    }

    .reveal {
      display: flex;
      gap: 0.5rem;
      align-items: center;
      overflow: hidden;
      padding: 0.25rem 0.25rem 0.25rem 0;
      margin-bottom: 0.5rem;
    }
  }

  #update {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    input {
      margin-bottom: 1rem;
    }
  }
</style>
