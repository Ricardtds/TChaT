<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import { onMount } from "svelte";
  import { writable, type Writable } from "svelte/store";
  import ChatTab from "$lib/components/ChatTab.svelte";
  import Formulary from "$lib/components/Formulary.svelte";
  import { type Channel } from "$lib/chat";

  let store: Store;

  type Tab = { id: number; title: string };
  let tabs: Tab[] = $state([]);
  type ChatTabsStore = Writable<Channel[]>;
  export const chatTabs: ChatTabsStore = writable([]);
  let activeTabId: number | null = $state(null);

  let showPopup: boolean = $state(false);
  let tabsContainer: HTMLUListElement | null = null;

  // --- Estado para o Drag and Drop Nativo ---
  let draggedItemId: number | null = $state(null);
  let dragOverItemId: number | null = $state(null);

  // --- Funções da Interface ---
  function openPopup(): void {
    showPopup = true;
  }

  function closePopup(): void {
    showPopup = false;
  }

  function handleWheel(event: WheelEvent) {
    if (!tabsContainer) return;
    event.preventDefault();
    tabsContainer.scrollLeft += event.deltaY;
  }

  $effect(() => {
    if (store) {
      saveTabsState();
    }
  });

  function removeChatTab(id: number) {
    chatTabs.update((tabs) => tabs.filter((tab) => tab.id !== id));
  }

  async function saveTabsState() {
    if (!store) return;
    const openTabs = tabs.map((t) => ({ id: t.id, title: t.title }));
    await store.set("openTabs", openTabs);
    await store.save();
  }

  async function addChatTab(newChannelName: string, newChatroomId: number) {
    try {
      await invoke("subscribe_to_channel", { chatroomId: newChatroomId });
      tabs = [...tabs, { id: newChatroomId, title: newChannelName }];
      activeTabId = newChatroomId;
      saveTabsState();
    } catch (e) {
      console.error(
        `Erro ao realizar a inscrição do chatroom ${newChatroomId}:`,
        e,
      );
    }
  }
  async function closeTab(tabIdToClose: number) {
    tabIdToClose = Number(tabIdToClose);
    try {
      await invoke("unsubscribe_from_channel", { chatroomId: tabIdToClose });
      const indexToRemove = tabs.findIndex((tab) => tab.id === tabIdToClose);
      if (indexToRemove > -1) {
        tabs.splice(indexToRemove, 1);
      }
      if (activeTabId === tabIdToClose) {
        activeTabId = tabs.at(0)?.id ?? null;
      }
      saveTabsState();
    } catch (e) {
      console.error(
        `Erro ao cancelar a inscrição do chatroom ${tabIdToClose}:`,
        e,
      );
    }
  }

  onMount(async () => {
    store = await load("tchat.config.dat");
    const savedTabs = await store.get<Tab[]>("openTabs");
    if (!savedTabs || savedTabs.length === 0) return;

    const connectionPromises = savedTabs.map((tab) =>
      invoke("subscribe_to_channel", { chatroomId: Number(tab.id) })
        .then(() => ({ ...tab, status: "success" as const }))
        .catch((e) => {
          console.error(`Falha ao se reinscrever no chatroom ${tab.id}:`, e);
          return { ...tab, status: "failure" as const };
        }),
    );
    const results = await Promise.all(connectionPromises);
    tabs = results.reduce<Tab[]>((acc, result) => {
      if (result.status === "success")
        acc.push({ id: result.id, title: result.title });
      return acc;
    }, []);
    if (tabs.length > 0) activeTabId = tabs.at(0)?.id ?? null;
  });
</script>

