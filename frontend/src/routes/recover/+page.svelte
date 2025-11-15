<script lang="ts">
  import { isError, type ApiError, resendToken } from "$lib/api";

  let email = $state("");

  let sent = $state(false);

  const onsubmit = async (e: SubmitEvent) => {
    e.preventDefault();
    await resendToken(email);
    sent = true;
  };
</script>

<div class="content-container">
  <h1 class="heading">Recover Token</h1>
  <form id="register" {onsubmit}>
    <label for="email">Email:</label>
    <input
      type="text"
      name="email"
      id="email"
      placeholder="Email"
      bind:value={email}
    />
    <button type="submit">Send Recovery Email</button>

    {#if sent}
      <div class="success-text">
        If a team with that email exists, your team token has been resent.
      </div>
    {/if}
  </form>
</div>

<style>
  .heading {
    text-align: center;
    margin-bottom: 1rem;
  }

  #register {
    border: 1px solid var(--text-neutral);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    input {
      margin-bottom: 1rem;
    }
  }
</style>
