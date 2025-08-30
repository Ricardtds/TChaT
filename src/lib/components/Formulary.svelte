<script lang="ts">
    import { preventDefault } from "svelte/legacy";

    let newChannelName: string = $state("");
    let newChatroomId: number = $state(0);

    const { addChatTab, closePopup } = $props<{
        addChatTab: (newChannelName: string, newChatroomId: number) => void;
        closePopup: () => void;
    }>();

    function handleConnect() {
        console.log(newChannelName, newChatroomId);

        addChatTab(newChannelName, newChatroomId);
        closePopup();
    }
</script>

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
        onkeydown={(e) => {
            if (e.key === "Enter") {
                e.preventDefault();
            }
        }}
    >
        <div class="connect-form">
            <input
                type="text"
                bind:value={newChannelName}
                placeholder="Nome do Canal"
            />
            <input
                type="text"
                bind:value={newChatroomId}
                placeholder="ID da Sala de Chat"
                onkeydown={(e) => e.key === "Enter" && handleConnect()}
            />
            <button onclick={handleConnect}>Conectar</button>
        </div>
    </div>
</div>

<style>
    /* Your CSS remains the same. */
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
</style>