<main class="container">
  <header class="header">
    <nav>
      <ul bind:this={tabsContainer} onwheel={handleWheel}>
        {#each tabs as tab (tab.id)}
          <li
            draggable="true"
            class:active={tab.id === activeTabId}
            class:dragging={draggedItemId === tab.id}
            class:drag-over={dragOverItemId === tab.id}
          >
            <button class="tab-title" onclick={() => (activeTabId = tab.id)}>
              {tab.title}
              {tab.id}
            </button>
            <button
              class="tab-close"
              onclick={(event) => {
                event.stopPropagation();
                closeTab(tab.id);
              }}
            >
              &times;
            </button>
          </li>
        {/each}
        <li>
          <button
            class="tab-add"
            onclick={openPopup}
            aria-label="Adicionar nova aba de chat"
          >
            <svg
              class="w-6 h-6"
              aria-hidden="true"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              fill="none"
              viewBox="0 0 24 24"
            >
              <path
                stroke="currentColor"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 5v14m-7-7h14"
              />
            </svg>
          </button>
        </li>
      </ul>
    </nav>
  </header>

  {#if showPopup}
    <Formulary {addChatTab} {closePopup} />
  {/if}

  <div class="tabs-content">
    {#each tabs as tab (tab.id)}
      <div class="tab-pane" class:active={activeTabId === tab.id}>
        <ChatTab channelId={tab.id} active={activeTabId === tab.id} />
      </div>
    {/each}
  </div>
</main>

<style>
  :global(html, body) {
    padding: 0;
    margin: 0;
    overflow: hidden;
    box-sizing: border-box;
  }
  :global(*, *:before, *:after) {
    box-sizing: inherit;
  }

  .container {
    display: flex;
    flex-direction: column;
    padding: 0;
    margin: 0 auto;
    width: 100%;
    height: 100vh;
  }
  .tabs-content {
    flex-grow: 1;
    min-height: 0;
    display: flex;
    padding: 0 1rem 1rem 1rem;
  }
  .header {
    flex-shrink: 0;
    padding: 1rem 1rem 0 1rem;
    border-bottom: 1px solid #374151;
  }

  .tab-pane {
    display: none;
    flex-grow: 1;
  }
  .tab-pane.active {
    display: flex;
  }

  nav ul {
    display: flex;
    list-style: none;
    margin: 0;
    padding: 0;
    flex-wrap: nowrap;
    overflow-x: auto;
    padding-bottom: 2px;
  }

  nav ul::-webkit-scrollbar {
    height: 6px;
  }
  nav ul::-webkit-scrollbar-track {
    background: transparent;
  }
  nav ul::-webkit-scrollbar-thumb {
    background: #374151;
    border-radius: 3px;
  }
  nav ul::-webkit-scrollbar-thumb:hover {
    background: #4b5563;
  }

  li {
    display: flex;
    align-items: center;
    border: 1px solid transparent;
    border-bottom: none;
    position: relative;
    top: 1px;
    background-color: #1f2937;
    border-radius: 6px 6px 0 0;
    margin-right: 4px;
    transition: transform 0.2s ease;
    flex-shrink: 0;
  }

  /* Estilos para o feedback visual do Drag and Drop Nativo */
  li[draggable="true"] {
    cursor: move;
  }

  li.dragging {
    opacity: 0.5;
  }

  /* Cria uma linha azul à esquerda da aba onde o item será solto */
  li.drag-over::before {
    content: "";
    position: absolute;
    left: -2px;
    top: 4px;
    bottom: 4px;
    width: 4px;
    background-color: #3b82f6;
    border-radius: 2px;
  }

  li.active {
    background-color: #111827;
    border-color: #374151;
    border-bottom-color: transparent;
  }
  li.active button.tab-title {
    color: #60a5fa;
    font-weight: 500;
  }
  button {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 0.9rem;
    color: #9ca3af;
    transition: color 0.2s ease-in-out;
  }
  .tab-title {
    padding: 0.75rem 1rem;
    white-space: nowrap;
  }
  .tab-title:hover {
    color: #e5e7eb;
  }
  .tab-close {
    padding: 0 0.5rem;
    font-size: 1.2rem;
    color: #6b7280;
    border-radius: 99px;
  }
  .tab-close:hover {
    color: #e5e7eb;
    background-color: rgba(255, 255, 255, 0.1);
  }
  .tab-add {
    padding: 0.75rem;
    display: flex;
    align-items: center;
  }
  .tab-add svg {
    width: 1.2em;
    height: 1.2em;
  }
  .tab-add:hover {
    color: #e5e7eb;
  }
</style>
