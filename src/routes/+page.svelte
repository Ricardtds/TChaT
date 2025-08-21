<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let channelId: string = "";
  let messages: string[] = [];
  let status: string = "Desconectado.";

  async function connectChat() {
    status = "Conectando...";
    try {
      // O nome do comando permanece o mesmo
      await invoke("connect_chat_command", { channelId });
    } catch (e) {
      status = `Erro: ${e}`;
    }
  }

  onMount(() => {
    // Escutando os eventos do Rust
    // O nome do evento e a forma de escutar permanecem os mesmos
    const unlistenMessage = listen<string>("new_chat_message", (event) => {
      messages = [...messages, event.payload];
    });

    const unlistenStatus = listen<string>("chat_status", (event) => {
      status = event.payload;
    });

    return () => {
      unlistenMessage();
      unlistenStatus();
    };
  });
</script>

<main class="container">
  <h1>T-Chat</h1>
  <p>Status: {status}</p>

  <div class="connect-form">
    <input bind:value={channelId} placeholder="ID do Canal da Kick" />
    <button on:click={connectChat}>Conectar</button>
  </div>

  <div class="chat-window">
    {#each messages as msg}
      <div class="message">{msg}</div>
    {/each}
  </div>
</main>

<style>
  /* Seus estilos aqui */
</style>
