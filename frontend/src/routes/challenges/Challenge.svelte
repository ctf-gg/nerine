<script lang="ts">
  import { marked } from "marked";
  import {
    type ApiError,
    type Challenge,
    type ChallengeDeployment,
    challenges,
    type ChallengeSolve,
    challengeSolves,
    deployChallenge,
    destroyChallenge,
    type Event,
    getChallengeDeployment,
    isError,
    submitFlag,
  } from "$lib/api";
  import TcpLink from "./TcpLink.svelte";
  import { onDestroy, onMount } from "svelte";
  const { chall: c, event }: { challenge: Challenge, event: Event } = $props();

  let flagInput: HTMLInputElement = $state(null!);
  let deployment: ChallengeDeployment | null = $state(null);
  const urls: { type: "tcp" | "http"; url: string }[] = $derived.by(() => {
    if (!deployment || !deployment?.data) return [];
    let res: { type: "tcp" | "http"; url: string }[] = [];
    for (const [name, { ports }] of Object.entries(deployment.data)) {
      for (const mapping of Object.values(ports)) {
        switch (mapping.type) {
          case "tcp":
            res.push({ type: "tcp", url: `nc ${mapping.base} ${mapping.port}` });
            break;
          case "http":
            res.push({
              type: "http",
              url: `https://${mapping.subdomain}.${mapping.base}`,
            });
        }
      }
    }

    return res;
  });

  let error: ApiError | null = $state(null);

  let correct = $state<boolean | null>(null);
  async function submit(e?: SubmitEvent) {
    e?.preventDefault();
    const j = await submitFlag(c.id, flagInput.value);
    flagInput.value = "";
    if (isError(j) && j.error != "wrong_flag") {
      error = j;
    }
    correct = !isError(j);
    if (correct) {
      c.solved_at = new Date();
    }
  }

  // Instancing

  const poller = async () => {
    const dep = await getChallengeDeployment(res.id);
    if (isError(dep)) {
      error = dep;
      return;
    }

    if (dep.data) {
      deployment = dep;
      if (dep.expired_at) {
	const totalTime =
	      new Date(dep.expired_at + 'Z').getTime() - new Date().getTime();
	instanceTimeRemaining = totalTime;
	if (!totalInstanceTime) totalInstanceTime = totalTime;
      }

      waiting = false;
    }

    if (dep.destroyed_at) {
      await destroyInstance(false);
      if (interval) clearInterval(interval);
    }
  }

  let waiting = $state(false);
  let interval: any = $state(null);
  let createCooldown = $state(false);

  async function deployInstance() {
    waiting = true;
    const res = await deployChallenge(c.id);
    if (isError(res)) {
      error = res;
      return;
    }
    interval = setInterval(poller, 2000);
  }

  let instanceTimeInterval: number = $state(null!);
  let instanceTimeRemaining: number | null = $state(null);
  let totalInstanceTime: number | null = $state(null);
  // TODO(aiden): investiagte whether its bad to have 40 intervals going off at once
  onMount(() => {
    instanceTimeInterval = setInterval(() => {
      if (deployment && deployment.expired_at) {
        instanceTimeRemaining =
          new Date(deployment.expired_at + 'Z').getTime() - new Date().getTime();
      } else {
        instanceTimeRemaining = null;
      }
    }, 1000);
  });

  async function destroyInstance(actuallyDestroy = true) {
    if (actuallyDestroy) {
      waiting = true;
      const res = await destroyChallenge(c.id);
      if (res != "ok") {
        error = res;
        return;
      }
    }

    waiting = false;
    deployment = null;
    c.deployment_id = "";
    createCooldown = true;
    totalInstanceTime = null;
    instanceTimeRemaining = null;
    if (interval) clearInterval(interval);

    setTimeout(() => {
      createCooldown = false;
    }, 2000);
  }

  async function getUrl() {
    const res = await getChallengeDeployment(c.deployment_id!);
    if (isError(res)) {
      error = res;
      return;
    }

    if (res.data) {
      deployment = res;
    }
  }

  $effect(() => {
    if (c.strategy === "instanced" && c.deployment_id) {
      getChallengeDeployment(c.deployment_id).then((r) => {
        if (isError(r)) {
          error = r;
        } else {
          deployment = r;
	  interval = setInterval(poller, 2000);
	  if (r.expired_at) {
            const totalTime =
              new Date(r.expired_at + 'Z').getTime() - new Date().getTime();
            instanceTimeRemaining = totalTime;
            totalInstanceTime = totalTime;
          }
        }
      });
    }
  });

  onDestroy(() => {
    if (interval) clearInterval(interval);
    clearInterval(instanceTimeInterval);
  });

  // Solves Dialog
  let solvesDialog: HTMLDialogElement = $state(null!);
  let solves: ChallengeSolve[] | ApiError | null = $state(null);

  async function showSolves() {
    solvesDialog.showModal();
    solves = await challengeSolves(c.id);
  }
