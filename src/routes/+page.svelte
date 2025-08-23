<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import ChatTab from "$lib/components/ChatTab.svelte";

  let store: Store;
  let newChannelName: string = "";
  let newChatroomId: string = "";

  type Tab = { id: string; title: string };
  let tabs: Tab[] = [];
  let activeTabId: string | null = null;

  async function saveTabsState() {
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
      console.warn(
        "Nome do canal e ID da sala são obrigatórios e o ID não pode ser duplicado."
      );
      return;
    }
    try {
      await invoke("connect_chat", { chatroomId: newChatroomId });
      const newTab: Tab = {
        id: newChatroomId,
        title: newChannelName,
      };
      tabs = [...tabs, newTab];
      activeTabId = newChatroomId;

      await saveTabsState();

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

    tabs = tabs.filter((tab) => tab.id !== tabIdToClose);

    if (activeTabId === tabIdToClose) {
      activeTabId = tabs.length > 0 ? tabs[0].id : null;
    }

    await saveTabsState();
  }

  onMount(async () => {
    store = await load(".tabs.dat");
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
    const successfullyReconnectedTabs = results.reduce<Tab[]>((acc, result) => {
      if (result.status === "success") {
        acc.push({ id: result.id, title: result.title });
      }
      return acc;
    }, []);

    if (successfullyReconnectedTabs.length > 0) {
      tabs = successfullyReconnectedTabs;
      activeTabId = successfullyReconnectedTabs[0].id;
    }
  });
</script>

<main class="container">
  <div class="connect-form">
    <input bind:value={newChannelName} placeholder="Nome do Canal" />
    <input
      bind:value={newChatroomId}
      placeholder="ID da Sala de Chat"
      on:keydown={(e) => e.key === "Enter" && addChatTab()}
    />
    <button on:click={addChatTab}>Conectar</button>
  </div>

  <header class="header">
    <nav>
      <ul>
        {#each tabs as tab (tab.id)}
          <li class:active={tab.id === activeTabId}>
            <button class="tab-title" on:click={() => (activeTabId = tab.id)}>
              {tab.title}
            </button>
            <button class="tab-close" on:click={() => closeTab(tab.id)}
              >×</button
            >
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
  .container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100%;
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
  }

  .tabs-content {
    flex-grow: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    border: 1px solid #374151;
    border-top: none;
    border-radius: 0 0 6px 6px;
  }

  .tab-pane {
    display: none;
  }

  .tab-pane.active {
    display: flex;
    flex-direction: column;
    flex-grow: 1;
    min-height: 0;
  }

  .header,
  .connect-form {
    flex-shrink: 0;
  }
  .connect-form {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .connect-form input {
    flex-grow: 1;
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
    flex-wrap: wrap;
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
    margin-bottom: -1px;
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
