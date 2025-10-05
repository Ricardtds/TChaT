<script lang="ts">
  import ChatTab from "$lib/components/ChatTab.svelte";
  import Form from "$lib/components/Form.svelte";
  import Formulary from "$lib/components/Formulary.svelte";
  import Tab from "$lib/components/Tab.svelte";
  import type { TabItem } from "$lib/models/TabItem";
  import { invoke } from "@tauri-apps/api/core";
  import { load, type Store } from "@tauri-apps/plugin-store";
  import { onMount } from "svelte";
  let store: Store;
  let streammers: TabItem[] = $state([]);

  async function addStreammer(
    name: string,
    status: boolean,
    channel: number,
    chatroom: number,
  ) {
    if (
      !streammers.some(
        (s) => s.name.toLocaleLowerCase() === name.toLocaleLowerCase(),
      )
    ) {
      streammers = [...streammers, { name, status, channel, chatroom }];
    }
    showFormulary = false;
    activeChatroom = chatroom;

    await store.set("openTabs", streammers);
    await store.save();
  }

  async function removeStreammer(stream: TabItem) {
    streammers = streammers.filter((s) => s.name !== stream.name);
    await invoke("unsubscribe_from_channel", { chatroomId: stream.chatroom });
    await store.set("openTabs", streammers);
    await store.save();
  }

  onMount(async () => {
    store = await load("tchat.config.dat");
    streammers = await store.get("openTabs");
    activeChatroom = streammers[0]?.chatroom;

    streammers.forEach(async (streammer) => {
      try {
        await invoke("subscribe_to_channel", {
          chatroomId: Number(streammer.chatroom),
          chatroomName: streammer.name,
        });
      } catch (e) {
        console.error(
          `Erro ao realizar a inscrição do chatroom ${streammer.chatroom}:`,
          e,
        );
      }
    });
  });

  let activeChatroom = $state(0);
  let showFormulary: boolean = $state(false);
</script>

<main class="h-screen bg-[#111827] flex flex-col text-white">
  {#if showFormulary}
    <Formulary {addStreammer} />
  {/if}
  <div class="flex border-b-2 items-center px-2">
    <div class="w-full h-fit gap-1 flex pt-2">
      {#each streammers as stream}
        <div
          onclick={() => (activeChatroom = stream.chatroom)}
          class="hover:cursor-pointer"
          class:bg-blue-600={activeChatroom == stream.chatroom}
        >
          <Tab {stream} {removeStreammer} />
        </div>
      {/each}
      <button
        class=" hover:cursor-pointer flex border-[#374151] border-2 w-fit rounded-t-lg p-1 items-center gap-1"
        onclick={() => (showFormulary = true)}>+</button
      >
    </div>
    <div class="h-fit">⚙️</div>
  </div>
  <div class="w-full bg-[#111827] flex-1 text-white">
    {#each streammers as stream (stream.name)}
      <div
        class="h-full overflow-y-hidden"
        class:hidden={stream.chatroom !== activeChatroom}
      >
        <ChatTab {stream} />
      </div>
    {/each}
  </div>
  <div class="bg-[#585858] h-fit w-full">
    <Form />
  </div>
</main>
