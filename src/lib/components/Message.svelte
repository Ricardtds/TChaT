<script lang="ts">
  import { parseMessage, type ChatMessage, type MessagePart } from "$lib/chat";
  import { formatTimestamp } from "$lib/utils";
  import Emote from "./Emote.svelte";
  export let message: ChatMessage;

  const messageParts: MessagePart[] = message
    ? parseMessage(message.content)
    : [];
</script>

<!-- <div class="mb-1 max-w-full text-sm hover:bg-[#1F2936]" id={message.id}>
  <p class="break-words overflow-wrap-anywhere">
    <span class="text-[#999999]">[{formatTimestamp(message.created_at)}]</span>
    <span style="color: {message.sender.identity.color};" class="font-semibold"
      >{message.sender.username}:</span
    >
    <span class="font-regular">{message.content}</span>
  </p>
</div> -->

<div class="message">
  <p class="break-words overflow-wrap-anywhere">
    <span class="timestamp">[{formatTimestamp(message.createdAt)}]</span>
    <span class="author" style="color: {message.sender.identity.color};">
      {#each message.sender.badges as badge}
        <img
          src={badge.url}
          alt={badge.text}
          class="badge"
          title={badge.text}
        />
      {/each}
      {message.sender.username}:
    </span>

    <span>
      {#each messageParts as part}
        {#if part.type === "text"}
          {part.content}
        {:else if part.type === "emote"}
          <Emote id={part.id} name={part.name} />
        {/if}
      {/each}
    </span>
  </p>
</div>
