<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onMount, tick } from "svelte";

  // --- Interfaces (sem mudanças) ---
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
  export let initialMessages: ChatMessage[] = [];

  let messages: ChatMessage[] = initialMessages;
  let unlisten: UnlistenFn;
  let chatWindowElement: HTMLDivElement;
  let messageElements: HTMLElement[] = [];

  // --- Estado da Lógica de Scroll ---
  let userHasScrolledUp = false;
  let newMessagesCount = 0;
  let isResumingScroll = false;
  let firstNewMessageIndex: number | null = null;

  // --- NOVOS ESTADOS DE CONTROLE ---
  // Trava para ignorar eventos de scroll causados pelo auto-scroll
  let isAutoScrolling = false;
  // Guarda a última posição do scroll para detectar a direção do movimento
  let lastScrollTop = 0;

  /**
   * Rola a janela de chat para o fundo.
   */
  async function scrollToBottom(behavior: "smooth" | "auto" = "auto") {
    // ATIVA A TRAVA para que o handleScroll ignore este evento
    isAutoScrolling = true;

    await tick();
    if (chatWindowElement) {
      chatWindowElement.scroll({
        top: chatWindowElement.scrollHeight,
        behavior,
      });
      // Atualiza nossa referência da última posição
      lastScrollTop = chatWindowElement.scrollTop;
    }

    // Libera a trava após um pequeno delay para garantir que o evento de scroll já passou
    setTimeout(() => {
      isAutoScrolling = false;
    }, 100);
  }

  /**
   * Lógica executada quando o componente é montado no DOM.
   */
  onMount(() => {
    scrollToBottom();

    const throttledHandleScroll = throttle(handleScroll, 100);
    chatWindowElement.addEventListener("scroll", throttledHandleScroll);

    const setupListener = async () => {
      unlisten = await listen<ChatMessage>("new-chat-message", (event) => {
        if (event.payload.chatroomId.toString() === channelId) {
          const isAtBottomBeforeUpdate = !userHasScrolledUp;

          messages = [...messages, event.payload];

          if (userHasScrolledUp) {
            if (firstNewMessageIndex === null) {
              firstNewMessageIndex = messages.length - 1;
            }
            newMessagesCount++;
          } else if (isAtBottomBeforeUpdate) {
            scrollToBottom();
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
   * Função de scroll, agora mais inteligente.
   */
  // Substitua sua função handleScroll por esta
  function handleScroll() {
    // Ignora eventos de scroll causados pelo nosso próprio código
    if (isAutoScrolling || isResumingScroll || !chatWindowElement) return;

    const currentScrollTop = chatWindowElement.scrollTop;
    const scrollHeight = chatWindowElement.scrollHeight;
    const clientHeight = chatWindowElement.clientHeight;

    // Usamos um buffer pequeno para a detecção de "fundo"
    const isAtBottom = scrollHeight - currentScrollTop - clientHeight < 1;

    if (isAtBottom) {
      userHasScrolledUp = false;
      newMessagesCount = 0;
      firstNewMessageIndex = null;
    } else {
      // Só ativa o modo "rolou para cima" se o movimento foi de fato para cima
      if (currentScrollTop < lastScrollTop) {
        userHasScrolledUp = true;
      }

      // Se não há novas mensagens para rastrear, não faz nada.
      if (firstNewMessageIndex === null) return;

      // --- LÓGICA DE CONTAGEM REINSERIDA AQUI ---
      const chatWindowRect = chatWindowElement.getBoundingClientRect();
      let lastVisibleMessageIndex = -1;

      // Itera sobre as mensagens de baixo para cima.
      for (let i = messageElements.length - 1; i >= firstNewMessageIndex; i--) {
        const messageEl = messageElements[i];
        if (messageEl) {
          const rect = messageEl.getBoundingClientRect();
          // Se o topo da mensagem está visível na janela, encontramos nossa referência.
          if (rect.top < chatWindowRect.bottom) {
            lastVisibleMessageIndex = i;
            break;
          }
        }
      }

      if (lastVisibleMessageIndex !== -1) {
        const remaining = messages.length - (lastVisibleMessageIndex + 1);
        newMessagesCount = Math.max(0, remaining);
      } else {
        newMessagesCount = messages.length - firstNewMessageIndex;
      }
      // --- FIM DA LÓGICA DE CONTAGEM ---
    }

    // Atualiza a última posição do scroll no final
    lastScrollTop = currentScrollTop <= 0 ? 0 : currentScrollTop;
  }

  /**
   * Chamado ao clicar no botão "Voltar para o final".
   */
  function resumeAutoScroll() {
    isResumingScroll = true;
    userHasScrolledUp = false;
    newMessagesCount = 0;
    firstNewMessageIndex = null;

    scrollToBottom("smooth");

    setTimeout(() => {
      isResumingScroll = false;
    }, 500);
  }

  // --- Funções de Utilidade (sem mudanças) ---
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
    {#if messages.length === 0}
      <p class="status">Aguardando mensagens para o canal {channelId}...</p>
    {:else}
      {#each messages as msg, i (msg.id)}
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
        on:click={resumeAutoScroll}
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
    overflow-y: scroll;
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
    background-color: #ef4444; /* Vermelho para novas mensagens */
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      transform: scale(1) translateX(-50%);
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }
    50% {
      transform: scale(1.05) translateX(-48%);
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
