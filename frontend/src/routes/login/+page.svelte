<script lang="ts">
  import { isError, login, type ApiError } from "$lib/api";
  import type { PageProps } from "./$types";

  const { data }: PageProps = $props();
  let teamName = $state(data.teamName);
  let token = $state(data.token ?? "");
  let error: ApiError | null = $state(null);

  const onSubmit = async (e?: Event) => {
    e?.preventDefault();
    const res = await login(token);
    if (isError(res)) {
      error = res;
      token = "";
      teamName = undefined; // go to manual input if link does not work
    } else {
      window.location.href = `/profile/${res.id}`;
    }
  };
</script>

<div class="content-container">
  <h1 class="heading">Login</h1>

  <div class="login-container">
    {#if teamName && token}
      <p class="confirm-login">Logging in as <strong>{teamName}</strong></p>
      <button onclick={onSubmit}>Login</button>
    {:else if teamName === null}
      <div class="error-text">Failed to login: {data.error!.message}</div>
      <button onclick={() => (teamName = undefined)}>OK</button>
    {:else if teamName === undefined}
      <form id="manual-login-form">
        <div class="token-input-container">
          <input
            type="text"
            name="token"
            id="token-input"
            placeholder="Token"
            bind:value={token}
          />
          <button onclick={onSubmit}>Login</button>
        </div>
        <a href="/recover" class="recover">Lost token?</a>
      </form>
    {/if}

    {#if error}
      <div class="error-text">{error.message}</div>
    {/if}
  </div>
</div>

<style>
  .login-container {
    border: 1px solid var(--text-primary);
    padding: 1rem;
    text-align: center;
  }

  .heading {
    text-align: center;
    margin-bottom: 1rem;
  }

  .confirm-login {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
  }

  .token-input-container {
    width: 100%;
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
    input {
      flex: 1;
    }
  }
</style>
