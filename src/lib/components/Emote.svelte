<script lang="ts">
  import { getEmoteDataURL } from "$lib/chat";
  export let id: string;
  export let name: string;

  let emoteSrcPromise: Promise<string> = getEmoteDataURL(id);
</script>

{#await emoteSrcPromise}
  <span title={name}></span>
{:then src}
  <img
    {src}
    alt={name}
    title={name}
    class="inline"
    style="max-height: 1.6em; vertical-align: middle;"
  />
{/await}
