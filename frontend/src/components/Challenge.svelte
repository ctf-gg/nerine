<script lang="ts">
  import { marked } from "marked";
  import {
    type Challenge,
    type ChallengeDeployment,
    challenges,
    deployChallenge,
    getChallengeDeployment,
    isError,
    submitFlag,
  } from "../api";
  const { chall: c }: { chall: Challenge } = $props();

  let flagInput: HTMLInputElement;
  let deployment: ChallengeDeployment | null = $state(null);
  const urls: { type: "tcp" | "http"; url: string }[] = $derived.by(() => {
    if (!deployment || !deployment?.data) return [];
    let res: { type: "tcp" | "http"; url: string }[] = [];
    for (const [port, mapping] of Object.entries(deployment.data.ports)) {
      switch (mapping.type) {
        case "tcp":
          res.push({ type: "tcp", url: "nc smiley.cat " + mapping.port });
          break;
        case "http":
          res.push({
            type: "http",
            url: `https://${mapping.subdomain}.${mapping.base}`,
          });
      }
    }

    return res;
  });
  let correct = $state<boolean | null>(null);
  async function submit(e: SubmitEvent) {
    e.preventDefault();
    const j = await submitFlag(c.id, flagInput.value);
    flagInput.value = "";
    if (isError(j)) {
      alert("uwuups! " + j.message);
    }
    correct = !isError(j);
  }

  async function deployInstance() {
    const res = await deployChallenge(c.id);
    if (isError(res)) {
      alert("something went wrong with deploying: " + JSON.stringify(res));
      return;
    }

    deployment = res;
  }

  async function getUrl() {
    const res = await getChallengeDeployment(c.deploymentId!);
    if (isError(res)) {
      alert("something went: " + JSON.stringify(res));
      return;
    }

    deployment = res;
  }

  $effect(() => {
    if (c.strategy === "instanced" && c.deploymentId) {
      getChallengeDeployment(c.deploymentId).then((r) => {
        if (isError(r)) {
          console.log(r);
        } else {
          deployment = r;
        }
      });
    }
  });
</script>

<div class="challenge">
  <div class="header">
    <h1>{c.category}/{c.name}</h1>

    <h2>{c.solves} solves / {c.points} points</h2>
  </div>
  <div class="subheader">
    <h2 class="author">{c.author}</h2>
    {#if c.selfSolved}
      <h2 class="solve-text">Solved by you!</h2>
    {/if}
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
        {#each urls as url}
          {#if url.type === "tcp"}
            <button onclick={() => navigator.clipboard.writeText(url.url)}>
              {url.url}
            </button>
          {:else}
            <a href={url.url}>
              <button>{url.url}</button>
            </a>
          {/if}
          {#if deployment.expired_at}
            <button disabled
              >Expires at {new Date(
                deployment.expired_at
              ).toLocaleTimeString()}</button
            >
          {/if}
        {/each}
      {:else if c.strategy === "static" && c.deploymentId}
        <button onclick={getUrl}>Show URL</button>
      {:else if c.strategy === "static" && !c.deploymentId}
        <div></div>
      {:else if c.strategy === "instanced" && c.deploymentId}
        <button>Loading...</button>
      {:else if c.strategy === "instanced" && !c.deploymentId}
        <button onclick={deployInstance}>Create Instance</button>
      {/if}
    </div>
  </div>
  {#if !c.selfSolved}
    <form class="submit" onsubmit={submit}>
      <input
        type="text"
        name="flag"
        placeholder="flag"
        autocomplete="off"
        onchange={() => (correct = null)}
        bind:this={flagInput}
        class={[{ correct: correct === true, incorrect: correct === false }]}
      />
      <button type="submit">submit</button>
    </form>
  {/if}
</div>

<style>
  .challenge {
    border-image: url("/border-big-off-white.png") 12 / 8px round;
    border-image-outset: 2px;
    background-color: #e6e6d2;
    padding: 1rem;

    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      h1 {
        font-size: 2rem;
      }
    }

    .subheader {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    .author,
    .solve-text {
      margin-top: -0.5rem;
    }

    .description {
      font-family: "Satoshi", sans-serif;
      margin-top: 0.25rem;
      margin-bottom: 1rem;
    }

    .resources {
      display: grid;
      grid-template-columns: 1fr 1fr;
      margin-bottom: 0.5rem;
      button {
        font-size: 1rem;
      }
    }

    .deployment {
      display: flex;
      gap: 0.5rem;
      justify-content: end;
    }

    .attachments {
      display: flex;
      gap: 0.5rem;
    }
  }

  .submit {
    width: 100%;
    display: flex;

    input {
      flex-grow: 1;
      transition: background-color 300ms ease-out;

      &.correct {
        background-color: lightgreen;
      }

      &.incorrect {
        background-color: pink;
      }
    }
  }
</style>
