<script lang="ts">
  import { type ChatMessageWithHeight } from "$lib/chat";
  import { formatTimestamp } from "$lib/utils";
  import { parseMessage, type MessagePart } from "$lib/chat";
  import Emote from "$lib/components/Emote.svelte";

  export let msg: ChatMessageWithHeight;

  const messageParts: MessagePart[] = msg ? parseMessage(msg.content) : [];
</script>

{#if msg}
  <div
    class="message"
    style="height: {msg._height ? msg._height + 'px' : 'auto'};"
  >
    <span class="timestamp">[{formatTimestamp(msg.createdAt)}]</span>
    <span class="author" style="color: {msg.sender.identity.color};">
      {#each msg.sender.identity.badges as badge}
        <img
          src={badge.url}
          alt={badge.text}
          class="badge"
          title={badge.text}
        />
      {/each}
      {msg.sender.username}:
    </span>

    <span class="content">
      {#each messageParts as part}
        {#if part.type === "text"}
          {part.content}
        {:else if part.type === "emote"}
          <Emote id={part.id} name={part.name} />
        {/if}
      {/each}
    </span>
  </div>
{/if}

<style>
  .message {
    padding: 0.25rem 0.5rem;
    line-height: 1.6;
    border-radius: 4px;
    transition: background-color 0.2s;
    font-size: 1rem;
    color: #f9fafb;
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
  .content {
    word-break: break-word;
  }
  .badge {
    height: 1.2em;
    margin-right: 0.3em;
    vertical-align: middle;
  }
</style>