</script>

<div class="challenge">
  <div class="header">
    <h1>{c.category}/{c.name}</h1>

    <button class="ghost solves" onclick={showSolves}>
      {c.solves} solve{c.solves === 1 ? "" : "s"} / {c.points} points
    </button>
  </div>
  <div class="subheader">
    <span class="author">{c.author}</span>
  </div>
  <p class="description">{@html marked(c.description)}</p>
  <div class="resources">
    <div class="attachments">
      {#if c.attachments}
        {#each Object.entries(c.attachments) as [name, url]}
          <a href={url} download><button>{name}</button></a>
        {/each}
      {/if}
    </div>
    <div class="deployment">
      {#if deployment}
        <div class="deployed-info">
          <div class="deployment-controls">
            {#if deployment.expired_at && instanceTimeRemaining}
              {@const totalSecs = instanceTimeRemaining / 1000}
              {@const minutes = Math.floor(totalSecs / 60)
                .toString()
                .padStart(2, "0")}
              {@const seconds = Math.round(totalSecs % 60).toString().padStart(2, "0")}
              <div class="expire-bar">
                <div style="width: {instanceTimeRemaining / totalInstanceTime * 100}%;" class="bar-fill"></div>
                <span class="bar-text">Expires in {minutes}:{seconds}</span>
              </div>
            {/if}
            {#if c.strategy === "instanced"}
              <button onclick={() => destroyInstance()}>Destroy</button>
            {/if}
          </div>
          {#each urls as url}
            {#if url.type === "tcp"}
              <TcpLink link={url.url} /><br />
            {:else}
              <a href={url.url} target="_blank">
                {url.url}
              </a>
	      <br />
            {/if}
          {/each}
        </div>
      {:else if c.strategy === "static" && c.deployment_id}
        <button onclick={getUrl}>Show URL</button>
      {:else if waiting || (c.strategy === "instanced" && c.deployment_id)}
        <button class="loading" disabled>
          <svg
            class="spinner"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              opacity="0.25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              stroke-width="4"
            ></circle>
            <path
              opacity="0.75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            ></path>
          </svg>
          <span>
            {c.strategy === "static" ? "Loading" : "Creating Instance"}
          </span>
        </button>
      {:else if c.strategy === "instanced" && !c.deployment_id}
        <button onclick={deployInstance} disabled={createCooldown}>
          Create Instance
        </button>
      {/if}
    </div>
  </div>
  {#if c.solved_at}
    <div class="solved">Solved at {c.solved_at.toLocaleString()}</div>
    <!-- {:else if eventHasEnded}
    <div class="ended">The event has ended</div> -->
  {:else}
    <form class="submit" onsubmit={submit}>
      <input
        type="text"
        name="flag"
        placeholder="Flag"
        autocomplete="off"
        onchange={() => (correct = null)}
        bind:this={flagInput}
        class={[correct === false && "incorrect"]}
      />
      <button type="submit">Submit</button>
    </form>
  {/if}

  {#if error}
    <div class="error-text">{error.message}</div>
  {/if}
</div>

<dialog bind:this={solvesDialog} closedby="any" class="solves-dialog">
  <button
    class="ghost close"
    onclick={() => solvesDialog.close()}
    aria-label="close dialog"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      stroke-width="1.5"
      stroke="currentColor"
    >
      <path
        stroke-linecap="round"
        stroke-linejoin="round"
        d="M6 18 18 6M6 6l12 12"
      />
    </svg>
  </button>
  <h2>Solves for {c.category}/{c.name}</h2>
  {#if !solves}
    <div class="loading">Loading...</div>
  {:else if isError(solves)}
    <div class="error">Failed to get solves: {solves.message}</div>
    <button onclick={showSolves}>Retry</button>
  {:else if solves.length === 0}
  <div>This challenge has no solves</div>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Team</th>
          <th>Solved At</th>
        </tr>
      </thead>
      <tbody>
        {#each solves as solve}
          {@const href = `/profile/${solve.id}`}
          <tr>
            <td><a {href}>{solve.name}</a></td>
            <td><a {href}>{solve.solved_at.toLocaleString()}</a></td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</dialog>

<style>
  .challenge {
    border: 1px solid var(--text-neutral);
    padding: 1rem;

    .header {
      line-height: 1;
      display: flex;
      justify-content: space-between;
      align-items: center;
      h1 {
        font-size: 2rem;
      }

      .solves {
        white-space: nowrap;
        font-weight: 600;
        font-size: 1.25rem;
        &:hover {
          text-decoration: underline 2px;
        }
      }
    }

    @media (max-width: 768px) {
      .header {
        flex-direction: column;
        align-items: flex-start;

        .solves {
          margin-bottom: 0.75rem;
        }
      }
    }

    .author {
      font-size: 1.25rem;
    }

    .solved,
    .ended {
      height: 2rem;
      background: var(--bg);
      color: var(--text);
      border: 1px solid var(--text);
      font-weight: 600;
      text-align: center;
      align-content: center;
    }

    .solved {
      --bg: var(--bg-success);
      --text: var(--text-success);
    }

    .ended {
      --bg: var(--bg-neutral);
      --text: var(--text-neutral);
    }

    .description {
      margin-top: 0.5rem;
      margin-bottom: 1rem;
    }

    .resources {
      display: flex;
      margin-bottom: 0.5rem;
      button {
        font-size: 1rem;
      }
    }

    .deployment {
      margin-left: auto;
      display: flex;
      gap: 0.5rem;
      justify-content: end;

      .deployment-controls {
        margin-left: auto;
        display: flex;
        gap: 0.5rem;
        justify-content: end;
      }

      .expire-bar {
        display: grid;
        width: 12rem;
        border: 1px solid var(--text-accent);
        background: var(--bg-accent);
        align-items: center;
        .bar-fill {
          opacity: 0.5;
          padding: 0.15rem 0;
          background: var(--text-accent);
          height: 100%;
        }

        .bar-text {
          z-index: 10;
          text-align: center;
        }

        * {
          grid-column-start: 1;
          grid-row-start: 1;
        }
      }
    }

    .attachments {
      display: flex;
      gap: 0.5rem;
      align-items: end;
    }
  }

  .submit {
    width: 100%;
    display: flex;
    height: 2rem;

    input {
      flex-grow: 1;

      &.incorrect {
        background-color: var(--bg-error);
      }
    }
  }

  button.loading {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    .spinner {
      height: 1rem;
      width: 1rem;
      display: inline;
      animation: spin 2s infinite linear;
    }
  }

  @keyframes spin {
    from {
      rotate: 0deg;
    }
    to {
      rotate: 360deg;
    }
  }

  .solves-dialog {
    min-width: 45rem;
    text-align: center;
    background: var(--bg-neutral);
    color: var(--text-primary);

    h2 {
      margin-bottom: 0.5rem;
    }

    .close {
      position: absolute;
      right: 1rem;

      svg {
        height: 1.5rem;
        width: 1.5rem;
      }
    }

    .error {
      margin-bottom: 0.25rem;
    }

    table {
      border-collapse: collapse;
    }

    tbody tr {
      transition: all 300ms;
      &:hover {
        background: var(--bg-neutral);
      }
    }

    td {
      border: none;
      border-top: 1px solid var(--text-neutral);
      border-bottom: 1px solid var(--text-neutral);
      padding: 0;
      a {
        display: block;
        padding: 0.15rem 0;
        text-decoration: none;
      }
    }
  }
</style>
