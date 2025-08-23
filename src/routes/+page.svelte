<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import ChatTab from "$lib/components/ChatTab.svelte";

  let store: Store;
  let newChannelName: string = $state("");
  let newChatroomId: string = $state("");

  type Tab = { id: string; title: string };
  let tabs: Tab[] = $state([]);
  let activeTabId: string | null = $state(null);

  // --- Estado para o Drag and Drop Nativo ---
  let draggedItemId: string | null = $state(null);
  let dragOverItemId: string | null = $state(null);

  // --- Funções para o Drag and Drop Nativo ---

  function handleDragStart(event: DragEvent, tabId: string) {
    draggedItemId = tabId;
    event.dataTransfer!.setData("text/plain", tabId);
    event.dataTransfer!.effectAllowed = "move";
  }

  function handleDragOver(event: DragEvent, tabId: string) {
    event.preventDefault();
    if (tabId !== draggedItemId) {
      dragOverItemId = tabId;
    }
  }

  async function handleDrop(event: DragEvent, droppedOnTabId: string) {
    event.preventDefault();
    if (!draggedItemId || draggedItemId === droppedOnTabId) {
      return;
    }

    const draggedIndex = tabs.findIndex((t) => t.id === draggedItemId);
    const targetIndex = tabs.findIndex((t) => t.id === droppedOnTabId);

    if (draggedIndex === -1 || targetIndex === -1) return;

    const newTabsOrder = [...tabs];
    const [draggedItem] = newTabsOrder.splice(draggedIndex, 1);
    newTabsOrder.splice(targetIndex, 0, draggedItem);

    tabs = newTabsOrder;
    // Salva o estado explicitamente após reordenar
    await saveTabsState();
  }

  function handleDragEnd() {
    draggedItemId = null;
    dragOverItemId = null;
  }

  // --- LÓGICA DE SALVAMENTO CORRIGIDA ---

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
    try {
      await invoke("connect_chat", { chatroomId: newChatroomId });
      const newTab: Tab = { id: newChatroomId, title: newChannelName };
      tabs.push(newTab);
      activeTabId = newChatroomId;

      // MUDANÇA: Agora esperamos o salvamento terminar antes de continuar
      await saveTabsState();

      // Só limpa os inputs depois que tudo foi salvo com sucesso
      newChatroomId = "";
      newChannelName = "";
    } catch (e) {
      console.error(`Erro ao conectar no chatroom ${newChatroomId}: ${e}`);
    }
  }

  async function closeTab(tabIdToClose: string) {
    try {
      await invoke("disconnect_chat", { chatroomId: tabIdToClose });
    } catch (e) {
      console.error(
        `Erro ao tentar desconectar do chatroom ${tabIdToClose}:`,
        e
      );
    }
    const indexToRemove = tabs.findIndex((tab) => tab.id === tabIdToClose);
    if (indexToRemove > -1) {
      tabs.splice(indexToRemove, 1);
    }
    if (activeTabId === tabIdToClose) {
      activeTabId = tabs.length > 0 ? tabs[0].id : null;
    }
    // Salva o estado explicitamente após fechar
    await saveTabsState();
  }

  onMount(async () => {
    store = await load("tchat.config.dat");
    const savedTabs = await store.get<Tab[]>("openTabs");
    if (!savedTabs || savedTabs.length === 0) return;

    const connectionPromises = savedTabs.map((tab) =>
      invoke("connect_chat", { chatroomId: tab.id })
        .then(() => ({ ...tab, status: "success" as const }))
        .catch((e) => {
          console.error(`Falha ao reconectar ao chatroom ${tab.id}:`, e);
          return { ...tab, status: "failure" as const };
        })
    );
    const results = await Promise.all(connectionPromises);
    tabs = results.reduce<Tab[]>((acc, result) => {
      if (result.status === "success")
        acc.push({ id: result.id, title: result.title });
      return acc;
    }, []);
    if (tabs.length > 0) activeTabId = tabs[0].id;
  });
</script>

<main class="container">
  <div class="connect-form">
    <input bind:value={newChannelName} placeholder="Nome do Canal" />
    <input
      bind:value={newChatroomId}
      placeholder="ID da Sala de Chat"
      onkeydown={(e) => e.key === "Enter" && addChatTab()}
    />
    <button onclick={addChatTab}>Conectar</button>
  </div>

  <header class="header">
    <nav>
      <ul>
        {#each tabs as tab (tab.id)}
          <li
            draggable="true"
            class:active={tab.id === activeTabId}
            class:dragging={draggedItemId === tab.id}
            class:drag-over={dragOverItemId === tab.id}
            ondragstart={(e) => handleDragStart(e, tab.id)}
            ondragover={(e) => handleDragOver(e, tab.id)}
            ondragleave={() => (dragOverItemId = null)}
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
      </ul>
    </nav>
  </header>

  <div class="tabs-content">
    {#each tabs as tab (tab.id)}
      <div class="tab-pane" class:active={activeTabId === tab.id}>
        <ChatTab channelId={tab.id} />
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
    padding: 1rem;
    max-width: 900px;
    margin: 0 auto;
    width: 100%;
    height: 100vh;
  }
  .tabs-content {
    flex-grow: 1;
    min-height: 0;
    border: 1px solid #374151;
    border-top: none;
    display: flex;
  }
  .header,
  .connect-form {
    flex-shrink: 0;
  }
  .connect-form {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
  }
  .connect-form > * {
    flex: 1 1 200px;
  }
  .connect-form input {
    padding: 0.75rem;
    background-color: #1f2937;
    border: 1px solid #374151;
    color: #e5e7eb;
    border-radius: 6px;
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
  header {
    background-color: transparent;
    border-bottom: 1px solid #374151;
    padding: 0;
  }
  nav ul {
    display: flex;
    list-style: none;
    margin: 0;
    padding: 0;
    flex-wrap: nowrap;
    overflow-x: auto;
    padding-bottom: 4px;
  }
  li {
    display: flex;
    align-items: center;
    border: 1px solid transparent;
    border-bottom: none;
    position: relative;
    background-color: #1f2937;
    border-radius: 6px 6px 0 0;
    margin-right: 4px;
    flex-shrink: 0;
    transition: transform 0.2s ease;
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
    border-bottom-color: #111827;
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
  }
  .tab-close:hover {
    color: #e5e7eb;
  }
</style>
