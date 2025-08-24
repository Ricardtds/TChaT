<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

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

  // --- Props (Svelte 5) ---
  const { channelId, active } = $props<{
    channelId: string;
    active: boolean;
  }>();

  // --- Estado do Componente ---
  let messages = $state<ChatMessage[]>([]);
  let unlisten: UnlistenFn;
  let chatWindowElement: HTMLDivElement;

  let userHasScrolledUp = $state(false);
  let newMessagesCount = $state(0);
  let isResumingScroll = false;
  let isAutoScrolling = false;
  let lastScrollTop = 0;

  // --- Estado para a lógica de contagem regressiva ---
  let messageElements: HTMLElement[] = [];
  let pendingMessages: ChatMessage[] = $state([]);

  /**
   * Rola a janela de chat para o fundo.
   */
  async function scrollToBottom(behavior: "smooth" | "auto" = "auto") {
    isAutoScrolling = true;
    await tick();
    if (chatWindowElement) {
      chatWindowElement.scrollTo({
        top: chatWindowElement.scrollHeight,
        behavior,
      });
    }
    setTimeout(() => {
      isAutoScrolling = false;
    }, 150);
  }

  let firstTime: boolean = true;
  $effect(() => {
    if (active && chatWindowElement && firstTime) {
      scrollToBottom();
      firstTime = false;
    }
  });

  onMount(() => {
    const setupDatabase = async () => {
      messages = await invoke("get_chat_history", {
        chatroomId: channelId,
      });
    };
    setupDatabase();

    const throttledHandleScroll = throttle(handleScroll, 100);
    chatWindowElement.addEventListener("scroll", throttledHandleScroll);

    const setupListener = async () => {
      unlisten = await listen<ChatMessage>("new-chat-message", (event) => {
        if (event.payload.chatroomId.toString() === channelId) {
          const isAtBottomBeforeUpdate = !userHasScrolledUp;
          if (userHasScrolledUp) {
            pendingMessages.push(event.payload);
            newMessagesCount = pendingMessages.length;
          } else if (isAtBottomBeforeUpdate) {
            if (messages.length >= 500) {
              messages.shift();
            }
            messages.push(event.payload);
            scrollToBottom("auto");
          }
        }
      });
    };
    setupListener();
    return () => {
      if (unlisten) unlisten();
      chatWindowElement.removeEventListener("scroll", throttledHandleScroll);
    };
  });

  /**
   * Função executada no evento de scroll para controlar o estado.
   */
  function handleScroll() {
    if (isAutoScrolling || isResumingScroll || !chatWindowElement) return;

    const currentScrollTop = chatWindowElement.scrollTop;
    const scrollHeight = chatWindowElement.scrollHeight;
    const clientHeight = chatWindowElement.clientHeight;
    const isAtBottom = scrollHeight - currentScrollTop - clientHeight < 100;

    if (isAtBottom) {
      userHasScrolledUp = false;
      newMessagesCount = 0;
      if (pendingMessages.length > 0) {
        messages = [...messages, ...pendingMessages];
        pendingMessages = [];
        while (messages.length > 500) {
          messages.shift();
        }
      }
    } else {
      if (currentScrollTop < lastScrollTop) {
        userHasScrolledUp = true;
      }
      if (userHasScrolledUp && pendingMessages.length > 0) {
        const chatWindowRect = chatWindowElement.getBoundingClientRect();
        let lastVisibleMessageIndex = -1;
        const allMessages = [...messages, ...pendingMessages];

        for (let i = allMessages.length - 1; i >= 0; i--) {
          const messageEl = messageElements[i];
          if (messageEl) {
            const rect = messageEl.getBoundingClientRect();
            if (rect.top < chatWindowRect.bottom) {
              lastVisibleMessageIndex = i;
              break;
            }
          }
        }

        if (lastVisibleMessageIndex !== -1) {
          const pendingStartIndex = messages.length;
          if (lastVisibleMessageIndex < pendingStartIndex) {
            newMessagesCount = pendingMessages.length;
          } else {
            const remaining =
              allMessages.length - (lastVisibleMessageIndex + 1);
            newMessagesCount = Math.max(0, remaining);
          }
        }
      }
    }
    lastScrollTop = currentScrollTop <= 0 ? 0 : currentScrollTop;
  }

  /**
   * Chamado ao clicar no botão para voltar ao fundo.
   */
  function resumeAutoScroll() {
    isResumingScroll = true;
    userHasScrolledUp = false;
    newMessagesCount = 0;

    if (pendingMessages.length > 0) {
      messages = [...messages, ...pendingMessages];
      pendingMessages = [];
      while (messages.length > 500) {
        messages.shift();
      }
    }

    scrollToBottom("smooth");
    setTimeout(() => {
      isResumingScroll = false;
    }, 500);
  }

  // --- Funções de Utilidade ---
  function throttle<T extends (...args: any[]) => any>(
    func: T,
    limit: number
  ): T {
    let inThrottle: boolean;
    return function (this: ThisParameterType<T>, ...args: Parameters<T>): void {
      if (!inThrottle) {
        func.apply(this, args);
        inThrottle = true;
        setTimeout(() => (inThrottle = false), limit);
      }
    } as T;
  }

  function parseEmotes(content: string): string {
    const emoteRegex = /\[emote:(\d+):(\w+)\]/g;
    return content.replace(emoteRegex, (match, emoteId, emoteName) => {
      const emoteUrl = `https://files.kick.com/emotes/${emoteId}/fullsize`;
      return `<img src="${emoteUrl}" alt="${emoteName}" class="emote" title="${emoteName}" style="max-height: 1.6em; vertical-align: middle;" />`;
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

<div class="chat-container">
  <div bind:this={chatWindowElement} class="chat-window">
    {#if messages.length === 0 && active}
      <p class="status">Aguardando mensagens para o canal {channelId}...</p>
    {:else}
      {@const allMessages = [...messages, ...pendingMessages]}
      {#each allMessages as msg, i (msg.id)}
        <div class="message" bind:this={messageElements[i]}>
          <span class="timestamp">[{formatTimestamp(msg.createdAt)}]</span>
          <span class="author" style="color: {msg.sender.identity.color};">
            {msg.sender.username}:
          </span>
          <span class="content">{@html parseEmotes(msg.content)}</span>
        </div>
      {/each}
    {/if}
  </div>

  {#if userHasScrolledUp}
    <div class="scroll-button-container">
      <button
        onclick={resumeAutoScroll}
        class:new-messages={newMessagesCount > 0}
      >
        {#if newMessagesCount > 0}
          ↓ {newMessagesCount} Novas Mensagens
        {:else}
          ↓ Voltar para o final
        {/if}
      </button>
    </div>
  {/if}
</div>

<style>
  .chat-container {
    position: relative;
    display: flex;
    flex-grow: 1;
    min-height: 0;
    flex-direction: column;
  }

  .chat-window {
    padding: 0.5rem;
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
    flex-grow: 1;
    width: 100%;
    min-height: 0;
    overflow-y: scroll;
  }

  .chat-window::-webkit-scrollbar {
    width: 10px;
  }
  .chat-window::-webkit-scrollbar-track {
    background: transparent;
  }
  .chat-window::-webkit-scrollbar-thumb {
    background: #555;
    border-radius: 5px;
  }
  .chat-window::-webkit-scrollbar-thumb:hover {
    background: #777;
  }

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
</style>
