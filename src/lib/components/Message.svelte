<script lang="ts">
    import { type ChatMessage } from "$lib/chat";
    import { formatTimestamp, parseEmotes } from "$lib/utils";
    const { msg } = $props<{ msg: ChatMessage }>();
</script>

<div class="message">
    <span class="timestamp">[{formatTimestamp(msg.createdAt)}]</span>
    <span class="author" style="color: {msg.sender.identity.color};">
        {msg.sender.username}:
    </span>
    <span class="content">{@html parseEmotes(msg.content)}</span>
    i
</div>

<style>
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

    .content {
        color: #f9fafb;
        word-break: break-word;
    }
</style>
