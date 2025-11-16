<script lang="ts">
  import { page } from '$app/state';
</script>

<div>
  <h1>{page.status}</h1>
  <p>{page.error?.message ?? "Something went wrong"}</p>
  <a href="/">Go Home</a>
</div>

<style>
  div {
    text-align: center;
    margin-top: 2rem;
  }
</style>
