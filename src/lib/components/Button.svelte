<script lang="ts">
  export let resumeAutoScroll: () => void;
  export let newMessagesCount: number;
</script>

<div class="scroll-button-container">
  <button onclick={resumeAutoScroll} class:new-messages={newMessagesCount > 0}>
    {#if newMessagesCount > 0}
      ↓ {newMessagesCount} Novas Mensagens
    {:else}
      ↓ Voltar para o final
    {/if}
  </button>
</div>

<style>
  .scroll-button-container {
    position: absolute;
    bottom: 1rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 10;
  }
  .scroll-button-container button {
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 9999px;
    padding: 0.5rem 1rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    backdrop-filter: blur(4px);
    transition: all 0.2s ease-in-out;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    white-space: nowrap;
  }
  .scroll-button-container button:hover {
    transform: scale(1.05);
  }
  .scroll-button-container button.new-messages {
    background-color: #ef4444;
    animation: pulse 1.5s infinite;
  }
  @keyframes pulse {
    0%,
    100% {
      transform: scale(1);
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }
    50% {
      transform: scale(1.05);
      box-shadow: 0 6px 12px rgba(239, 68, 68, 0.3);
    }
  }
</style>
