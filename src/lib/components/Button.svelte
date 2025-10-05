<script lang="ts">
  export let newMessagesCount: number;
</script>

<div class="flex justify-center">
  <button
    class:new-messages={newMessagesCount > 0}
    class="w-fit bg-red-600 animate-pulse text-white transform-3d rounded-2xl p-2 hover:cursor-pointer"
  >
    {#if newMessagesCount > 0}
      ↓ {newMessagesCount} Mensagens
    {:else}
      ↓ Final
    {/if}
  </button>
</div>
