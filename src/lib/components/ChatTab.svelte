<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, tick } from "svelte";

  // --- Interfaces ---
  interface Identity {
    color: string;
    badges: any[];
  }
  interface Sender {
    id: number;
    username: string;
    slug: string;
    identity: Identity;
  }
  interface ChatMessage {
    id: string;
    chatroomId: number;
    content: string;
    messageType: string;
    createdAt: string;
    sender: Sender;
  }

  // --- Props e Estado ---
  export let channelId: string;
  // MUDANÇA: Nova prop para receber o histórico
  export let initialMessages: ChatMessage[] = [];

  // MUDANÇA: O estado agora começa com as mensagens do histórico
  let messages: ChatMessage[] = initialMessages;

  let unlisten: UnlistenFn;
  let chatWindowElement: HTMLDivElement;

  // --- Lógica de Ciclo de Vida ---
  onMount(() => {
    // A lógica de escutar por NOVAS mensagens permanece a mesma
    const setupListener = async () => {
      unlisten = await listen<ChatMessage>("new-chat-message", (event) => {
        if (event.payload.chatroomId.toString() === channelId) {
          messages = [...messages, event.payload];
        }
      });
    };
    setupListener();

    return () => {
      if (unlisten) {
        console.log(`Parando de escutar o canal ${channelId}`);
        unlisten();
      }
    };
  });

  // --- Lógica de Scroll Automático ---
  $: if (messages.length > 0 && chatWindowElement) {
    (async () => {
      await tick();
      chatWindowElement.scroll({
        top: chatWindowElement.scrollHeight,
        behavior: "smooth",
      });
    })();
  }

  // --- Funções Auxiliares ---
  function parseEmotes(content: string): string {
    const emoteRegex = /\[emote:(\d+):(\w+)\]/g;
    return content.replace(emoteRegex, (match, emoteId, emoteName) => {
      const emoteUrl = `https://files.kick.com/emotes/${emoteId}/fullsize`;
      return `<img src="${emoteUrl}" alt="${emoteName}" class="emote" title="${emoteName}" />`;
    });
  }

  function formatTimestamp(isoString: string): string {
    try {
      const date = new Date(isoString);
      const hours = date.getHours().toString().padStart(2, "0");
      const minutes = date.getMinutes().toString().padStart(2, "0");
      return `${hours}:${minutes}`;
    } catch (e) {
      return "00:00";
    }
  }
</script>

<div bind:this={chatWindowElement} class="chat-window">
  {#if messages.length === 0}
    <p class="status">Aguardando mensagens para o canal {channelId}...</p>
  {:else}
    {#each messages as msg (msg.id)}
      <div class="message">
        <span class="timestamp">[{formatTimestamp(msg.createdAt)}]</span>
        <span class="author" style="color: {msg.sender.identity.color};">
          {msg.sender.username}:
        </span>
        <span class="content">{@html parseEmotes(msg.content)}</span>
      </div>
    {/each}
  {/if}
</div>

<style>
  .chat-window {
    height: 50vh;
    overflow-y: auto;
    padding: 1rem;
    background-color: #111827;
    display: flex;
    flex-direction: column;
    color: #d1d5db;
    font-family:
      "Inter",
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      Roboto,
      Oxygen,
      Ubuntu,
      Cantarell,
      "Open Sans",
      "Helvetica Neue",
      sans-serif;
    font-size: 14px;
  }
  .message {
    padding: 0.25rem 0.5rem;
    line-height: 1.6;
    border-radius: 4px;
    transition: background-color 0.2s;
  }
  .message:hover {
    background-color: #1f2937;
  }
  .timestamp {
    color: #6b7280;
    margin-right: 0.6em;
    font-size: 0.9em;
  }
  .author {
    font-weight: 600;
    margin-right: 0.6em;
  }
  .status {
    color: #9ca3af;
    text-align: center;
    margin: auto;
  }
  .content {
    color: #f9fafb;
    word-break: break-word;
  }
  .chat-window .message .content :global(.emote) {
    max-height: 1.6rem;
  }
</style>
