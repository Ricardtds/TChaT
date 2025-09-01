<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import VirtualList from "@sveltejs/svelte-virtual-list";
  import type { ChatMessage } from "$lib/chat";
  import Message from "./Message.svelte";
  import Button from "$lib/components/Button.svelte";

  export let channelId: number;
  export let active: boolean;

  const ROWS_QTD = 500; // Limite de mensagens em memória
  const HISTORY_PAGE_SIZE = 250; // Histórico por página
  const ITEM_HEIGHT = 60; // Altura média da mensagem

  let messages: ChatMessage[] = [];
  let pendingMessages: ChatMessage[] = [];
  let unlisten: UnlistenFn;
  let chatWindowElement: HTMLDivElement;

  let userHasScrolledUp = false;
  let newMessagesCount = 0;
  let isLoadingHistory = false;
  let hasMoreHistoryToLoad = true;

  let start = 0;
  let end = 0;

  // --- Inicializa histórico e listener ---
  onMount(() => {
    const setupDatabase = async () => {
      const now = new Date();
      const initialMessages: ChatMessage[] = await invoke("get_chat_history", {
        chatroomId: channelId,
        beforeDate: now.toISOString(),
        rowsQtd: ROWS_QTD,
      });
      messages = [...initialMessages];
      await tick();
      end = messages.length - 1; // força VirtualList para o final
    };

    const setupListener = async () => {
      unlisten = await listen<ChatMessage>(
        "new-chat-message",
        async (event) => {
          if (event.payload.chatroomId !== channelId) return;

          if (userHasScrolledUp) {
            pendingMessages = [...pendingMessages, event.payload];
            newMessagesCount = pendingMessages.length;
          } else {
            const prevHeight = chatWindowElement.scrollHeight;
            messages = [...messages, event.payload];
            if (messages.length > ROWS_QTD)
              messages.splice(0, messages.length - ROWS_QTD);
            await tick();
            chatWindowElement.scrollTop +=
              chatWindowElement.scrollHeight - prevHeight;
            end = messages.length - 1;
          }
        }
      );
    };

    setupDatabase();
    setupListener();

    return () => {
      unlisten?.();
    };
  });

  // --- Infinite scroll ---
  async function loadMoreHistory() {
    if (isLoadingHistory || !hasMoreHistoryToLoad) return;
    isLoadingHistory = true;

    const oldestDate = messages[0]?.createdAt;
    if (!oldestDate) {
      isLoadingHistory = false;
      return;
    }

    const olderMessages: ChatMessage[] = await invoke("get_chat_history", {
      chatroomId: channelId,
      beforeDate: oldestDate,
      rowsQtd: HISTORY_PAGE_SIZE,
    });

    if (olderMessages.length) {
      const prevHeight = chatWindowElement.scrollHeight;
      messages = [...olderMessages, ...messages];
      await tick();
      chatWindowElement.scrollTop +=
        chatWindowElement.scrollHeight - prevHeight;
    } else {
      hasMoreHistoryToLoad = false;
    }

    isLoadingHistory = false;
  }

  $: end;
  // --- Scroll handler ---
  function handleScroll() {
    if (!chatWindowElement) return;

    const scrollTop = chatWindowElement.scrollTop;
    const scrollHeight = chatWindowElement.scrollHeight;
    const clientHeight = chatWindowElement.clientHeight;

    const isAtBottom = scrollHeight - scrollTop - clientHeight < 10;
    userHasScrolledUp = !isAtBottom;

    if (start <= 20 && !isLoadingHistory) loadMoreHistory();

    if (isAtBottom && pendingMessages.length > 0) {
      messages = [...messages, ...pendingMessages];
      pendingMessages = [];
      newMessagesCount = 0;
      end = messages.length - 1;
      tick().then(() => {
        chatWindowElement.scrollTop = chatWindowElement.scrollHeight;
      });
    } else {
      newMessagesCount = pendingMessages.length;
    }
  }

  // --- Botão de novas mensagens ---
  function resumeAutoScroll() {
    messages = [...messages, ...pendingMessages];
    pendingMessages = [];
    newMessagesCount = 0;
    tick().then(() => {
      end = messages.length - 1;
      chatWindowElement.scrollTop = chatWindowElement.scrollHeight;
    });
  }
</script>

<div class="chat-container">
  <div
    bind:this={chatWindowElement}
    class="chat-window"
    on:scroll={handleScroll}
  >
    {#if isLoadingHistory}
      <div class="loading-spinner">Carregando histórico...</div>
    {/if}

    <VirtualList
      items={messages}
      bind:start
      bind:end
      let:item
      itemSize={ITEM_HEIGHT}
    >
      {#if item}
        {#key item.id ?? item.createdAt}
          <Message msg={item} />
        {/key}
      {/if}
    </VirtualList>

    {#if end < messages.length}
      <Button {resumeAutoScroll} newMessagesCount={messages.length - end} />
    {/if}

    <p>showing {start}-{end} of {messages.length}</p>
  </div>
</div>

<style>
  .chat-container {
    position: relative;
    display: flex;
    flex-direction: column;
    flex-grow: 1;
    min-height: 0;
  }

  .chat-window {
    padding: 0.5rem;
    background-color: #111827;
    display: flex;
    flex-direction: column;
    color: #d1d5db;
    font-family: "Inter", system-ui, sans-serif;
    font-size: 14px;
    flex-grow: 1;
    width: 100%;
    min-height: 0;
    overflow-y: auto;
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

  .loading-spinner {
    padding: 1rem;
    text-align: center;
    color: #9ca3af;
    font-size: 0.875rem;
  }
</style>
