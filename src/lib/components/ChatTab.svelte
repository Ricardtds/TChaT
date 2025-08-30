<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, tick } from "svelte";
  import { formatTimestamp, parseEmotes } from "$lib/utils";
  import Button from "$lib/components/Button.svelte";
  import type { ChatMessage } from "$lib/chat";

  // --- Props (Svelte 5) ---
  const { channelId, active } = $props<{
    channelId: string;
    active: boolean;
  }>();

  // --- CONSTANTE PARA QUANTIDADE DE LINHAS ---
  const ROWS_QTD = 200;
  const HISTORY_PAGE_SIZE = 250; // Quantidade para buscar no scroll infinito

  // --- Estado do Componente ---
  let messages = $state<ChatMessage[]>([]);
  let pendingMessages: ChatMessage[] = $state([]);
  let unlisten: UnlistenFn;
  let chatWindowElement: HTMLDivElement;
  let messageElements: HTMLElement[] = $state([]);

  let userHasScrolledUp = $state(false);
  let newMessagesCount = $state(0);
  let isResumingScroll = false;
  let isAutoScrolling = false;
  let lastScrollTop = 0;

  let isLoadingHistory = $state(false);
  let hasMoreHistoryToLoad = $state(true);

  async function scrollToBottom(behavior: "smooth" | "auto" = "auto") {
    isAutoScrolling = true;
    await tick();
    if (chatWindowElement) {
      chatWindowElement.scrollTo({
        top: chatWindowElement.scrollHeight,
        behavior,
      });
      lastScrollTop = chatWindowElement.scrollTop;
    }
    setTimeout(() => {
      isAutoScrolling = false;
    }, 150);
  }

  let firstTime: boolean = true;
  $effect(() => {
    if (active && chatWindowElement && messages.length > 0 && firstTime) {
      scrollToBottom();
      firstTime = false;
    }
  });

  onMount(() => {
    const setupDatabase = async () => {
      messages = await invoke("get_chat_history", {
        chatroomId: channelId,
        rowsQtd: ROWS_QTD,
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
            if (messages.length >= ROWS_QTD) {
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

  async function loadMoreHistory() {
    if (isLoadingHistory || !hasMoreHistoryToLoad || messages.length === 0) {
      return;
    }
    isLoadingHistory = true;
    const oldestMessageDate = messages[0].createdAt;
    console.log("Catando mensagens");
    console.log(oldestMessageDate);

    const olderMessages = await invoke<ChatMessage[]>("get_older_messages", {
      chatroomId: channelId,
      beforeDate: oldestMessageDate,
      rowsQtd: HISTORY_PAGE_SIZE,
    });

    if (olderMessages.length > 0) {
      const oldScrollHeight = chatWindowElement.scrollHeight;
      messages = [...olderMessages, ...messages];
      await tick();
      const newScrollHeight = chatWindowElement.scrollHeight;
      chatWindowElement.scrollTop += newScrollHeight - oldScrollHeight;
    } else {
      hasMoreHistoryToLoad = false;
    }
    isLoadingHistory = false;
  }

  function handleScroll() {
    if (isAutoScrolling || isResumingScroll || !chatWindowElement) return;
    console.log(chatWindowElement.scrollTop);

    if (chatWindowElement.scrollTop <= 200 && !isLoadingHistory) {
      loadMoreHistory();
    }

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
        while (messages.length > ROWS_QTD) {
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

  function resumeAutoScroll() {
    isResumingScroll = true;
    userHasScrolledUp = false;
    newMessagesCount = 0;

    if (pendingMessages.length > 0) {
      messages = [...messages, ...pendingMessages];
      pendingMessages = [];
      while (messages.length > ROWS_QTD) {
        messages.shift();
      }
    }
    scrollToBottom("smooth");
    setTimeout(() => {
      isResumingScroll = false;
    }, 500);
  }

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
</script>

<div class="chat-container">
  <div bind:this={chatWindowElement} class="chat-window">
    {#if isLoadingHistory}
      <div class="loading-spinner">Carregando histórico...</div>
    {/if}

    {#if messages.length === 0 && pendingMessages.length === 0 && active && !isLoadingHistory}
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
    <Button {newMessagesCount} {resumeAutoScroll}></Button>
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
  .loading-spinner {
    padding: 1rem;
    text-align: center;
    color: #9ca3af;
    font-size: 0.875rem;
  }
</style>
