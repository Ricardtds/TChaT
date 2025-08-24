<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import ChatTab from "$lib/components/ChatTab.svelte";

  // --- Estado do Componente (Usando Svelte 5 Runes) ---
  let store: Store;
  let newChannelName: string = $state("");
  let newChatroomId: string = $state("");

  type Tab = { id: string; title: string };
  let tabs: Tab[] = $state([]);
  let activeTabId: string | null = $state(null);

  let showPopup: boolean = $state(false);
  let tabsContainer: HTMLUListElement | null = null;

  // --- Estado para o Drag and Drop Nativo ---
  let draggedItemId: string | null = $state(null);
  let dragOverItemId: string | null = $state(null);

  // --- Funções da Interface ---
  function openPopup() {
    showPopup = true;
  }

  function closePopup() {
    showPopup = false;
  }

  function handleWheel(event: WheelEvent) {
    if (!tabsContainer) return;
    event.preventDefault();
    tabsContainer.scrollLeft += event.deltaY;
  }

  // --- Funções para o Drag and Drop Nativo ---

  function handleDragStart(event: DragEvent, tabId: string) {
    draggedItemId = tabId;
    // É necessário definir algum dado para o arraste funcionar no Firefox
    event.dataTransfer!.setData("text/plain", tabId);
    event.dataTransfer!.effectAllowed = "move";
  }

  function handleDragOver(event: DragEvent, tabId: string) {
    // Prevenir o comportamento padrão é CRUCIAL para permitir que o evento 'drop' funcione.
    event.preventDefault();
    if (tabId !== draggedItemId) {
      dragOverItemId = tabId;
    }
  }

  function handleDragLeave() {
    dragOverItemId = null;
  }

  function handleDrop(event: DragEvent, droppedOnTabId: string) {
    event.preventDefault();
    if (!draggedItemId || draggedItemId === droppedOnTabId) {
      return;
    }

    const draggedIndex = tabs.findIndex((t) => t.id === draggedItemId);
    const targetIndex = tabs.findIndex((t) => t.id === droppedOnTabId);

    if (draggedIndex === -1 || targetIndex === -1) return;

    // Lógica para reordenar o array
    const newTabsOrder = [...tabs];
    const [draggedItem] = newTabsOrder.splice(draggedIndex, 1);
    newTabsOrder.splice(targetIndex, 0, draggedItem);

    // Atualiza o estado do Svelte. O Svelte cuidará do DOM.
    // O $effect cuidará de salvar o novo estado.
    tabs = newTabsOrder;
  }

  function handleDragEnd() {
    // Limpa o estado ao final do arraste para remover os estilos visuais
    draggedItemId = null;
    dragOverItemId = null;
  }

  // Efeito reativo para salvar o estado sempre que as abas mudarem.
  $effect(() => {
    if (store) {
      saveTabsState();
    }
  });

  async function saveTabsState() {
    if (!store) return;
    const openTabs = tabs.map((t) => ({ id: t.id, title: t.title }));
    await store.set("openTabs", openTabs);
    await store.save();
  }

  async function addChatTab() {
    if (
      !newChatroomId ||
      !newChannelName ||
      tabs.find((t) => t.id === newChatroomId)
    ) {
      return;
    }
    closePopup();
    try {
      await invoke("subscribe_to_channel", { chatroomId: newChatroomId });
      const newTab: Tab = { id: newChatroomId, title: newChannelName };
      tabs.push(newTab);
      activeTabId = newChatroomId;
      newChannelName = "";
      newChatroomId = "";
      saveTabsState();
    } catch (e) {
      console.error(`Erro ao se inscrever no chatroom ${newChatroomId}: ${e}`);
    }
  }

  async function closeTab(tabIdToClose: string) {
    try {
      await invoke("unsubscribe_from_channel", { chatroomId: tabIdToClose });
      saveTabsState();
    } catch (e) {
      console.error(
        `Erro ao cancelar a inscrição do chatroom ${tabIdToClose}:`,
        e
      );
    }
    const indexToRemove = tabs.findIndex((tab) => tab.id === tabIdToClose);
    if (indexToRemove > -1) {
      tabs.splice(indexToRemove, 1);
    }
    if (activeTabId === tabIdToClose) {
      activeTabId = tabs.at(0)?.id ?? null;
    }
  }

  onMount(async () => {
    store = await load("tchat.config.dat");
    const savedTabs = await store.get<Tab[]>("openTabs");
    if (!savedTabs || savedTabs.length === 0) return;

    const connectionPromises = savedTabs.map((tab) =>
      invoke("subscribe_to_channel", { chatroomId: tab.id })
        .then(() => ({ ...tab, status: "success" as const }))
        .catch((e) => {
          console.error(`Falha ao se reinscrever no chatroom ${tab.id}:`, e);
          return { ...tab, status: "failure" as const };
        })
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
            ondragstart={(e) => handleDragStart(e, tab.id)}
            ondragover={(e) => handleDragOver(e, tab.id)}
            ondragleave={handleDragLeave}
            ondrop={(e) => handleDrop(e, tab.id)}
            ondragend={handleDragEnd}
          >
            <button class="tab-title" onclick={() => (activeTabId = tab.id)}>
              {tab.title}
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
    <div
      class="popup-backdrop"
      role="button"
      tabindex="0"
      onclick={closePopup}
      onkeydown={(e) => e.key === "Escape" && closePopup()}
    >
      <div
        class="popup"
        role="dialog"
        tabindex="0"
        onclick={(e) => e.stopPropagation()}
      >
        <div class="connect-form">
          <input bind:value={newChannelName} placeholder="Nome do Canal" />
          <input
            bind:value={newChatroomId}
            placeholder="ID da Sala de Chat"
            onkeydown={(e) => e.key === "Enter" && addChatTab()}
          />
          <button onclick={addChatTab}>Conectar</button>
        </div>
      </div>
    </div>
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

  .popup-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .popup {
    background: #1f2937;
    padding: 1.5rem;
    border-radius: 8px;
    border: 1px solid #374151;
    min-width: 400px;
    max-width: 90%;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
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

  .connect-form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .connect-form input {
    padding: 0.75rem;
    background-color: #111827;
    border: 1px solid #374151;
    color: #e5e7eb;
    border-radius: 6px;
    font-size: 1rem;
  }
  .connect-form button {
    padding: 0.75rem 1.5rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s;
  }
  .connect-form button:hover {
    background-color: #2563eb;
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
