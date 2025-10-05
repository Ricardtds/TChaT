<script lang="ts">
  import { getChatInfo } from "$lib/chat";
  let { addStreammer } = $props();

  let newChannelName: string = $state("");

  async function handleButton() {
    let resp = await getChatInfo(newChannelName);
    console.log(resp?.data);
    let isOnline = false;

    let slug: string = resp?.data.slug;
    if (resp?.data.livestream) {
      let title = resp?.data.livestream.session_title;
      isOnline = true;
    }
    let channelId = resp?.data.chatroom.channel_id;
    let chatroomId = resp?.data.chatroom.id;
    console.log(resp);

    addStreammer(slug, isOnline, channelId, chatroomId);
  }
</script>

<div
  class="bg-black/80 flex z-80 justify-center flex-col items-center absolute top-0 bottom-0 left-0 right-0"
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Escape"}
>
  <div
    class="p-20"
    role="dialog"
    tabindex="0"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => {
      if (e.key === "Enter") {
        e.preventDefault();
      }
    }}
  >
    <div
      class="connect-form flex flex-col gap-2 bg-[#1F2937] p-4 rounded-lg text-white"
    >
      <input
        type="text"
        class="text-center bg-[#111827] border-1 border-[#1E1E1E] rounded-lg placeholder:text-white/50 required:border-2 required:animate-bounce focus:animate-none"
        bind:value={newChannelName}
        placeholder="Nome do Canal"
        required={true}
      />
      <button
        class="bg-[#3B82F6] p-2 rounded-lg hover:cursor-pointer"
        onclick={handleButton}>Conectar</button
      >
    </div>
  </div>
</div>
