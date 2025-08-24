<script lang="ts">
  import {
    isError,
    verifyEmail,
    verifyEmailUpdate,
    type ApiError,
  } from "$lib/api";
  import type { PageProps } from "./$types";

  const { data }: PageProps = $props();
  const { verification, verificationToken } = data;

  let error: ApiError | null = $state(null);

  const onclick = async () => {
    let verifyRes;
    if (verification.type === "team_registration") {
      verifyRes = await verifyEmail(verificationToken);
    } else if (verification.type === "email_update") {
      verifyRes = await verifyEmailUpdate(verificationToken);
    } else {
      alert("invalid verification type");
      return;
    }

    if (isError(verifyRes)) {
      error = verifyRes;
    } else {
      window.location.href = `/profile/${verifyRes.id}`;
    }
  };
</script>

<div class="content-container">
  <h1 class="heading">
    Verify
    {#if verification.type === "team_registration"}
      Registration
    {:else}
      Email Update
    {/if}
  </h1>

  <div id="confirmation">
    <p>
      {#if verification.type === "team_registration"}
        Are you sure you want to register team "<strong
          >{verification.name}</strong
        >" with email "<strong>{verification.email}</strong>"?
      {:else if verification.type === "email_update"}
        Are you sure you want to update your email to "<strong
          >{verification.new_email}</strong
        >" for team "<strong>{verification.name}</strong>"?
      {/if}
    </p>

    <button {onclick}>Verify</button>
    {#if error}<div class="error-text">{error.message}</div>{/if}
  </div>
</div>

<style>
  .heading {
    text-align: center;
    margin-bottom: 1rem;
  }

  #confirmation {
    text-align: center;
    border: 1px solid var(--text-primary);
    padding: 1rem;
    button {
      margin-top: 0.5rem;
    }
  }
</style>
