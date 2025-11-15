<script lang="ts">
  import { register, isError, type ApiError } from "$lib/api";

  let email = $state("");
  let name = $state("");
  let disabled = $state(false)

  let status: { message: string } | ApiError | null = $state(null);

  const onsubmit = async (e: SubmitEvent) => {
    e.preventDefault();
    disabled = true;

    const res = await register(email, name);
    if (isError(res)) status = res;
    else status = { message: `Verification email sent to ${email}.` };

    disabled = false;
  };
</script>

<div class="content-container">
  <h1 class="heading">Register</h1>
  <form id="register" {onsubmit}>
    <label for="email">Email:</label>
    <input type="text" name="email" id="email" placeholder="Email" bind:value={email} />
    <label for="name">Team Name:</label>
    <input type="text" name="name" id="name" placeholder="Team Name" bind:value={name} />
    <button type="submit" {disabled}>Register</button>

    {#if status}
      <div class="{isError(status) ? 'error' : 'success'}-text">
        {status.message}
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
