<script lang="ts">
  import { onMount, tick } from "svelte";
  import VirtualList from "svelte-tiny-virtual-list";
  import Message from "./Message.svelte";
  import type { ItemProps } from "$lib/models/ItemProps";
  import { getChatHistory, type ChatMessage } from "$lib/chat";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import type { TabItem } from "$lib/models/TabItem";
  let unlisten: UnlistenFn;

  let { stream }: { stream: TabItem } = $props<{ stream: TabItem }>();

  let scrollToIndex: number | null = $state(0);

  let virtualList: VirtualList;
  let height: number = $state(0);
  let end: number = $state(0);
  let scrolled: boolean = $state(false);
  let messages: ChatMessage[] = $state([]);
  let itemHeights: number[] = $state([]);

  $effect(() => {
    if (virtualList) {
      virtualList.recomputeSizes();
    }
  });

  onMount(async () => {
    let resp = await getChatHistory(stream.channel);
    messages = resp?.data.data.messages.map(mapMessage);
    console.log(resp);

    itemHeights = Array(500).fill(0);
    await tick();
    scrollToIndex = messages.length - 1;
    setupListener();
  });

  function mapMessage(apiMessage: any): ChatMessage {
    return {
      chatId: apiMessage.chat_id,
      content: apiMessage.content,
      createdAt: apiMessage.created_at,
      id: apiMessage.id,
      sender: apiMessage.sender,
      type: "message",
    };
  }

  async function handleButton() {
    if (virtualList) {
      scrollToIndex = null;
      await tick();
      scrollToIndex = messages.length - 1;
    }
  }

  const setupListener = async () => {
    unlisten = await listen<Array<ChatMessage>>(
      `new-chat-messages-${stream.chatroom}`,
      async (event) => {
        event.payload.forEach(async (message) => {
          messages = [...messages, message];
          if (messages.length > 500) {
            messages.splice(0, messages.length - 500);
          }
        });
        if (!scrolled) {
          scrollToIndex = null;
          await tick();
          scrollToIndex = messages.length - 1;
        }
      },
    );
  };
  function handleListUpdate(data: any) {
    if (messages.length > data.detail.end + 1) {
      scrolled = true;
    } else {
      scrolled = false;
    }
    end = messages.length - data.detail.end;
  }
</script>

<div class="h-full relative flex-col z-50">
  <div class="h-full" bind:clientHeight={height}>
    <VirtualList
      bind:this={virtualList}
      on:itemsUpdated={(data) => handleListUpdate(data)}
      width="100%"
      {height}
      itemCount={messages.length}
      itemSize={(index: number) => itemHeights[index]}
      {scrollToIndex}
      scrollToAlignment="center"
      scrollToBehaviour="smooth"
    >
      {#snippet item({ style, index }: ItemProps)}
        <div {style}>
          <div class="h-fit" bind:clientHeight={itemHeights[index]}>
            <Message message={messages[index]} />
          </div>
        </div>
      {/snippet}
    </VirtualList>
  </div>
  {#if scrolled}
    <button
      class="rounded-2xl bg-[#EF4444] p-2 absolute w-fit h-fit m-auto left-0 right-0 bottom-5 animate-pulse hover:cursor-pointer"
      onclick={handleButton}
    >
      {#if end > 0}
        ↓ {end} Mensagens
      {:else}
        ↓ Final
      {/if}
    </button>
  {/if}
</div>
