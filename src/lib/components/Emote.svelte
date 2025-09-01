<script lang="ts">
  import { getEmoteDataURL } from "$lib/chat";
  export let id: string;
  export let name: string;

  let emoteSrcPromise: Promise<string> = getEmoteDataURL(id);
</script>

{#await emoteSrcPromise}
  <span
    title={name}
    style="display: inline-block; width: 1.6em; height: 1.6em; vertical-align: middle;"
  ></span>
{:then src}
  <img
    {src}
    alt={name}
    class="emote"
    title={name}
    style="max-height: 1.6em; vertical-align: middle;"
  />
{/await}
